use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use repo_index::repo_inventory;
mod memory;
mod models;
mod repositories;
mod retrieval;
mod storage;

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
pub use retrieval::{
    assemble_context, assemble_overview, assemble_task_capsule, context_run_detail,
    context_run_history, invalidate_exact_match_cache, retrieval_truth, retrieve_context,
};
pub(crate) use retrieval::{fit_within_budget, normalize_query};
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
