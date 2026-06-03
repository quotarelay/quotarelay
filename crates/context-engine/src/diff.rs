use std::io;
use std::path::Path;

use repo_index::{changed_documents, index_freshness, matching_documents, ChangedDocumentStatus};

use super::*;
use crate::budget::{budget_estimate, included_document_bytes};

pub fn assemble_diff_aware(
    root: &Path,
    query: Option<&str>,
    limit: usize,
) -> io::Result<ContextCapsule> {
    let capped_limit = limit.clamp(1, MAX_CONTEXT_ITEMS);
    let freshness = index_freshness(root)?;
    let mut prepared = changed_documents(root, capped_limit + 1)?
        .into_iter()
        .map(|document| {
            let status = match document.status {
                ChangedDocumentStatus::Changed => "changed",
                ChangedDocumentStatus::New => "new",
            };
            ContextDocument {
                path: document.path,
                contents: document.contents,
                reason: InclusionReason {
                    kind: InclusionReasonKind::DiffChangedFile,
                    detail: format!("Local {status} file differs from the last explicit sync."),
                },
            }
        })
        .collect::<Vec<_>>();

    if let Some(query) = query.filter(|value| !value.trim().is_empty()) {
        let existing_paths = prepared
            .iter()
            .map(|document| document.path.clone())
            .collect::<std::collections::BTreeSet<_>>();
        for document in matching_documents(root, query, capped_limit + 1)? {
            if prepared.len() >= capped_limit + 1 {
                break;
            }
            if existing_paths.contains(&document.path) {
                continue;
            }
            prepared.push(ContextDocument {
                path: document.path,
                contents: document.contents,
                reason: InclusionReason {
                    kind: InclusionReasonKind::DiffRelatedMatch,
                    detail: format!("Indexed document matched query '{query}'."),
                },
            });
        }
    }

    let raw_document_bytes = prepared
        .iter()
        .map(|document| document.contents.len())
        .sum::<usize>();
    let (documents, mut omissions) = pack_prepared_documents(prepared, capped_limit);
    if documents.is_empty() && !freshness.is_stale {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoDocumentMatches,
            detail: "No local file changes detected since the last explicit sync.".to_string(),
        });
    }
    if freshness.missing_count > 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::MissingIndexedFile,
            detail: format!(
                "{} indexed files are missing locally and cannot be packed.",
                freshness.missing_count
            ),
        });
    }

    let mut capsule = ContextCapsule {
        generated_at_epoch_ms: now_epoch_ms()?,
        documents,
        omissions,
        cache_status: crate::retrieval::cache_status(
            CacheStatusKind::NotApplicable,
            "diff_aware reads current local changes and is not cached.",
        ),
        budget: budget_estimate(0, raw_document_bytes),
        stale: ContextStaleStatus::default(),
    };
    capsule.budget = budget_estimate(
        included_document_bytes(&capsule.documents),
        capsule.budget.raw_bytes_considered,
    );
    capsule.stale = stale_status(root)?;
    Ok(capsule)
}

fn pack_prepared_documents(
    documents: Vec<ContextDocument>,
    limit: usize,
) -> (Vec<ContextDocument>, Vec<OmissionReason>) {
    let total_documents = documents.len();
    let mut packed = Vec::new();
    let mut omissions = Vec::new();
    let mut remaining_bytes = MAX_DOCUMENT_PACK_BYTES;
    let mut byte_budget_hit = false;

    for document in documents.into_iter().take(limit) {
        let Some((contents, truncated)) =
            fit_within_budget(&document.contents, &mut remaining_bytes)
        else {
            byte_budget_hit = true;
            break;
        };
        packed.push(ContextDocument {
            contents,
            ..document
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if total_documents > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!(
                "Additional diff-aware documents omitted because item limit {limit} was reached."
            ),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!(
                "Packed diff-aware documents reached byte budget {MAX_DOCUMENT_PACK_BYTES}."
            ),
        });
    }

    (packed, omissions)
}

fn stale_status(root: &Path) -> io::Result<ContextStaleStatus> {
    let freshness = index_freshness(root)?;
    Ok(ContextStaleStatus {
        is_stale: freshness.is_stale,
        changed_files: freshness.changed_count,
        missing_files: freshness.missing_count,
        new_files: freshness.new_count,
    })
}
