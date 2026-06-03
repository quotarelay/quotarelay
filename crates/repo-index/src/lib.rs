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
pub use models::{
    ChangedDocument, ChangedDocumentStatus, IndexFreshness, IndexedDocument, RepoInventory,
    RepoMap, RepoMapDirectory, RepoMapRustFile, SearchHit, SyncResult,
};
use rust_capsule::{contains_symbol_match, is_symbol_line_match};
pub use search::{indexed_documents, matching_documents, repo_inventory, repo_map, search_code};
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

pub fn changed_documents(root: &Path, limit: usize) -> io::Result<Vec<ChangedDocument>> {
    let stored = load_index(root)?;
    let previous_files = stored
        .files
        .into_iter()
        .map(|file| (file.path.clone(), file))
        .collect::<HashMap<_, _>>();
    let current_files = collect_indexed_files(root, &previous_files)?;
    let mut changed = current_files
        .into_iter()
        .filter_map(|file| match previous_files.get(&file.path) {
            Some(indexed) if indexed.modified_at_epoch_ms != file.modified_at_epoch_ms => {
                Some(ChangedDocument {
                    path: file.path,
                    contents: file.contents,
                    status: ChangedDocumentStatus::Changed,
                })
            }
            None => Some(ChangedDocument {
                path: file.path,
                contents: file.contents,
                status: ChangedDocumentStatus::New,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    changed.sort_by(|left, right| left.path.cmp(&right.path));
    changed.truncate(limit);
    Ok(changed)
}

#[cfg(test)]
mod tests;
