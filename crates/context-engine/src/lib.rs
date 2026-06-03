use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use repo_index::{indexed_documents, matching_documents, repo_inventory, search_code};
mod memory;
mod models;
mod repositories;
mod storage;

use memory::{find_memory_notes, pack_memory_notes};
pub use memory::{
    memory_delete, memory_export, memory_import, memory_read, memory_search, memory_update,
    memory_write,
};
pub use models::*;
pub use repositories::{
    assemble_context_for_registered_repositories, list_registered_repositories,
    list_workspace_profiles, register_repository, registered_repository_detail,
    registered_repository_state, remove_registered_repository, save_workspace_profile,
    update_registered_repository_metadata,
};
pub(crate) use storage::*;

impl EngineInfo {
    pub fn quotarelay() -> Self {
        Self {
            name: "quotarelay",
            mode: "bootstrap",
        }
    }

    pub fn banner(&self) -> String {
        format!("{} {}", self.name, self.mode)
    }
}

fn normalize_query(query: &str) -> String {
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
        append_history(root, &assembly)?;
        return Ok(assembly);
    }

    let hits = search_code(root, &query, capped_limit + 1)?;
    let (snippets, omissions) = pack_snippets(&query, hits, capped_limit);
    let (memory_notes, memory_omissions) =
        pack_memory_notes(&query, find_memory_notes(root, &query)?, capped_limit);
    let mut omissions = omissions;
    omissions.extend(memory_omissions);
    let assembly = ContextAssembly {
        query: query.clone(),
        generated_at_epoch_ms: now_epoch_ms()?,
        snippets,
        memory_notes,
        omissions,
    };

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
        ],
        inclusion_reason_kinds: vec![
            InclusionReasonKind::QueryLineMatch,
            InclusionReasonKind::OverviewDocument,
            InclusionReasonKind::TaskCapsuleMatch,
            InclusionReasonKind::MemoryNoteMatch,
        ],
        omission_reason_kinds: vec![
            OmissionReasonKind::NoLineMatches,
            OmissionReasonKind::NoDocumentMatches,
            OmissionReasonKind::NoMemoryMatches,
            OmissionReasonKind::ItemLimitReached,
            OmissionReasonKind::ByteBudgetReached,
        ],
        limits: RetrievalLimits {
            max_context_items: MAX_CONTEXT_ITEMS,
            max_snippet_pack_bytes: MAX_SNIPPET_PACK_BYTES,
            max_document_pack_bytes: MAX_DOCUMENT_PACK_BYTES,
            max_memory_pack_bytes: MAX_MEMORY_PACK_BYTES,
            max_history_runs: MAX_HISTORY_RUNS,
        },
        durable_memory_enabled: true,
    }
}

pub fn invalidate_exact_match_cache(root: &Path) -> io::Result<()> {
    clear_exact_match_cache(root)?;
    clear_capsule_cache(root)
}

pub fn assemble_overview(root: &Path, limit: usize) -> io::Result<ContextCapsule> {
    if let Some(capsule) = read_capsule_cache(root, RetrievalMode::Overview, None)? {
        return Ok(capsule);
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
    let capsule = ContextCapsule {
        generated_at_epoch_ms: now_epoch_ms()?,
        documents,
        omissions,
    };
    persist_capsule_cache(root, RetrievalMode::Overview, None, &capsule)?;
    Ok(capsule)
}

pub fn assemble_task_capsule(root: &Path, query: &str, limit: usize) -> io::Result<ContextCapsule> {
    let query = normalize_query(query);
    if let Some(capsule) = read_capsule_cache(root, RetrievalMode::TaskCapsule, Some(&query))? {
        return Ok(capsule);
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
    let capsule = ContextCapsule {
        generated_at_epoch_ms: now_epoch_ms()?,
        documents,
        omissions,
    };
    persist_capsule_cache(root, RetrievalMode::TaskCapsule, Some(&query), &capsule)?;
    Ok(capsule)
}

pub fn inspect_local_state(root: &Path) -> io::Result<LocalStateInspection> {
    let index = match repo_inventory(root) {
        Ok(inventory) => LocalStateArtifact {
            present: true,
            item_count: inventory.indexed_files,
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => LocalStateArtifact {
            present: false,
            item_count: 0,
        },
        Err(error) => return Err(error),
    };
    let memory_notes = load_memory_notes(root)?;
    let history = load_history(root)?;
    let registered_repositories = load_registered_repositories(root)?;
    let exact_cache = load_exact_match_cache(root)?;
    let capsule_cache = load_capsule_cache(root)?;

    Ok(LocalStateInspection {
        index,
        memory_notes: LocalStateArtifact {
            present: memory_notes_path(root).exists(),
            item_count: memory_notes.notes.len(),
        },
        context_run_history: LocalStateArtifact {
            present: history_path(root).exists(),
            item_count: history.runs.len(),
        },
        registered_repositories: LocalStateArtifact {
            present: registered_repositories_path(root).exists(),
            item_count: registered_repositories.repositories.len(),
        },
        exact_search_cache: LocalStateArtifact {
            present: exact_match_cache_path(root).exists(),
            item_count: exact_cache.entries.len(),
        },
        retrieval_capsule_cache: LocalStateArtifact {
            present: capsule_cache_path(root).exists(),
            item_count: capsule_cache.entries.len(),
        },
    })
}

pub fn inspect_retrieval_caches(root: &Path) -> io::Result<CacheInspection> {
    let exact_cache = load_exact_match_cache(root)?;
    let capsule_cache = load_capsule_cache(root)?;

    Ok(CacheInspection {
        exact_search_cache: LocalStateArtifact {
            present: exact_match_cache_path(root).exists(),
            item_count: exact_cache.entries.len(),
        },
        retrieval_capsule_cache: LocalStateArtifact {
            present: capsule_cache_path(root).exists(),
            item_count: capsule_cache.entries.len(),
        },
    })
}

pub fn clear_retrieval_caches(root: &Path) -> io::Result<CacheClearResult> {
    let exact_search_cache_cleared = exact_match_cache_path(root).exists();
    let retrieval_capsule_cache_cleared = capsule_cache_path(root).exists();

    clear_exact_match_cache(root)?;
    clear_capsule_cache(root)?;

    Ok(CacheClearResult {
        exact_search_cache_cleared,
        retrieval_capsule_cache_cleared,
    })
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

fn fit_within_budget(value: &str, remaining_bytes: &mut usize) -> Option<(String, bool)> {
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

fn normalized_repo_root(repo_root: &Path) -> io::Result<String> {
    let canonical = std::fs::canonicalize(repo_root)?;
    Ok(canonical.to_string_lossy().into_owned())
}

fn repo_name(repo_root: &Path) -> String {
    repo_root
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| repo_root.to_string_lossy().into_owned())
}

fn now_epoch_ms() -> io::Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_millis())
}

fn unique_epoch_nanos() -> io::Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_nanos())
}

#[cfg(test)]
mod tests;
