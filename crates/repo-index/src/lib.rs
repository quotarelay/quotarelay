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
pub use models::{IndexFreshness, IndexedDocument, RepoInventory, SearchHit, SyncResult};
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

pub fn index_freshness(root: &Path) -> io::Result<IndexFreshness> {
    let stored = load_index(root)?;
    let previous_files = stored
        .files
        .into_iter()
        .map(|file| (file.path.clone(), file))
        .collect::<HashMap<_, _>>();
    let current_files = collect_indexed_files(root, &previous_files)?;
    let current_paths = current_files
        .iter()
        .map(|file| (file.path.clone(), file.modified_at_epoch_ms))
        .collect::<HashMap<_, _>>();

    let mut changed_count = 0;
    let mut missing_count = 0;
    for (path, indexed) in &previous_files {
        match current_paths.get(path) {
            Some(modified_at_epoch_ms) if *modified_at_epoch_ms != indexed.modified_at_epoch_ms => {
                changed_count += 1;
            }
            Some(_) => {}
            None => missing_count += 1,
        }
    }

    let new_count = current_paths
        .keys()
        .filter(|path| !previous_files.contains_key(*path))
        .count();

    Ok(IndexFreshness {
        is_stale: changed_count > 0 || missing_count > 0 || new_count > 0,
        changed_count,
        missing_count,
        new_count,
    })
}

#[cfg(test)]
mod tests;
