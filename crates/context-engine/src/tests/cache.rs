use std::fs;
use std::io::ErrorKind;

use super::common::temp_repo;
use crate::*;

#[test]
fn exact_search_cache_persists_and_reuses_previous_payload() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let first = assemble_context(&root, "needle", 2).expect("first assembly should succeed");
    fs::write(
        root.join("alpha.txt"),
        "changed contents without the query\n",
    )
    .expect("repo file should rewrite");

    let second = assemble_context(&root, "needle", 2).expect("second assembly should succeed");

    assert_eq!(second.query, first.query);
    assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
    assert_eq!(second.snippets.len(), first.snippets.len());
    assert_eq!(second.snippets[0].path, first.snippets[0].path);
    assert_eq!(
        second.snippets[0].line_number,
        first.snippets[0].line_number
    );
    assert_eq!(second.snippets[0].line, first.snippets[0].line);
    assert_eq!(
        second.snippets[0].reason.kind,
        first.snippets[0].reason.kind
    );
    assert_eq!(
        second.snippets[0].reason.detail,
        first.snippets[0].reason.detail
    );
    assert_eq!(second.memory_notes.len(), first.memory_notes.len());
    assert_eq!(second.omissions, first.omissions);
    assert!(root
        .join(".quotarelay")
        .join("exact_match_cache.json")
        .exists());
}

#[test]
fn exact_search_cache_misses_for_different_queries() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let cached = assemble_context(&root, "needle", 2).expect("needle assembly should succeed");
    let miss = assemble_context(&root, "missing", 2).expect("missing assembly should succeed");

    assert_eq!(cached.snippets.len(), 1);
    assert!(miss.snippets.is_empty());
    assert!(miss
        .omissions
        .iter()
        .any(|item| item.kind == OmissionReasonKind::NoLineMatches));
}

#[test]
fn exact_search_cache_canonicalizes_equivalent_queries() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let first = assemble_context(&root, "Needle", 2).expect("first assembly should succeed");
    fs::write(
        root.join("alpha.txt"),
        "changed contents without the query\n",
    )
    .expect("repo file should rewrite");

    let second = assemble_context(&root, "  needle  ", 2).expect("second assembly should succeed");

    assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
    assert_eq!(second.query, first.query);
    assert_eq!(second.snippets[0].path, first.snippets[0].path);
    assert_eq!(second.snippets[0].line, first.snippets[0].line);
}

#[test]
fn task_capsule_cache_canonicalizes_equivalent_queries() {
    let root = temp_repo();
    fs::write(root.join("lib.rs"), "struct Widget {\n    id: usize,\n}\n")
        .expect("rust file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let first =
        assemble_task_capsule(&root, "Widget", 2).expect("first task capsule should succeed");
    fs::write(root.join("lib.rs"), "struct Widget {\n    id: u32,\n}\n")
        .expect("rust file should rewrite");

    let second =
        assemble_task_capsule(&root, " widget ", 2).expect("second task capsule should succeed");

    assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
    assert_eq!(second.documents[0].path, first.documents[0].path);
    assert_eq!(second.documents[0].contents, first.documents[0].contents);
}

#[test]
fn exact_search_cache_is_cleared_on_sync_invalidation() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let cached = assemble_context(&root, "needle", 2).expect("initial assembly should succeed");
    fs::write(root.join("alpha.txt"), "fresh needle after sync\n")
        .expect("repo file should rewrite");
    repo_index::sync_repo(&root).expect("resync should succeed");
    invalidate_exact_match_cache(&root).expect("cache invalidation should succeed");

    let refreshed =
        assemble_context(&root, "needle", 2).expect("refreshed assembly should succeed");

    assert_ne!(
        refreshed.generated_at_epoch_ms,
        cached.generated_at_epoch_ms
    );
    assert_ne!(refreshed.snippets[0].line, cached.snippets[0].line);
    assert_eq!(refreshed.snippets[0].line, "fresh needle after sync");
}

#[test]
fn overview_cache_persists_and_reuses_previous_payload() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let first = assemble_overview(&root, 2).expect("first overview should succeed");
    fs::write(root.join("alpha.txt"), "changed overview without sync\n")
        .expect("alpha file should rewrite");

    let second = assemble_overview(&root, 2).expect("second overview should succeed");

    assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
    assert_eq!(second.documents.len(), first.documents.len());
    assert_eq!(second.documents[0].path, first.documents[0].path);
    assert_eq!(second.documents[0].contents, first.documents[0].contents);
    assert!(root
        .join(".quotarelay")
        .join("retrieval_capsules.json")
        .exists());
}

#[test]
fn task_capsule_cache_persists_and_reuses_previous_payload() {
    let root = temp_repo();
    fs::write(root.join("lib.rs"), "struct Widget {\n    id: usize,\n}\n")
        .expect("rust file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let first =
        assemble_task_capsule(&root, "Widget", 2).expect("first task capsule should succeed");
    fs::write(root.join("lib.rs"), "struct Widget {\n    id: u32,\n}\n")
        .expect("rust file should rewrite");

    let second =
        assemble_task_capsule(&root, "Widget", 2).expect("second task capsule should succeed");

    assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
    assert_eq!(second.documents.len(), first.documents.len());
    assert_eq!(second.documents[0].path, first.documents[0].path);
    assert_eq!(second.documents[0].contents, first.documents[0].contents);
}

#[test]
fn capsule_caches_are_cleared_on_sync_invalidation() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
    fs::write(root.join("lib.rs"), "struct Widget {\n    id: usize,\n}\n")
        .expect("rust file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let cached_overview = assemble_overview(&root, 2).expect("cached overview should succeed");
    let cached_task =
        assemble_task_capsule(&root, "Widget", 2).expect("cached task capsule should succeed");

    fs::write(root.join("alpha.txt"), "fresh overview after sync\n")
        .expect("alpha file should rewrite");
    fs::write(root.join("lib.rs"), "struct Widget {\n    id: u32,\n}\n")
        .expect("rust file should rewrite");
    repo_index::sync_repo(&root).expect("resync should succeed");
    invalidate_exact_match_cache(&root).expect("cache invalidation should succeed");

    let refreshed_overview =
        assemble_overview(&root, 2).expect("refreshed overview should succeed");
    let refreshed_task =
        assemble_task_capsule(&root, "Widget", 2).expect("refreshed task capsule should succeed");

    assert_ne!(
        refreshed_overview.generated_at_epoch_ms,
        cached_overview.generated_at_epoch_ms
    );
    assert_ne!(
        refreshed_overview.documents[0].contents,
        cached_overview.documents[0].contents
    );
    assert_eq!(
        refreshed_overview.documents[0].contents,
        "fresh overview after sync\n"
    );
    assert_ne!(
        refreshed_task.generated_at_epoch_ms,
        cached_task.generated_at_epoch_ms
    );
    assert_ne!(
        refreshed_task.documents[0].contents,
        cached_task.documents[0].contents
    );
    assert!(refreshed_task.documents[0].contents.contains("id: u32"));
}

#[test]
fn cache_inspect_and_clear_missing_state_are_explicit() {
    let root = temp_repo();

    let inspected = inspect_retrieval_caches(&root).expect("cache inspection should succeed");
    assert_eq!(inspected.exact_search_cache.present, false);
    assert_eq!(inspected.exact_search_cache.item_count, 0);
    assert_eq!(inspected.retrieval_capsule_cache.present, false);
    assert_eq!(inspected.retrieval_capsule_cache.item_count, 0);

    let cleared = clear_retrieval_caches(&root).expect("cache clear should succeed");
    assert_eq!(cleared.exact_search_cache_cleared, false);
    assert_eq!(cleared.retrieval_capsule_cache_cleared, false);
    assert_eq!(cleared.status.kind, CacheStatusKind::Empty);

    let after = inspect_retrieval_caches(&root).expect("cache inspection should still succeed");
    assert_eq!(after.exact_search_cache.present, false);
    assert_eq!(after.exact_search_cache.item_count, 0);
    assert_eq!(after.retrieval_capsule_cache.present, false);
    assert_eq!(after.retrieval_capsule_cache.item_count, 0);
}

#[test]
fn corrupt_cache_files_return_recoverable_errors() {
    for file_name in ["exact_match_cache.json", "retrieval_capsules.json"] {
        let root = temp_repo();
        let state_dir = root.join(".quotarelay");
        fs::create_dir_all(&state_dir).expect("state dir should create");
        fs::write(state_dir.join(file_name), "{ not json").expect("corrupt cache should write");

        let error = inspect_retrieval_caches(&root).expect_err("corrupt cache should fail");
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        let message = error.to_string();
        assert!(message.contains("corrupt local state file"));
        assert!(message.contains(file_name));
        assert!(message.contains("repair or remove the file to recover"));
    }
}

#[test]
fn overview_and_task_capsule_cache_entries_are_separate() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
    fs::write(root.join("lib.rs"), "struct Widget {\n    id: usize,\n}\n")
        .expect("rust file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    assemble_overview(&root, 2).expect("overview should populate cache");
    assemble_task_capsule(&root, "Widget", 2).expect("task capsule should populate cache");

    let inspected = inspect_retrieval_caches(&root).expect("cache inspection should succeed");
    assert_eq!(inspected.exact_search_cache.present, false);
    assert_eq!(inspected.exact_search_cache.item_count, 0);
    assert_eq!(inspected.retrieval_capsule_cache.present, true);
    assert_eq!(inspected.retrieval_capsule_cache.item_count, 2);
}

#[test]
fn retrieval_cache_inspection_and_clear_are_explicit() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    assemble_context(&root, "needle", 2).expect("exact search should populate cache");
    assemble_overview(&root, 2).expect("overview should populate cache");

    let before = inspect_retrieval_caches(&root).expect("cache inspection should succeed");
    assert_eq!(before.exact_search_cache.present, true);
    assert_eq!(before.exact_search_cache.item_count, 1);
    assert_eq!(before.retrieval_capsule_cache.present, true);
    assert_eq!(before.retrieval_capsule_cache.item_count, 1);

    let cleared = clear_retrieval_caches(&root).expect("cache clear should succeed");
    assert_eq!(cleared.exact_search_cache_cleared, true);
    assert_eq!(cleared.retrieval_capsule_cache_cleared, true);
    assert_eq!(cleared.status.kind, CacheStatusKind::Cleared);

    let after = inspect_retrieval_caches(&root).expect("cache inspection should succeed");
    assert_eq!(after.exact_search_cache.present, false);
    assert_eq!(after.exact_search_cache.item_count, 0);
    assert_eq!(after.retrieval_capsule_cache.present, false);
    assert_eq!(after.retrieval_capsule_cache.item_count, 0);
}
