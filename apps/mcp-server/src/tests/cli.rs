use std::fs;

use context_engine::list_registered_repositories;
use serde_json::Value;

use super::common::temp_repo;
use crate::cli::classify_error;

#[test]
fn local_cli_register_sync_assemble_and_truth_workflow() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

    let mut output = Vec::new();
    super::run_cli(
        [
            "register".to_string(),
            state_root.to_string_lossy().to_string(),
            repo_root.to_string_lossy().to_string(),
        ],
        &mut output,
    )
    .expect("cli register should succeed");

    let registered: Value =
        serde_json::from_slice(&output).expect("cli register payload should parse");
    assert!(registered["ok"].as_bool().unwrap_or(false));
    assert_eq!(registered["command"], "register");

    output.clear();
    super::run_cli(
        ["sync".to_string(), repo_root.to_string_lossy().to_string()],
        &mut output,
    )
    .expect("cli sync should succeed");

    let synced: Value = serde_json::from_slice(&output).expect("cli sync payload should parse");
    assert!(synced["ok"].as_bool().unwrap_or(false));
    assert_eq!(synced["command"], "sync");
    assert_eq!(synced["result"]["sync"]["indexed_files"], 1);

    output.clear();
    super::run_cli(
        ["state".to_string(), repo_root.to_string_lossy().to_string()],
        &mut output,
    )
    .expect("cli state should succeed");

    let state: Value = serde_json::from_slice(&output).expect("cli state payload should parse");
    assert!(state["ok"].as_bool().unwrap_or(false));
    assert_eq!(state["command"], "state");
    assert_eq!(state["result"]["state"]["index"]["present"], true);
    assert_eq!(state["result"]["state"]["index"]["item_count"], 1);

    output.clear();
    super::run_cli(
        [
            "assemble".to_string(),
            repo_root.to_string_lossy().to_string(),
            "exact_search".to_string(),
            "needle".to_string(),
            "2".to_string(),
        ],
        &mut output,
    )
    .expect("cli assemble should succeed");

    let assembly: Value =
        serde_json::from_slice(&output).expect("cli assemble payload should parse");
    assert!(assembly["ok"].as_bool().unwrap_or(false));
    assert_eq!(assembly["command"], "assemble");
    assert_eq!(assembly["result"]["context"]["mode"], "exact_search");
    assert_eq!(
        assembly["result"]["context"]["snippets"]
            .as_array()
            .map(|items| items.len()),
        Some(1)
    );
    assert_eq!(
        assembly["result"]["context"]["budget"]["included_bytes"],
        "needle in repo".len()
    );
    assert_eq!(
        assembly["result"]["context"]["budget"]["approximate_tokens"],
        4
    );

    output.clear();
    super::run_cli(["truth".to_string()], &mut output).expect("cli truth should succeed");

    let truth: Value = serde_json::from_slice(&output).expect("cli truth payload should parse");
    assert!(truth["ok"].as_bool().unwrap_or(false));
    assert_eq!(truth["command"], "truth");
    assert_eq!(
        truth["result"]["truth"]["cache"]["exact_search_enabled"],
        true
    );
    assert_eq!(
        truth["result"]["truth"]["cache"]["sync_invalidates_caches"],
        true
    );
}

#[cfg(windows)]
#[test]
fn local_cli_register_normalizes_windows_style_repo_path_variants() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    let forward_slash_root = repo_root.to_string_lossy().replace('\\', "/");
    let dotted_root = format!("{}\\.", repo_root.to_string_lossy());

    let mut output = Vec::new();
    super::run_cli(
        [
            "register".to_string(),
            state_root.to_string_lossy().to_string(),
            forward_slash_root,
        ],
        &mut output,
    )
    .expect("first cli register should succeed");
    let first: Value =
        serde_json::from_slice(&output).expect("first register payload should parse");

    output.clear();
    super::run_cli(
        [
            "register".to_string(),
            state_root.to_string_lossy().to_string(),
            dotted_root,
        ],
        &mut output,
    )
    .expect("second cli register should succeed");
    let duplicate: Value =
        serde_json::from_slice(&output).expect("second register payload should parse");
    let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");

    assert!(first["ok"].as_bool().unwrap_or(false));
    assert!(duplicate["ok"].as_bool().unwrap_or(false));
    assert_eq!(
        first["result"]["repository"]["id"],
        duplicate["result"]["repository"]["id"]
    );
    assert_eq!(
        first["result"]["repository"]["root"],
        duplicate["result"]["repository"]["root"]
    );
    assert_eq!(listed.len(), 1);
}

#[test]
fn local_cli_uses_stable_json_success_and_error_contract() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");
    repo_index::sync_repo(&repo_root).expect("sync should succeed");

    let mut output = Vec::new();
    super::run_cli(["truth".to_string()], &mut output).expect("cli truth should succeed");
    let truth: Value = serde_json::from_slice(&output).expect("truth payload should parse");
    assert_eq!(truth["ok"], true);
    assert_eq!(truth["command"], "truth");
    assert!(truth["result"]["truth"]["tools"].is_array());
    assert_eq!(
        truth["result"]["truth"]["retrieval"]["budget_estimate_enabled"],
        true
    );
    assert_eq!(
        truth["result"]["truth"]["retrieval"]["budget_estimate_unit"],
        "approximate_tokens_from_included_bytes"
    );

    output.clear();
    super::run_cli(
        [
            "handoff".to_string(),
            repo_root.to_string_lossy().to_string(),
            "Continue T102".to_string(),
            "exact_search".to_string(),
            "needle".to_string(),
            "2".to_string(),
        ],
        &mut output,
    )
    .expect("cli handoff should succeed");

    let handoff: Value = serde_json::from_slice(&output).expect("handoff payload should parse");
    assert!(handoff["ok"].as_bool().unwrap_or(false));
    assert_eq!(handoff["command"], "handoff");
    assert_eq!(handoff["result"]["handoff"]["active_task"], "Continue T102");
    assert_eq!(
        handoff["result"]["handoff"]["context"]["snippets"]
            .as_array()
            .map(|items| items.len()),
        Some(1)
    );
    assert!(handoff["result"]["handoff"]["validation_commands"].is_array());

    output.clear();
    super::run_cli(
        [
            "search".to_string(),
            repo_root.to_string_lossy().to_string(),
            "needle".to_string(),
            "1".to_string(),
        ],
        &mut output,
    )
    .expect("cli search should succeed");
    let search: Value = serde_json::from_slice(&output).expect("search payload should parse");
    assert_eq!(search["ok"], true);
    assert_eq!(search["command"], "search");
    assert_eq!(
        search["result"]["hits"].as_array().map(|items| items.len()),
        Some(1)
    );

    output.clear();
    super::run_cli(
        [
            "assemble".to_string(),
            repo_root.to_string_lossy().to_string(),
            "overview".to_string(),
            "1".to_string(),
        ],
        &mut output,
    )
    .expect("cli overview should succeed");
    let overview: Value = serde_json::from_slice(&output).expect("overview payload should parse");
    assert_eq!(overview["ok"], true);
    assert_eq!(overview["command"], "assemble");
    assert_eq!(overview["result"]["context"]["mode"], "overview");

    output.clear();
    super::run_cli(
        [
            "assemble".to_string(),
            repo_root.to_string_lossy().to_string(),
            "task_capsule".to_string(),
            "needle".to_string(),
            "1".to_string(),
        ],
        &mut output,
    )
    .expect("cli task capsule should succeed");
    let task_capsule: Value =
        serde_json::from_slice(&output).expect("task capsule payload should parse");
    assert_eq!(task_capsule["ok"], true);
    assert_eq!(task_capsule["command"], "assemble");
    assert_eq!(task_capsule["result"]["context"]["mode"], "task_capsule");

    output.clear();
    super::run_cli(Vec::<String>::new(), &mut output)
        .expect("missing command should produce json error");
    let missing: Value =
        serde_json::from_slice(&output).expect("missing command payload should parse");
    assert_eq!(missing["ok"], false);
    assert!(missing["command"].is_null());
    assert_eq!(missing["error_category"], "invalid_args");
    assert_eq!(missing["error"], "missing command");

    output.clear();
    super::run_cli(["unknown".to_string()], &mut output)
        .expect("unknown command should produce json error");
    let unknown: Value =
        serde_json::from_slice(&output).expect("unknown command payload should parse");
    assert_eq!(unknown["ok"], false);
    assert_eq!(unknown["command"], "unknown");
    assert_eq!(unknown["error_category"], "invalid_args");
    assert_eq!(unknown["error"], "unknown command: unknown");

    output.clear();
    super::run_cli(
        [
            "search".to_string(),
            repo_root.to_string_lossy().to_string(),
            "needle".to_string(),
            "not-a-number".to_string(),
        ],
        &mut output,
    )
    .expect("invalid limit should produce json error");
    let invalid_limit: Value =
        serde_json::from_slice(&output).expect("invalid limit payload should parse");
    assert_eq!(invalid_limit["ok"], false);
    assert_eq!(invalid_limit["command"], "search");
    assert_eq!(invalid_limit["error_category"], "invalid_args");
    assert!(invalid_limit["error"]
        .as_str()
        .expect("error should be a string")
        .starts_with("limit must be a positive integer"));
}

#[test]
fn error_classifier_uses_stable_categories() {
    assert_eq!(classify_error("missing command"), "invalid_args");
    assert_eq!(
        classify_error("repository C:\\repo is not registered"),
        "missing_state"
    );
    assert_eq!(classify_error("corrupt memory_notes.json"), "corrupt_state");
    assert_eq!(
        classify_error("sync failed: disk unavailable"),
        "backend_error"
    );
}
