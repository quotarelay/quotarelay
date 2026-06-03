use std::collections::HashMap;
use std::io;
use std::path::Path;

mod indexing;
mod models;
mod rust_capsule;
mod search;
mod storage;

use indexing::{collect_indexed_files, now_epoch_ms};
pub(crate) use models::*;
pub use models::{IndexedDocument, RepoInventory, SearchHit, SyncResult};
use rust_capsule::{contains_symbol_match, is_symbol_line_match};
pub use search::{indexed_documents, matching_documents, repo_inventory, search_code};
use storage::{load_index, persist_index};

pub fn sync_repo(root: &Path) -> io::Result<SyncResult> {
    let previous_files = load_index(root)
        .map(|index| {
            index
                .files
                .into_iter()
                .map(|file| (file.path.clone(), file))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let files = collect_indexed_files(root, &previous_files)?;
    let stored = StoredIndex {
        indexed_at_epoch_ms: now_epoch_ms()?,
        files,
    };
    let index_path = persist_index(root, &stored)?;

    Ok(SyncResult {
        indexed_files: stored.files.len(),
        index_path: index_path.display().to_string(),
    })
}

#[cfg(test)]
mod tests;
