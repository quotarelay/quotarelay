use std::fs;

use super::common::temp_repo;
use crate::storage::load_index;
use crate::{
    indexed_documents, matching_documents, search_code, sync_repo, MAX_INDEXED_CONTENT_BYTES,
};

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

    let matches = matching_documents(&root, "needle", 5).expect("matching documents should load");
    assert_eq!(matches.len(), 5);
}
