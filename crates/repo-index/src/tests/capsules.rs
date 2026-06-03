use std::fs;

use super::common::temp_repo;
use crate::storage::load_index;
use crate::{
    matching_documents, search_code, sync_repo, MAX_INDEXED_CONTENT_BYTES, TRUNCATED_INDEX_MARKER,
};

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
