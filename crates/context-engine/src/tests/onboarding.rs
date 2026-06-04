use std::fs;

use super::common::temp_repo;
use crate::*;

#[test]
fn onboarding_pack_reports_bounded_local_orientation() {
    let root = temp_repo();
    fs::create_dir_all(root.join("src")).expect("src dir should write");
    fs::write(root.join("src/lib.rs"), "pub fn onboard_target() {}\n")
        .expect("repo file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let pack = onboarding_pack(&root, &["crates/context-engine/src/lib.rs".to_string()])
        .expect("onboarding pack should build");

    assert_eq!(pack.indexed_files, 1);
    assert!(pack.sample_directories.iter().any(|path| path == "src"));
    assert!(pack
        .rust_entrypoints
        .iter()
        .any(|entry| entry.contains("onboard_target")));
    assert!(pack
        .recommended_validation
        .iter()
        .any(|command| command == "cargo test -p context-engine"));
    assert!(pack.handoff_templates.iter().any(|name| name == "review"));
    assert!(pack
        .privacy_notes
        .iter()
        .any(|note| note.contains("does not upload source code")));
}
