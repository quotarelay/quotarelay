use std::collections::HashMap;
use std::io;
use std::path::Path;

mod indexing;
mod models;
mod rust_capsule;
mod storage;

use indexing::{collect_indexed_files, now_epoch_ms};
pub(crate) use models::*;
pub use models::{IndexedDocument, RepoInventory, SearchHit, SyncResult};
use rust_capsule::{contains_symbol_match, is_symbol_line_match};
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

pub fn search_code(root: &Path, query: &str, limit: usize) -> io::Result<Vec<SearchHit>> {
    let index = load_index(root)?;
    let normalized_query = query.to_ascii_lowercase();
    let capped_limit = limit.clamp(1, 10);
    let mut symbol_hits = Vec::new();
    let mut text_hits = Vec::new();

    for file in index.files {
        for (offset, line) in file.contents.lines().enumerate() {
            if line.to_ascii_lowercase().contains(&normalized_query) {
                let hit = SearchHit {
                    path: file.path.clone(),
                    line_number: offset + 1,
                    line: line.to_string(),
                };
                if is_symbol_line_match(line, query) {
                    symbol_hits.push(hit);
                } else {
                    text_hits.push(hit);
                }
                if symbol_hits.len() + text_hits.len() >= capped_limit {
                    break;
                }
            }
        }
        if symbol_hits.len() + text_hits.len() >= capped_limit {
            break;
        }
    }

    symbol_hits.extend(text_hits);
    symbol_hits.truncate(capped_limit);
    Ok(symbol_hits)
}

pub fn repo_inventory(root: &Path) -> io::Result<RepoInventory> {
    let index = load_index(root)?;
    let sample_paths = index
        .files
        .iter()
        .take(5)
        .map(|file| file.path.clone())
        .collect();

    Ok(RepoInventory {
        indexed_at_epoch_ms: index.indexed_at_epoch_ms,
        indexed_files: index.files.len(),
        sample_paths,
    })
}

pub fn indexed_documents(root: &Path, limit: usize) -> io::Result<Vec<IndexedDocument>> {
    let capped_limit = limit.clamp(1, 10);
    let index = load_index(root)?;

    Ok(index
        .files
        .into_iter()
        .take(capped_limit)
        .map(|file| IndexedDocument {
            path: file.path,
            contents: file.contents,
        })
        .collect())
}

pub fn matching_documents(
    root: &Path,
    query: &str,
    limit: usize,
) -> io::Result<Vec<IndexedDocument>> {
    let capped_limit = limit.clamp(1, 10);
    let normalized_query = query.to_ascii_lowercase();
    let index = load_index(root)?;

    let mut symbol_documents = Vec::new();
    let mut text_documents = Vec::new();

    for file in index.files {
        if !file
            .contents
            .to_ascii_lowercase()
            .contains(&normalized_query)
        {
            continue;
        }

        let document = IndexedDocument {
            path: file.path,
            contents: file.contents,
        };

        if contains_symbol_match(&document.contents, query) {
            symbol_documents.push(document);
        } else {
            text_documents.push(document);
        }

        if symbol_documents.len() + text_documents.len() >= capped_limit {
            break;
        }
    }

    symbol_documents.extend(text_documents);
    symbol_documents.truncate(capped_limit);
    Ok(symbol_documents)
}

#[cfg(test)]
mod tests {
    use super::{
        indexed_documents, load_index, matching_documents, repo_inventory, search_code, sync_repo,
        MAX_INDEXED_CONTENT_BYTES, TRUNCATED_INDEX_MARKER,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sync_persists_and_search_returns_bounded_hits() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle one\nneedle two\n")
            .expect("alpha file should write");
        fs::write(root.join("beta.txt"), "needle three\n").expect("beta file should write");

        let sync = sync_repo(&root).expect("sync should succeed");
        assert_eq!(sync.indexed_files, 2);
        assert!(root.join(".quotarelay").join("index.json").exists());

        let hits = search_code(&root, "needle", 2).expect("search should succeed");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].path, "alpha.txt");
        assert_eq!(hits[0].line_number, 1);

        let inventory = repo_inventory(&root).expect("inventory should succeed");
        assert_eq!(inventory.indexed_files, 2);
        assert_eq!(inventory.sample_paths, vec!["alpha.txt", "beta.txt"]);
        assert!(inventory.indexed_at_epoch_ms > 0);
    }

    #[test]
    fn sync_rust_files_into_structural_capsules_with_bounded_output() {
        let root = temp_repo();
        let large_body = "println!(\"padding\");\n".repeat(900);
        fs::write(
            root.join("lib.rs"),
            format!(
                "use std::fmt;\n\nstruct Widget {{\n    id: usize,\n}}\n\nfn plan(input: &str) -> usize {{\n    let body_only_term = input.len();\n    {large_body}    body_only_term\n}}\n"
            ),
        )
        .expect("rust file should write");

        sync_repo(&root).expect("sync should succeed");

        let index = load_index(&root).expect("index should load");
        assert_eq!(index.files.len(), 1);
        assert!(index.files[0].contents.contains("use std::fmt;"));
        assert!(index.files[0]
            .contents
            .contains("struct Widget { id: usize }"));
        assert!(index.files[0]
            .contents
            .contains("fn plan(input: &str) -> usize;"));
        assert!(!index.files[0]
            .contents
            .contains("body_only_term = input.len()"));
        assert!(index.files[0].contents.len() <= MAX_INDEXED_CONTENT_BYTES);
        assert!(
            index.files[0].contents.len() < MAX_INDEXED_CONTENT_BYTES
                || index.files[0].contents.ends_with(TRUNCATED_INDEX_MARKER)
        );
    }

    #[test]
    fn malformed_rust_files_fall_back_to_raw_text_search() {
        let root = temp_repo();
        fs::write(
            root.join("broken.rs"),
            "fn broken( {\n    let raw_fallback_term = 1;\n}\n",
        )
        .expect("broken rust file should write");

        sync_repo(&root).expect("sync should succeed");

        let hits = search_code(&root, "raw_fallback_term", 2).expect("search should succeed");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "broken.rs");
        assert_eq!(hits[0].line_number, 2);
    }

    #[test]
    fn document_queries_return_bounded_indexed_contents() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "alpha term\n").expect("alpha file should write");
        fs::write(root.join("beta.txt"), "beta term\n").expect("beta file should write");

        sync_repo(&root).expect("sync should succeed");

        let docs = indexed_documents(&root, 1).expect("documents should load");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].path, "alpha.txt");

        let matches = matching_documents(&root, "beta", 2).expect("matching documents should load");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "beta.txt");
        assert_eq!(matches[0].contents, "beta term\n");
    }

    #[test]
    fn sync_repo_updates_incrementally_for_add_change_delete() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "alpha one\n").expect("alpha file should write");
        fs::write(root.join("beta.txt"), "beta one\n").expect("beta file should write");

        sync_repo(&root).expect("initial sync should succeed");
        let first_index = load_index(&root).expect("first index should load");
        let first_beta = first_index
            .files
            .iter()
            .find(|file| file.path == "beta.txt")
            .expect("beta should exist")
            .modified_at_epoch_ms;

        thread::sleep(Duration::from_millis(20));
        fs::remove_file(root.join("alpha.txt")).expect("alpha should delete");
        fs::write(root.join("beta.txt"), "beta two\n").expect("beta should update");
        fs::write(root.join("gamma.txt"), "gamma one\n").expect("gamma should write");

        sync_repo(&root).expect("incremental sync should succeed");
        let second_index = load_index(&root).expect("second index should load");

        assert_eq!(second_index.files.len(), 2);
        assert!(second_index
            .files
            .iter()
            .all(|file| file.path != "alpha.txt"));
        assert!(second_index
            .files
            .iter()
            .any(|file| file.path == "gamma.txt"));
        let second_beta = second_index
            .files
            .iter()
            .find(|file| file.path == "beta.txt")
            .expect("beta should still exist");
        assert_eq!(second_beta.contents, "beta two\n");
        assert!(second_beta.modified_at_epoch_ms >= first_beta);
    }

    #[test]
    fn sync_repo_respects_explicit_local_ignore_config_after_sync() {
        let root = temp_repo();
        let generated_dir = root.join("generated");
        fs::create_dir_all(&generated_dir).expect("generated dir should create");
        fs::write(root.join("alpha.txt"), "alpha keep\n").expect("alpha file should write");
        fs::write(root.join("secret.txt"), "secret needle\n").expect("secret file should write");
        fs::write(generated_dir.join("artifact.txt"), "artifact needle\n")
            .expect("artifact file should write");

        sync_repo(&root).expect("initial sync should succeed");
        assert_eq!(
            search_code(&root, "needle", 10)
                .expect("initial search should succeed")
                .len(),
            2
        );

        let state_dir = root.join(".quotarelay");
        fs::create_dir_all(&state_dir).expect("state dir should create");
        fs::write(
            state_dir.join("ignore.json"),
            r#"{
  "paths": ["secret.txt"],
  "prefixes": ["generated"]
}"#,
        )
        .expect("ignore config should write");

        assert_eq!(
            search_code(&root, "needle", 10)
                .expect("search before explicit resync should still use old index")
                .len(),
            2
        );

        let resync = sync_repo(&root).expect("resync should honor ignore config");
        assert_eq!(resync.indexed_files, 1);

        let index = load_index(&root).expect("index should load");
        assert_eq!(index.files.len(), 1);
        assert_eq!(index.files[0].path, "alpha.txt");
        assert!(search_code(&root, "needle", 10)
            .expect("ignored search should succeed")
            .is_empty());
    }

    #[test]
    fn sync_repo_normalizes_windows_style_ignore_entries() {
        let root = temp_repo();
        let generated_dir = root.join("generated");
        fs::create_dir_all(&generated_dir).expect("generated dir should create");
        fs::write(generated_dir.join("artifact.txt"), "artifact needle\n")
            .expect("artifact file should write");
        fs::write(root.join("keep.txt"), "keep needle\n").expect("keep file should write");
        let state_dir = root.join(".quotarelay");
        fs::create_dir_all(&state_dir).expect("state dir should create");
        fs::write(
            state_dir.join("ignore.json"),
            r#"{
  "paths": ["generated\\artifact.txt"]
}"#,
        )
        .expect("ignore config should write");

        let sync = sync_repo(&root).expect("sync should honor normalized ignore path");
        assert_eq!(sync.indexed_files, 1);
        let hits = search_code(&root, "needle", 10).expect("search should succeed");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "keep.txt");
    }

    #[test]
    fn many_file_repos_keep_index_and_query_outputs_bounded() {
        let root = temp_repo();
        let large_tail = "padding\n".repeat(2_000);
        for index in 0..40 {
            fs::write(
                root.join(format!("file-{index:02}.txt")),
                format!("needle line {index}\n{large_tail}"),
            )
            .expect("repo file should write");
        }

        let sync = sync_repo(&root).expect("sync should succeed");
        assert_eq!(sync.indexed_files, 40);

        let stored = load_index(&root).expect("index should load");
        assert_eq!(stored.files.len(), 40);
        assert!(stored
            .files
            .iter()
            .all(|file| file.contents.len() <= MAX_INDEXED_CONTENT_BYTES));

        let hits = search_code(&root, "needle", 3).expect("search should succeed");
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].path, "file-00.txt");

        let documents = indexed_documents(&root, 4).expect("documents should load");
        assert_eq!(documents.len(), 4);
        assert!(documents
            .iter()
            .all(|document| document.contents.len() <= MAX_INDEXED_CONTENT_BYTES));

        let matches =
            matching_documents(&root, "needle", 5).expect("matching documents should load");
        assert_eq!(matches.len(), 5);
    }

    #[test]
    fn search_and_document_matching_prefer_symbol_hits() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "Widget appears in notes only\n")
            .expect("alpha file should write");
        fs::write(root.join("lib.rs"), "struct Widget {\n    id: usize,\n}\n")
            .expect("rust file should write");

        sync_repo(&root).expect("sync should succeed");

        let hits = search_code(&root, "Widget", 2).expect("search should succeed");
        assert_eq!(hits[0].path, "lib.rs");

        let documents =
            matching_documents(&root, "Widget", 2).expect("matching documents should succeed");
        assert_eq!(documents[0].path, "lib.rs");
    }

    fn temp_repo() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("quotarelay-index-{unique}"));
        fs::create_dir_all(&root).expect("temp repo should create");
        root
    }
}
