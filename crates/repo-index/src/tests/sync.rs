use std::fs;
use std::thread;
use std::time::Duration;

use super::common::temp_repo;
use crate::storage::load_index;
use crate::{repo_inventory, search_code, sync_repo};

#[test]
fn sync_persists_and_search_returns_bounded_hits() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle one\nneedle two\n").expect("alpha file should write");
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
