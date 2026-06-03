use std::fs;

use super::common::temp_repo;
use crate::storage::load_index;
use crate::{search_code, sync_repo};

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
