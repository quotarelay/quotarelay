use std::fs;
use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_responses, json_rpc_request, temp_repo};
use crate::run_stdio;

#[test]
fn repository_registration_tools_work_over_stdio() {
    let state_root = temp_repo();
    let repo_root = temp_repo();

    let register_request = json_rpc_request(
        24,
        "tools/call",
        json!({
            "name": "register_repository",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_root.to_string_lossy()
            }
        }),
    );
    let list_request = json_rpc_request(
        25,
        "tools/call",
        json!({
            "name": "list_repositories",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "limit": 5
            }
        }),
    );
    let remove_request = json_rpc_request(
        26,
        "tools/call",
        json!({
            "name": "remove_repository",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_root.to_string_lossy()
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        register_request.len(),
        register_request,
        list_request.len(),
        list_request,
        remove_request.len(),
        remove_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("repository registration tools should succeed");

    let responses = decode_responses(&output);
    let registered: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("register text should exist"),
    )
    .expect("registered payload should be valid json");
    let listed: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("list text should exist"),
    )
    .expect("list payload should be valid json");
    let removed: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("remove text should exist"),
    )
    .expect("remove payload should be valid json");

    assert_eq!(listed.as_array().map(|items| items.len()), Some(1));
    assert_eq!(listed[0]["root"], registered["repository"]["root"]);
    assert_eq!(
        removed["repository"]["root"],
        registered["repository"]["root"]
    );
}

#[test]
fn inspect_local_state_works_over_stdio() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

    let sync_request = json_rpc_request(
        61,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let register_request = json_rpc_request(
        62,
        "tools/call",
        json!({
            "name": "register_repository",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "repo_root": state_root.to_string_lossy()
            }
        }),
    );
    let memory_request = json_rpc_request(
        63,
        "tools/call",
        json!({
            "name": "memory_write",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "title": "Needle note",
                "content": "needle memory"
            }
        }),
    );
    let assemble_request = json_rpc_request(
        64,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "exact_search",
                "query": "needle",
                "limit": 2
            }
        }),
    );
    let inspect_request = json_rpc_request(
        65,
        "tools/call",
        json!({
            "name": "inspect_local_state",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            sync_request.len(),
            sync_request,
            register_request.len(),
            register_request,
            memory_request.len(),
            memory_request,
            assemble_request.len(),
            assemble_request,
            inspect_request.len(),
            inspect_request
        );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("local state inspection workflow should succeed");

    let responses = decode_responses(&output);
    let state: Value = serde_json::from_str(
        responses[4]["result"]["content"][0]["text"]
            .as_str()
            .expect("state text should exist"),
    )
    .expect("state payload should be valid json");

    assert_eq!(state["index"]["present"], true);
    assert_eq!(state["index"]["item_count"], 1);
    assert_eq!(state["memory_notes"]["present"], true);
    assert_eq!(state["memory_notes"]["item_count"], 1);
    assert_eq!(state["context_run_history"]["present"], true);
    assert_eq!(state["context_run_history"]["item_count"], 1);
    assert_eq!(state["registered_repositories"]["present"], true);
    assert_eq!(state["registered_repositories"]["item_count"], 1);
    assert_eq!(state["exact_search_cache"]["present"], true);
    assert_eq!(state["exact_search_cache"]["item_count"], 1);
}

#[test]
fn repository_state_tool_reports_sync_and_recent_run_truth_over_stdio() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

    let register_request = json_rpc_request(
        27,
        "tools/call",
        json!({
            "name": "register_repository",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_root.to_string_lossy()
            }
        }),
    );
    let sync_request = json_rpc_request(
        28,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        29,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "exact_search",
                "query": "needle",
                "limit": 2
            }
        }),
    );
    let state_request = json_rpc_request(
        30,
        "tools/call",
        json!({
            "name": "repository_state",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "limit": 5
            }
        }),
    );
    let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            register_request.len(),
            register_request,
            sync_request.len(),
            sync_request,
            assemble_request.len(),
            assemble_request,
            state_request.len(),
            state_request
        );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("repository_state should succeed");

    let responses = decode_responses(&output);
    let state: Value = serde_json::from_str(
        responses[3]["result"]["content"][0]["text"]
            .as_str()
            .expect("repository_state text should exist"),
    )
    .expect("repository_state payload should be valid json");

    assert_eq!(state.as_array().map(|items| items.len()), Some(1));
    assert_eq!(state[0]["sync"]["status"], "indexed");
    assert_eq!(state[0]["sync"]["indexed_files"], 1);
    assert_eq!(state[0]["recent_context_run"]["query"], "needle");
}

#[test]
fn repository_detail_tool_reports_one_repo_truth_over_stdio() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    let missing_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

    let register_request = json_rpc_request(
        31,
        "tools/call",
        json!({
            "name": "register_repository",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_root.to_string_lossy()
            }
        }),
    );
    let sync_request = json_rpc_request(
        32,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        33,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "exact_search",
                "query": "needle",
                "limit": 2
            }
        }),
    );
    let detail_request = json_rpc_request(
        34,
        "tools/call",
        json!({
            "name": "repository_detail",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_root.to_string_lossy()
            }
        }),
    );
    let missing_request = json_rpc_request(
        35,
        "tools/call",
        json!({
            "name": "repository_detail",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": missing_root.to_string_lossy()
            }
        }),
    );
    let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            register_request.len(),
            register_request,
            sync_request.len(),
            sync_request,
            assemble_request.len(),
            assemble_request,
            detail_request.len(),
            detail_request,
            missing_request.len(),
            missing_request
        );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("repository_detail should succeed");

    let responses = decode_responses(&output);
    let detail: Value = serde_json::from_str(
        responses[3]["result"]["content"][0]["text"]
            .as_str()
            .expect("repository_detail text should exist"),
    )
    .expect("repository_detail payload should be valid json");

    assert_eq!(detail["repository"]["id"], detail["repository"]["root"]);
    assert!(detail["repository"]["root"]
        .as_str()
        .expect("repository root should exist")
        .contains("quotarelay-repo-"));
    assert_eq!(detail["sync"]["status"], "indexed");
    assert_eq!(detail["sync"]["indexed_files"], 1);
    assert_eq!(detail["recent_context_run"]["query"], "needle");
    assert_eq!(responses[4]["result"]["content"][0]["text"], "null");
}

#[test]
fn repository_update_metadata_changes_display_name_over_stdio() {
    let state_root = temp_repo();
    let repo_root = temp_repo();

    let register_request = json_rpc_request(
        36,
        "tools/call",
        json!({
            "name": "register_repository",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_root.to_string_lossy()
            }
        }),
    );
    let update_request = json_rpc_request(
        37,
        "tools/call",
        json!({
            "name": "repository_update_metadata",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_root.to_string_lossy(),
                "name": "Workspace API"
            }
        }),
    );
    let detail_request = json_rpc_request(
        38,
        "tools/call",
        json!({
            "name": "repository_detail",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_root.to_string_lossy()
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        register_request.len(),
        register_request,
        update_request.len(),
        update_request,
        detail_request.len(),
        detail_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("repository metadata update should succeed");

    let responses = decode_responses(&output);
    let registered: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("registration text should exist"),
    )
    .expect("registration payload should be valid json");
    let updated: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("metadata update text should exist"),
    )
    .expect("metadata update payload should be valid json");
    let detail: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("repository detail text should exist"),
    )
    .expect("repository detail payload should be valid json");

    assert_eq!(updated["repository"]["name"], "Workspace API");
    assert_eq!(updated["repository"]["id"], registered["repository"]["id"]);
    assert_eq!(
        updated["repository"]["root"],
        registered["repository"]["root"]
    );
    assert_eq!(detail["repository"]["name"], "Workspace API");
    assert_eq!(detail["repository"]["id"], registered["repository"]["id"]);
}
