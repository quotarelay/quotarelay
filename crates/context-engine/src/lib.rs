use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod budget;
mod clarification;
mod diff;
mod feedback;
mod handoff;
mod memory;
mod models;
mod repositories;
mod retrieval;
mod storage;
mod validation;

pub use diff::assemble_diff_aware;
pub use feedback::{context_feedback_list, context_feedback_write};
pub use handoff::assemble_handoff_packet;
pub use memory::{
    memory_delete, memory_export, memory_import, memory_read, memory_search, memory_update,
    memory_update_with_profile, memory_write, memory_write_with_profile,
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
pub use storage::{clear_retrieval_caches, inspect_local_state, inspect_retrieval_caches};
pub use validation::recommend_validation;

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
