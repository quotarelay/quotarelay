use std::io;
use std::path::Path;

use repo_index::{index_freshness, indexed_documents, matching_documents, search_code};

use super::*;
use crate::diff::assemble_diff_aware;
use crate::memory::{find_memory_notes, pack_memory_notes};

pub(crate) fn normalize_query(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub fn assemble_context(root: &Path, query: &str, limit: usize) -> io::Result<ContextAssembly> {
    let query = normalize_query(query);
    let capped_limit = limit.clamp(1, 5);
    if let Some(assembly) = read_exact_match_cache(root, &query)? {
        let assembly = assembly_with_cache_status(
            assembly_with_budget_and_stale(root, assembly)?,
            CacheStatusKind::Hit,
            "exact_search cache entry returned this context pack.",
        );
        append_history(root, &assembly)?;
        return Ok(assembly);
    }

    let hits = search_code(root, &query, capped_limit + 1)?;
    let (snippets, omissions) = pack_snippets(&query, hits, capped_limit);
    let (memory_notes, memory_omissions) =
        pack_memory_notes(&query, find_memory_notes(root, &query)?, capped_limit);
    let mut omissions = omissions;
    omissions.extend(memory_omissions);
    let assembly = assembly_with_budget_and_stale(
        root,
        ContextAssembly {
            query: query.clone(),
            generated_at_epoch_ms: now_epoch_ms()?,
            snippets,
            memory_notes,
            omissions,
            cache_status: cache_status(
                CacheStatusKind::Miss,
                "exact_search cache miss; context pack was assembled and cached.",
            ),
            budget: ContextBudgetEstimate::default(),
            stale: ContextStaleStatus::default(),
        },
    )?;

    persist_exact_match_cache(root, &query, &assembly)?;
    append_history(root, &assembly)?;

    Ok(assembly)
}

pub fn context_run_history(root: &Path, limit: usize) -> io::Result<Vec<ContextAssembly>> {
    let capped_limit = limit.clamp(1, 5);
    let history = load_history(root)?;

    Ok(history.runs.into_iter().rev().take(capped_limit).collect())
}

pub fn context_run_detail(
    root: &Path,
    generated_at_epoch_ms: u128,
) -> io::Result<Option<ContextAssembly>> {
    let history = load_history(root)?;

    Ok(history
        .runs
        .into_iter()
        .rev()
        .take(MAX_HISTORY_RUNS)
        .find(|run| run.generated_at_epoch_ms == generated_at_epoch_ms))
}

pub fn retrieve_context(
    root: &Path,
    mode: RetrievalMode,
    query: Option<&str>,
    limit: usize,
) -> io::Result<RetrievedContext> {
    match mode {
        RetrievalMode::ExactSearch => {
            let query = query.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "exact_search requires a query")
            })?;
            let assembly = assemble_context(root, query, limit)?;
            Ok(RetrievedContext {
                mode,
                query: Some(assembly.query.clone()),
                generated_at_epoch_ms: assembly.generated_at_epoch_ms,
                snippets: assembly.snippets,
                memory_notes: assembly.memory_notes,
                documents: Vec::new(),
                omissions: assembly.omissions,
                cache_status: assembly.cache_status,
                budget: assembly.budget,
                stale: assembly.stale,
            })
        }
        RetrievalMode::Overview => {
            let capsule = assemble_overview(root, limit)?;
            Ok(RetrievedContext {
                mode,
                query: None,
                generated_at_epoch_ms: capsule.generated_at_epoch_ms,
                snippets: Vec::new(),
                memory_notes: Vec::new(),
                documents: capsule.documents,
                omissions: capsule.omissions,
                cache_status: capsule.cache_status,
                budget: capsule.budget,
                stale: capsule.stale,
            })
        }
        RetrievalMode::TaskCapsule => {
            let query = query.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "task_capsule requires a query")
            })?;
            let normalized_query = normalize_query(query);
            let capsule = assemble_task_capsule(root, &normalized_query, limit)?;
            Ok(RetrievedContext {
                mode,
                query: Some(normalized_query),
                generated_at_epoch_ms: capsule.generated_at_epoch_ms,
                snippets: Vec::new(),
                memory_notes: Vec::new(),
                documents: capsule.documents,
                omissions: capsule.omissions,
                cache_status: capsule.cache_status,
                budget: capsule.budget,
                stale: capsule.stale,
            })
        }
        RetrievalMode::DiffAware => {
            let normalized_query = query.map(normalize_query);
            let capsule = assemble_diff_aware(root, normalized_query.as_deref(), limit)?;
            Ok(RetrievedContext {
                mode,
                query: normalized_query,
                generated_at_epoch_ms: capsule.generated_at_epoch_ms,
                snippets: Vec::new(),
                memory_notes: Vec::new(),
                documents: capsule.documents,
                omissions: capsule.omissions,
                cache_status: capsule.cache_status,
                budget: capsule.budget,
                stale: capsule.stale,
            })
        }
    }
}

pub fn retrieval_truth() -> RetrievalTruth {
    RetrievalTruth {
        modes: vec![
            RetrievalMode::ExactSearch,
            RetrievalMode::Overview,
            RetrievalMode::TaskCapsule,
            RetrievalMode::DiffAware,
        ],
        inclusion_reason_kinds: vec![
            InclusionReasonKind::QueryLineMatch,
            InclusionReasonKind::OverviewDocument,
            InclusionReasonKind::TaskCapsuleMatch,
            InclusionReasonKind::MemoryNoteMatch,
            InclusionReasonKind::DiffChangedFile,
            InclusionReasonKind::DiffRelatedMatch,
        ],
        omission_reason_kinds: vec![
            OmissionReasonKind::NoLineMatches,
            OmissionReasonKind::NoDocumentMatches,
            OmissionReasonKind::NoMemoryMatches,
            OmissionReasonKind::ItemLimitReached,
            OmissionReasonKind::ByteBudgetReached,
            OmissionReasonKind::MissingIndexedFile,
        ],
        limits: RetrievalLimits {
            max_context_items: MAX_CONTEXT_ITEMS,
            max_snippet_pack_bytes: MAX_SNIPPET_PACK_BYTES,
            max_document_pack_bytes: MAX_DOCUMENT_PACK_BYTES,
            max_memory_pack_bytes: MAX_MEMORY_PACK_BYTES,
            max_history_runs: MAX_HISTORY_RUNS,
        },
        durable_memory_enabled: true,
        budget_estimate_enabled: true,
        budget_estimate_unit: "approximate_tokens_from_included_bytes".to_string(),
    }
}

pub fn invalidate_exact_match_cache(root: &Path) -> io::Result<()> {
    clear_exact_match_cache(root)?;
    clear_capsule_cache(root)
}

pub fn assemble_overview(root: &Path, limit: usize) -> io::Result<ContextCapsule> {
    if let Some(capsule) = read_capsule_cache(root, RetrievalMode::Overview, None)? {
        return capsule_with_cache_status(
            capsule_with_budget_and_stale(root, capsule)?,
            CacheStatusKind::Hit,
            "overview cache entry returned this context pack.",
        );
    }

    let capped_limit = limit.clamp(1, MAX_CONTEXT_ITEMS);
    let documents = indexed_documents(root, capped_limit + 1)?;
    let (documents, omissions) = pack_documents(
        documents,
        capped_limit,
        InclusionReasonKind::OverviewDocument,
        "Indexed document supports repository overview.",
        "repository overview",
    );
    let capsule = capsule_with_budget_and_stale(
        root,
        ContextCapsule {
            generated_at_epoch_ms: now_epoch_ms()?,
            documents,
            omissions,
            cache_status: cache_status(
                CacheStatusKind::Miss,
                "overview cache miss; context pack was assembled and cached.",
            ),
            budget: ContextBudgetEstimate::default(),
            stale: ContextStaleStatus::default(),
        },
    )?;
    persist_capsule_cache(root, RetrievalMode::Overview, None, &capsule)?;
    Ok(capsule)
}

pub fn assemble_task_capsule(root: &Path, query: &str, limit: usize) -> io::Result<ContextCapsule> {
    let query = normalize_query(query);
    if let Some(capsule) = read_capsule_cache(root, RetrievalMode::TaskCapsule, Some(&query))? {
        return capsule_with_cache_status(
            capsule_with_budget_and_stale(root, capsule)?,
            CacheStatusKind::Hit,
            "task_capsule cache entry returned this context pack.",
        );
    }

    let capped_limit = limit.clamp(1, MAX_CONTEXT_ITEMS);
    let documents = matching_documents(root, &query, capped_limit + 1)?;
    let reason_detail = format!("Indexed document contents matched query '{query}'.");
    let scope = format!("query '{query}'");
    let (documents, omissions) = pack_documents(
        documents,
        capped_limit,
        InclusionReasonKind::TaskCapsuleMatch,
        &reason_detail,
        &scope,
    );
    let capsule = capsule_with_budget_and_stale(
        root,
        ContextCapsule {
            generated_at_epoch_ms: now_epoch_ms()?,
            documents,
            omissions,
            cache_status: cache_status(
                CacheStatusKind::Miss,
                "task_capsule cache miss; context pack was assembled and cached.",
            ),
            budget: ContextBudgetEstimate::default(),
            stale: ContextStaleStatus::default(),
        },
    )?;
    persist_capsule_cache(root, RetrievalMode::TaskCapsule, Some(&query), &capsule)?;
    Ok(capsule)
}

fn pack_snippets(
    query: &str,
    hits: Vec<repo_index::SearchHit>,
    limit: usize,
) -> (Vec<ContextSnippet>, Vec<OmissionReason>) {
    let total_hits = hits.len();
    let mut snippets = Vec::new();
    let mut omissions = Vec::new();
    let mut remaining_bytes = MAX_SNIPPET_PACK_BYTES;
    let mut byte_budget_hit = false;

    for hit in hits.into_iter().take(limit) {
        let Some((line, truncated)) = fit_within_budget(&hit.line, &mut remaining_bytes) else {
            byte_budget_hit = true;
            break;
        };
        snippets.push(ContextSnippet {
            path: hit.path,
            line_number: hit.line_number,
            line,
            reason: InclusionReason {
                kind: InclusionReasonKind::QueryLineMatch,
                detail: format!("Indexed line matched query '{query}'."),
            },
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if snippets.is_empty() && total_hits == 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoLineMatches,
            detail: format!("No indexed lines matched query '{query}'."),
        });
    }
    if total_hits > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!(
                "Additional matching lines omitted because item limit {limit} was reached."
            ),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!("Packed matching lines reached byte budget {MAX_SNIPPET_PACK_BYTES}."),
        });
    }

    (snippets, omissions)
}

fn pack_documents(
    documents: Vec<repo_index::IndexedDocument>,
    limit: usize,
    reason_kind: InclusionReasonKind,
    reason_detail: &str,
    scope: &str,
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
            path: document.path,
            contents,
            reason: InclusionReason {
                kind: reason_kind.clone(),
                detail: reason_detail.to_string(),
            },
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if packed.is_empty() && total_documents == 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoDocumentMatches,
            detail: format!("No indexed documents matched {scope}."),
        });
    }
    if total_documents > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!(
                "Additional indexed documents omitted because item limit {limit} was reached."
            ),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!(
                "Packed indexed documents reached byte budget {MAX_DOCUMENT_PACK_BYTES}."
            ),
        });
    }

    (packed, omissions)
}

pub(crate) fn fit_within_budget(
    value: &str,
    remaining_bytes: &mut usize,
) -> Option<(String, bool)> {
    if *remaining_bytes == 0 {
        return None;
    }

    if value.len() <= *remaining_bytes {
        *remaining_bytes -= value.len();
        return Some((value.to_string(), false));
    }

    if *remaining_bytes <= TRUNCATED_PACK_MARKER.len() {
        return None;
    }

    let cutoff = *remaining_bytes - TRUNCATED_PACK_MARKER.len();
    let mut boundary = cutoff;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    if boundary == 0 {
        return None;
    }

    let mut truncated = value[..boundary].to_string();
    truncated.push_str(TRUNCATED_PACK_MARKER);
    *remaining_bytes = 0;
    Some((truncated, true))
}

fn assembly_with_budget_and_stale(
    root: &Path,
    mut assembly: ContextAssembly,
) -> io::Result<ContextAssembly> {
    let snippet_bytes = assembly
        .snippets
        .iter()
        .map(|snippet| snippet.line.len())
        .sum::<usize>();
    let memory_bytes = assembly
        .memory_notes
        .iter()
        .map(|note| note.content.len())
        .sum::<usize>();
    assembly.budget = budget_estimate(snippet_bytes + memory_bytes);
    assembly.stale = stale_status(root)?;
    Ok(assembly)
}

fn assembly_with_cache_status(
    mut assembly: ContextAssembly,
    kind: CacheStatusKind,
    detail: &str,
) -> ContextAssembly {
    assembly.cache_status = cache_status(kind, detail);
    assembly
}

fn capsule_with_budget_and_stale(
    root: &Path,
    mut capsule: ContextCapsule,
) -> io::Result<ContextCapsule> {
    let document_bytes = capsule
        .documents
        .iter()
        .map(|document| document.contents.len())
        .sum::<usize>();
    capsule.budget = budget_estimate(document_bytes);
    capsule.stale = stale_status(root)?;
    Ok(capsule)
}

fn capsule_with_cache_status(
    mut capsule: ContextCapsule,
    kind: CacheStatusKind,
    detail: &str,
) -> io::Result<ContextCapsule> {
    capsule.cache_status = cache_status(kind, detail);
    Ok(capsule)
}

pub(crate) fn cache_status(kind: CacheStatusKind, detail: &str) -> CacheStatus {
    CacheStatus {
        kind,
        detail: detail.to_string(),
    }
}

fn budget_estimate(included_bytes: usize) -> ContextBudgetEstimate {
    ContextBudgetEstimate {
        included_bytes,
        approximate_tokens: included_bytes.div_ceil(4),
    }
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
