use std::fs;
use std::io::ErrorKind;

use super::common::temp_repo;
use crate::*;

#[test]
fn corrupt_local_json_state_returns_recoverable_errors() {
    let root = temp_repo();
    let state_dir = root.join(".quotarelay");
    fs::create_dir_all(&state_dir).expect("state dir should create");

    for file_name in [
        "memory_notes.json",
        "registered_repositories.json",
        "context_runs.json",
        "exact_match_cache.json",
    ] {
        fs::write(state_dir.join(file_name), "{ not json").expect("corrupt state should write");
    }

    for error in [
        memory_search(&root, "needle", 3).expect_err("corrupt memory should fail"),
        list_registered_repositories(&root, 10).expect_err("corrupt registration should fail"),
        context_run_history(&root, 3).expect_err("corrupt history should fail"),
        retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 3)
            .expect_err("corrupt exact cache should fail"),
    ] {
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        let message = error.to_string();
        assert!(message.contains("corrupt local state file"));
        assert!(message.contains("repair or remove the file to recover"));
    }
}

#[test]
fn inspect_local_state_reports_bounded_presence_and_counts() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");
    memory_write(
        &root,
        "Needle note",
        "needle memory",
        &["memory".to_string()],
    )
    .expect("memory write should succeed");
    register_repository(&root, &root).expect("repository registration should succeed");
    assemble_context(&root, "needle", 2).expect("exact assembly should succeed");
    assemble_overview(&root, 2).expect("overview assembly should succeed");

    let state = inspect_local_state(&root).expect("local state inspection should succeed");

    assert_eq!(state.index.present, true);
    assert_eq!(state.index.item_count, 1);
    assert_eq!(state.memory_notes.present, true);
    assert_eq!(state.memory_notes.item_count, 1);
    assert_eq!(state.context_run_history.present, true);
    assert_eq!(state.context_run_history.item_count, 1);
    assert_eq!(state.registered_repositories.present, true);
    assert_eq!(state.registered_repositories.item_count, 1);
    assert_eq!(state.exact_search_cache.present, true);
    assert_eq!(state.exact_search_cache.item_count, 1);
    assert_eq!(state.retrieval_capsule_cache.present, true);
    assert_eq!(state.retrieval_capsule_cache.item_count, 1);
}
