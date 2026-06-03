use std::fs;

use super::common::temp_repo;
use crate::storage::load_index;
use crate::{
    indexed_documents, matching_documents, repo_map, search_code, sync_repo,
    MAX_INDEXED_CONTENT_BYTES,
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
fn repo_map_reports_bounded_directories_and_rust_symbols() {
    let root = temp_repo();
    fs::create_dir(root.join("src")).expect("src directory should write");
    fs::create_dir(root.join("docs")).expect("docs directory should write");
    fs::write(
        root.join("src").join("lib.rs"),
        "pub mod api;\npub struct Widget;\nfn helper() {}\n",
    )
    .expect("rust file should write");
    fs::write(root.join("docs").join("guide.md"), "operator docs\n")
        .expect("doc file should write");
    fs::write(root.join("README.md"), "readme\n").expect("readme should write");

    sync_repo(&root).expect("sync should succeed");

    let map = repo_map(&root).expect("repo map should load");
    assert_eq!(map.indexed_files, 3);
    assert!(map
        .directories
        .iter()
        .any(|directory| directory.path == "src" && directory.indexed_files == 1));
    assert!(map
        .directories
        .iter()
        .any(|directory| directory.path == "." && directory.indexed_files == 1));
    assert_eq!(map.rust_files.len(), 1);
    assert_eq!(map.rust_files[0].path, "src/lib.rs");
    assert!(map.rust_files[0]
        .symbols
        .iter()
        .any(|symbol| symbol.contains("struct Widget")));
    assert_eq!(map.omitted_directory_count, 0);
    assert_eq!(map.omitted_rust_file_count, 0);
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
