use std::io::Cursor;

use serde_json::json;

use super::common::decode_response;
use crate::run_stdio;

#[test]
fn responds_to_initialize_over_stdio() {
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let framed = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("stdio transport should succeed");

    let response = decode_response(&output);
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["serverInfo"]["name"], "quotarelay");
}

#[test]
fn responds_to_tool_call_over_stdio() {
    let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bootstrap_status","arguments":{}}}"#;
    let framed = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("tool call should succeed");

    let response = decode_response(&output);
    assert_eq!(response["id"], 2);
    assert_eq!(
        response["result"]["content"][0]["text"],
        "quotarelay bootstrap"
    );
}

#[test]
fn tools_list_exposes_current_backend_truth_surface() {
    let request = r#"{"jsonrpc":"2.0","id":12,"method":"tools/list","params":{}}"#;
    let framed = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("tools list should succeed");

    let response = decode_response(&output);
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    let names = tools
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name should exist"))
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "bootstrap_status",
            "sync_repo",
            "repo_inventory",
            "inspect_local_state",
            "cache_inspect",
            "cache_clear",
            "register_repository",
            "list_repositories",
            "repository_state",
            "repository_detail",
            "repository_update_metadata",
            "remove_repository",
            "workspace_profile_save",
            "workspace_profile_list",
            "search_code",
            "memory_write",
            "memory_read",
            "memory_update",
            "memory_delete",
            "memory_export",
            "memory_import",
            "memory_search",
            "context_run_detail",
            "context_run_history",
            "multi_repo_assemble_context",
            "assemble_context",
            "handoff_packet",
        ]
    );

    let assemble_tool = tools
        .iter()
        .find(|tool| tool["name"] == "assemble_context")
        .expect("assemble_context tool should exist");
    assert_eq!(
        assemble_tool["inputSchema"]["properties"]["mode"]["enum"],
        json!(["exact_search", "overview", "task_capsule", "diff_aware"])
    );
}
