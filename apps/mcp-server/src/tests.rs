
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::Request;
use serde_json::{json, Value};
use tower::ServiceExt;

use context_engine::{
    assemble_context, list_registered_repositories, memory_write, register_repository,
};

use super::{cli::classify_error, http_router, run_stdio};

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
        ]
    );

    let assemble_tool = tools
        .iter()
        .find(|tool| tool["name"] == "assemble_context")
        .expect("assemble_context tool should exist");
    assert_eq!(
        assemble_tool["inputSchema"]["properties"]["mode"]["enum"],
        json!(["exact_search", "overview", "task_capsule"])
    );
}

#[test]
fn sync_and_search_work_over_stdio() {
    let repo_root = temp_repo();
    fs::write(
        repo_root.join("src.txt"),
        "bootstrap marker\nsearch target\n",
    )
    .expect("repo file should write");

    let sync_request = json_rpc_request(
        3,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let search_request = json_rpc_request(
        4,
        "tools/call",
        json!({
            "name": "search_code",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "query": "search target",
                "limit": 3
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        search_request.len(),
        search_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("sync and search should succeed");

    let responses = decode_responses(&output);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["id"], 3);
    assert!(responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("sync text should exist")
        .contains("synced 1 files"));
    assert_eq!(responses[1]["id"], 4);
    assert!(responses[1]["result"]["content"][0]["text"]
        .as_str()
        .expect("search text should exist")
        .contains("search target"));
}

#[test]
fn repo_inventory_works_over_stdio() {
    let repo_root = temp_repo();
    fs::write(
        repo_root.join("src.txt"),
        "bootstrap marker\nsearch target\n",
    )
    .expect("repo file should write");

    let sync_request = json_rpc_request(
        7,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let inventory_request = json_rpc_request(
        8,
        "tools/call",
        json!({
            "name": "repo_inventory",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        inventory_request.len(),
        inventory_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("inventory call should succeed");

    let responses = decode_responses(&output);
    assert_eq!(responses[1]["id"], 8);
    let inventory: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("inventory text should exist"),
    )
    .expect("inventory payload should be valid json");
    assert_eq!(inventory["indexed_files"], 1);
    assert_eq!(inventory["sample_paths"][0], "src.txt");
    assert!(inventory["indexed_at_epoch_ms"].as_u64().is_some());
}

#[test]
fn assemble_context_works_over_stdio() {
    let repo_root = temp_repo();
    fs::write(
        repo_root.join("src.txt"),
        "alpha needle\nbeta needle\ngamma needle\n",
    )
    .expect("repo file should write");

    let sync_request = json_rpc_request(
        5,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        6,
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
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        assemble_request.len(),
        assemble_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("assemble_context should succeed");

    let responses = decode_responses(&output);
    assert_eq!(responses[1]["id"], 6);
    let assembly: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("assemble text should exist"),
    )
    .expect("assembly payload should be valid json");
    assert_eq!(assembly["mode"], "exact_search");
    assert_eq!(
        assembly["snippets"].as_array().map(|items| items.len()),
        Some(2)
    );
    assert_eq!(
        assembly["memory_notes"].as_array().map(|items| items.len()),
        Some(0)
    );
    assert_eq!(
        assembly["snippets"][0]["reason"]["kind"],
        "query_line_match"
    );
    assert_eq!(assembly["omissions"][0]["kind"], "item_limit_reached");
}

#[test]
fn assemble_context_supports_overview_and_task_capsule_modes_over_stdio() {
    let repo_root = temp_repo();
    fs::write(
            repo_root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n\nfn plan() -> usize {\n    let body_only_term = 7;\n    body_only_term\n}\n",
        )
        .expect("rust file should write");

    let sync_request = json_rpc_request(
        13,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let overview_request = json_rpc_request(
        14,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "overview",
                "limit": 2
            }
        }),
    );
    let task_request = json_rpc_request(
        15,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "task_capsule",
                "query": "Widget",
                "limit": 2
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        overview_request.len(),
        overview_request,
        task_request.len(),
        task_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("mode-specific assemble_context calls should succeed");

    let responses = decode_responses(&output);
    let overview: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("overview text should exist"),
    )
    .expect("overview payload should be valid json");
    assert_eq!(overview["mode"], "overview");
    assert_eq!(
        overview["documents"].as_array().map(|items| items.len()),
        Some(1)
    );

    let task: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("task text should exist"),
    )
    .expect("task payload should be valid json");
    assert_eq!(task["mode"], "task_capsule");
    assert_eq!(task["documents"][0]["reason"]["kind"], "task_capsule_match");
    assert!(task["documents"][0]["contents"]
        .as_str()
        .expect("task contents should exist")
        .contains("struct Widget { id: usize }"));
}

#[test]
fn durable_memory_tools_work_over_stdio() {
    let repo_root = temp_repo();

    let write_request = json_rpc_request(
        18,
        "tools/call",
        json!({
            "name": "memory_write",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "title": "Design note",
                "content": "Persistent memory should survive restarts.",
                "tags": ["memory", "design"]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        write_request.len(),
        write_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_write should succeed");

    let write_response = decode_response(&output);
    let created: Value = serde_json::from_str(
        write_response["result"]["content"][0]["text"]
            .as_str()
            .expect("memory write text should exist"),
    )
    .expect("memory write payload should be valid json");
    let note_id = created["note"]["id"]
        .as_str()
        .expect("note id should exist");

    let read_request = json_rpc_request(
        19,
        "tools/call",
        json!({
            "name": "memory_read",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "id": note_id
            }
        }),
    );
    let search_request = json_rpc_request(
        20,
        "tools/call",
        json!({
            "name": "memory_search",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "query": "memory",
                "limit": 2
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        read_request.len(),
        read_request,
        search_request.len(),
        search_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("memory read and search should succeed");

    let responses = decode_responses(&output);
    let loaded: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("memory read text should exist"),
    )
    .expect("memory read payload should be valid json");
    assert_eq!(loaded["title"], "Design note");

    let search_results: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("memory search text should exist"),
    )
    .expect("memory search payload should be valid json");
    assert_eq!(
        search_results["notes"].as_array().map(|items| items.len()),
        Some(1)
    );
    assert_eq!(search_results["notes"][0]["title"], "Design note");
}

#[test]
fn durable_memory_update_and_delete_work_over_stdio() {
    let repo_root = temp_repo();

    let write_request = json_rpc_request(
        27,
        "tools/call",
        json!({
            "name": "memory_write",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "title": "Design note",
                "content": "Original memory content.",
                "tags": ["memory"]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        write_request.len(),
        write_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_write should succeed");
    let created_response = decode_response(&output);
    let created: Value = serde_json::from_str(
        created_response["result"]["content"][0]["text"]
            .as_str()
            .expect("memory write text should exist"),
    )
    .expect("memory write payload should be valid json");
    let note_id = created["note"]["id"]
        .as_str()
        .expect("note id should exist")
        .to_string();

    let update_request = json_rpc_request(
        28,
        "tools/call",
        json!({
            "name": "memory_update",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "id": note_id,
                "title": "Updated note",
                "content": "Updated memory content.",
                "tags": ["updated"]
            }
        }),
    );
    let delete_request = json_rpc_request(
        29,
        "tools/call",
        json!({
            "name": "memory_delete",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "id": created["note"]["id"]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        update_request.len(),
        update_request,
        delete_request.len(),
        delete_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("memory update and delete should succeed");
    let responses = decode_responses(&output);
    let updated: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("update text should exist"),
    )
    .expect("update payload should be valid json");
    let deleted: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("delete text should exist"),
    )
    .expect("delete payload should be valid json");

    assert_eq!(updated["note"]["title"], "Updated note");
    assert_eq!(updated["note"]["content"], "Updated memory content.");
    assert_eq!(updated["note"]["tags"], json!(["updated"]));
    assert_eq!(deleted["note"]["id"], updated["note"]["id"]);

    let read_request = json_rpc_request(
        30,
        "tools/call",
        json!({
            "name": "memory_read",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "id": updated["note"]["id"]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        read_request.len(),
        read_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory read should succeed");
    let read_response = decode_response(&output);
    assert_eq!(read_response["result"]["content"][0]["text"], "null");
}

#[test]
fn durable_memory_export_and_import_work_over_stdio() {
    let source_root = temp_repo();
    let target_root = temp_repo();

    let write_request = json_rpc_request(
        31,
        "tools/call",
        json!({
            "name": "memory_write",
            "arguments": {
                "root": source_root.to_string_lossy(),
                "title": "Design note",
                "content": "Memory content for migration.",
                "tags": ["memory"]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        write_request.len(),
        write_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_write should succeed");

    let export_request = json_rpc_request(
        32,
        "tools/call",
        json!({
            "name": "memory_export",
            "arguments": {
                "root": source_root.to_string_lossy(),
                "limit": 10
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        export_request.len(),
        export_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_export should succeed");
    let export_response = decode_response(&output);
    let exported: Value = serde_json::from_str(
        export_response["result"]["content"][0]["text"]
            .as_str()
            .expect("export text should exist"),
    )
    .expect("export payload should be valid json");

    assert_eq!(
        exported["payload"]["notes"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(exported["omitted_count"], 0);

    let import_request = json_rpc_request(
        33,
        "tools/call",
        json!({
            "name": "memory_import",
            "arguments": {
                "root": target_root.to_string_lossy(),
                "payload": exported["payload"]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        import_request.len(),
        import_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_import should succeed");
    let import_response = decode_response(&output);
    let imported: Value = serde_json::from_str(
        import_response["result"]["content"][0]["text"]
            .as_str()
            .expect("import text should exist"),
    )
    .expect("import payload should be valid json");

    assert_eq!(imported["imported_count"], 1);
    assert_eq!(imported["replaced_count"], 0);
    assert_eq!(imported["omitted_count"], 0);

    let invalid_import_request = json_rpc_request(
        34,
        "tools/call",
        json!({
            "name": "memory_import",
            "arguments": {
                "root": target_root.to_string_lossy(),
                "payload": { "notes": [{ "id": 7 }] }
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        invalid_import_request.len(),
        invalid_import_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("invalid memory_import request should still produce a response");
    let invalid_response = decode_response(&output);
    assert!(invalid_response["result"]["content"][0]["text"]
        .as_str()
        .expect("invalid import text should exist")
        .contains("memory_import payload is invalid"));
}

#[test]
fn memory_write_rejects_non_string_tags_over_stdio() {
    let repo_root = temp_repo();

    let write_request = json_rpc_request(
        46,
        "tools/call",
        json!({
            "name": "memory_write",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "title": "Design note",
                "content": "Memory content.",
                "tags": ["memory", 7]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        write_request.len(),
        write_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("memory_write should produce an error response");
    let response = decode_response(&output);

    assert_eq!(
        response["result"]["content"][0]["text"],
        "memory_write tags must be an array of strings"
    );
}

#[test]
fn corrupt_memory_state_reports_recoverable_error_over_stdio() {
    let repo_root = temp_repo();
    let state_dir = repo_root.join(".quotarelay");
    fs::create_dir_all(&state_dir).expect("state dir should create");
    fs::write(state_dir.join("memory_notes.json"), "{ not json")
        .expect("corrupt memory should write");

    let search_request = json_rpc_request(
        47,
        "tools/call",
        json!({
            "name": "memory_search",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "query": "needle",
                "limit": 3
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        search_request.len(),
        search_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("corrupt memory search should produce an error response");
    let response = decode_response(&output);
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("error text should exist");

    assert!(text.contains("memory_search failed: corrupt local state file"));
    assert!(text.contains("repair or remove the file to recover"));
}

#[test]
fn cache_inspect_and_clear_work_over_stdio() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

    let sync_request = json_rpc_request(
        48,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        49,
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
        50,
        "tools/call",
        json!({
            "name": "cache_inspect",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let clear_request = json_rpc_request(
        51,
        "tools/call",
        json!({
            "name": "cache_clear",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let inspect_after_request = json_rpc_request(
        52,
        "tools/call",
        json!({
            "name": "cache_inspect",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            sync_request.len(),
            sync_request,
            assemble_request.len(),
            assemble_request,
            inspect_request.len(),
            inspect_request,
            clear_request.len(),
            clear_request,
            inspect_after_request.len(),
            inspect_after_request
        );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("cache inspect and clear should succeed");
    let responses = decode_responses(&output);
    let before: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("cache inspect text should exist"),
    )
    .expect("cache inspect payload should be valid json");
    let cleared: Value = serde_json::from_str(
        responses[3]["result"]["content"][0]["text"]
            .as_str()
            .expect("cache clear text should exist"),
    )
    .expect("cache clear payload should be valid json");
    let after: Value = serde_json::from_str(
        responses[4]["result"]["content"][0]["text"]
            .as_str()
            .expect("cache inspect after text should exist"),
    )
    .expect("cache inspect after payload should be valid json");

    assert_eq!(before["exact_search_cache"]["present"], true);
    assert_eq!(before["exact_search_cache"]["item_count"], 1);
    assert_eq!(cleared["exact_search_cache_cleared"], true);
    assert_eq!(after["exact_search_cache"]["present"], false);
}

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

#[test]
fn multi_repo_assemble_context_works_over_stdio() {
    let state_root = temp_repo();
    let repo_one = temp_repo();
    let repo_two = temp_repo();
    fs::write(repo_one.join("alpha.txt"), "needle one\nneedle two\n")
        .expect("repo one file should write");
    fs::write(repo_two.join("beta.txt"), "needle three\nneedle four\n")
        .expect("repo two file should write");

    let register_one = json_rpc_request(
        39,
        "tools/call",
        json!({
            "name": "register_repository",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_one.to_string_lossy()
            }
        }),
    );
    let register_two = json_rpc_request(
        40,
        "tools/call",
        json!({
            "name": "register_repository",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_root": repo_two.to_string_lossy()
            }
        }),
    );
    let sync_one = json_rpc_request(
        41,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_one.to_string_lossy()
            }
        }),
    );
    let sync_two = json_rpc_request(
        42,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_two.to_string_lossy()
            }
        }),
    );
    let assemble = json_rpc_request(
        43,
        "tools/call",
        json!({
            "name": "multi_repo_assemble_context",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "repo_roots": [repo_one.to_string_lossy(), repo_two.to_string_lossy()],
                "query": "needle",
                "per_repo_limit": 1
            }
        }),
    );
    let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            register_one.len(),
            register_one,
            register_two.len(),
            register_two,
            sync_one.len(),
            sync_one,
            sync_two.len(),
            sync_two,
            assemble.len(),
            assemble
        );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("multi repo assembly should succeed");

    let responses = decode_responses(&output);
    let assembly: Value = serde_json::from_str(
        responses[4]["result"]["content"][0]["text"]
            .as_str()
            .expect("multi repo assembly text should exist"),
    )
    .expect("multi repo assembly payload should be valid json");

    assert_eq!(assembly["query"], "needle");
    assert_eq!(assembly["repositories"].as_array().map(Vec::len), Some(2));
    assert_eq!(
        assembly["repositories"][0]["assembly"]["snippets"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        assembly["repositories"][1]["assembly"]["snippets"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        assembly["repositories"][0]["assembly"]["omissions"][0]["kind"],
        "item_limit_reached"
    );
}

#[test]
fn workspace_profiles_save_and_list_over_stdio() {
    let state_root = temp_repo();
    let repo_one = temp_repo();
    let repo_two = temp_repo();

    let save_request = json_rpc_request(
        44,
        "tools/call",
        json!({
            "name": "workspace_profile_save",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "name": "local-api",
                "repo_roots": [repo_one.to_string_lossy(), repo_two.to_string_lossy()],
                "default_mode": "task_capsule",
                "default_limit": 5,
                "per_repo_limit": 2
            }
        }),
    );
    let list_request = json_rpc_request(
        45,
        "tools/call",
        json!({
            "name": "workspace_profile_list",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "limit": 10
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        save_request.len(),
        save_request,
        list_request.len(),
        list_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("workspace profile calls should succeed");

    let responses = decode_responses(&output);
    let saved: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("workspace profile save text should exist"),
    )
    .expect("workspace profile save payload should be valid json");
    let listed: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("workspace profile list text should exist"),
    )
    .expect("workspace profile list payload should be valid json");

    assert_eq!(saved["profile"]["name"], "local-api");
    assert_eq!(saved["profile"]["default_mode"], "task_capsule");
    assert_eq!(saved["profile"]["default_limit"], 5);
    assert_eq!(saved["profile"]["per_repo_limit"], 2);
    assert_eq!(listed.as_array().map(Vec::len), Some(1));
    assert_eq!(listed[0]["repo_roots"].as_array().map(Vec::len), Some(2));
}

#[test]
fn assemble_context_includes_matching_memory_over_stdio() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

    let sync_request = json_rpc_request(
        21,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let write_request = json_rpc_request(
        22,
        "tools/call",
        json!({
            "name": "memory_write",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "title": "Needle note",
                "content": "needle should appear in assembled context memory",
                "tags": ["memory"]
            }
        }),
    );
    let assemble_request = json_rpc_request(
        23,
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
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        write_request.len(),
        write_request,
        assemble_request.len(),
        assemble_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("memory-aware assemble_context should succeed");

    let responses = decode_responses(&output);
    let assembly: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("assembly text should exist"),
    )
    .expect("assembly payload should be valid json");
    assert_eq!(
        assembly["memory_notes"].as_array().map(|items| items.len()),
        Some(1)
    );
    assert_eq!(
        assembly["memory_notes"][0]["reason"]["kind"],
        "memory_note_match"
    );
    assert_eq!(assembly["memory_notes"][0]["title"], "Needle note");
}

#[test]
fn assemble_context_exact_search_cache_honors_sync_invalidation_over_stdio() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

    let sync_request = json_rpc_request(
        31,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        32,
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
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        assemble_request.len(),
        assemble_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("initial exact_search should succeed");

    let responses = decode_responses(&output);
    let first: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("first assembly text should exist"),
    )
    .expect("first assembly payload should be valid json");

    fs::write(repo_root.join("src.txt"), "fresh needle after sync\n")
        .expect("repo file should rewrite");
    let resync_request = json_rpc_request(
        33,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );

    let mut second_output = Vec::new();
    let second_request = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        resync_request.len(),
        resync_request,
        assemble_request.len(),
        assemble_request,
        assemble_request.len(),
        assemble_request
    );
    run_stdio(Cursor::new(second_request.into_bytes()), &mut second_output)
        .expect("repeated exact_search should succeed");

    let second_responses = decode_responses(&second_output);
    let cached: Value = serde_json::from_str(
        second_responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("cached assembly text should exist"),
    )
    .expect("cached assembly payload should be valid json");
    let refreshed: Value = serde_json::from_str(
        second_responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("refreshed assembly text should exist"),
    )
    .expect("refreshed assembly payload should be valid json");

    assert_eq!(cached, refreshed);
    assert_eq!(cached["snippets"][0]["line"], "fresh needle after sync");
    assert_ne!(cached, first);
}

#[test]
fn assemble_context_exact_search_cache_invalidation_refreshes_memory_sensitive_results_over_stdio()
{
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

    let sync_request = json_rpc_request(
        36,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        37,
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
    let initial = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        assemble_request.len(),
        assemble_request
    );
    let mut initial_output = Vec::new();
    run_stdio(Cursor::new(initial.into_bytes()), &mut initial_output)
        .expect("initial exact_search should succeed");

    let initial_responses = decode_responses(&initial_output);
    let cached_without_memory: Value = serde_json::from_str(
        initial_responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("initial assembly text should exist"),
    )
    .expect("initial assembly payload should be valid json");
    assert_eq!(
        cached_without_memory["memory_notes"]
            .as_array()
            .map(|items| items.len()),
        Some(0)
    );

    let write_memory_request = json_rpc_request(
        38,
        "tools/call",
        json!({
            "name": "memory_write",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "title": "Needle note",
                "content": "Remember the needle workflow",
                "tags": ["needle"]
            }
        }),
    );
    let resync_request = json_rpc_request(
        39,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let refreshed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        write_memory_request.len(),
        write_memory_request,
        resync_request.len(),
        resync_request,
        assemble_request.len(),
        assemble_request
    );
    let mut refreshed_output = Vec::new();
    run_stdio(Cursor::new(refreshed.into_bytes()), &mut refreshed_output)
        .expect("memory-sensitive exact_search should refresh after sync invalidation");

    let refreshed_responses = decode_responses(&refreshed_output);
    let refreshed_with_memory: Value = serde_json::from_str(
        refreshed_responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("refreshed assembly text should exist"),
    )
    .expect("refreshed assembly payload should be valid json");

    assert_eq!(
        refreshed_with_memory["memory_notes"]
            .as_array()
            .map(|items| items.len()),
        Some(1)
    );
    assert_eq!(
        refreshed_with_memory["memory_notes"][0]["title"],
        "Needle note"
    );
    assert_eq!(
        refreshed_with_memory["memory_notes"][0]["reason"]["kind"],
        "memory_note_match"
    );
}

#[test]
fn assemble_context_exact_search_cache_normalizes_equivalent_queries_over_stdio() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

    let sync_request = json_rpc_request(
        31,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let first_assemble_request = json_rpc_request(
        32,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "exact_search",
                "query": "Needle",
                "limit": 2
            }
        }),
    );
    let second_assemble_request = json_rpc_request(
        33,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "exact_search",
                "query": "  needle  ",
                "limit": 2
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        first_assemble_request.len(),
        first_assemble_request,
        second_assemble_request.len(),
        second_assemble_request,
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("canonicalized exact_search should succeed");

    let responses = decode_responses(&output);
    let first: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("first assembly text should exist"),
    )
    .expect("first assembly payload should be valid json");
    let second: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("second assembly text should exist"),
    )
    .expect("second assembly payload should be valid json");

    assert_eq!(first["query"], second["query"]);
    assert_eq!(
        first["generated_at_epoch_ms"],
        second["generated_at_epoch_ms"]
    );
    assert_eq!(first["snippets"], second["snippets"]);
}

#[test]
fn assemble_context_overview_and_task_capsule_responses_are_repeatable_over_stdio() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
    fs::write(
        repo_root.join("lib.rs"),
        "struct Widget {\n    id: usize,\n}\n",
    )
    .expect("rust file should write");

    let sync_request = json_rpc_request(
        35,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let overview_request = json_rpc_request(
        36,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "overview",
                "limit": 2
            }
        }),
    );
    let task_request = json_rpc_request(
        37,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "task_capsule",
                "query": "Widget",
                "limit": 2
            }
        }),
    );
    let repeated_overview_request = json_rpc_request(
        38,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "overview",
                "limit": 2
            }
        }),
    );
    let repeated_task_request = json_rpc_request(
        39,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "task_capsule",
                "query": "Widget",
                "limit": 2
            }
        }),
    );
    let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            sync_request.len(),
            sync_request,
            overview_request.len(),
            overview_request,
            task_request.len(),
            task_request,
            repeated_overview_request.len(),
            repeated_overview_request,
            repeated_task_request.len(),
            repeated_task_request
        );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("repeatable overview and task capsule responses should succeed");

    let responses = decode_responses(&output);
    let first_overview: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("overview text should exist"),
    )
    .expect("overview payload should be valid json");
    let first_task: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("task text should exist"),
    )
    .expect("task payload should be valid json");
    let second_overview: Value = serde_json::from_str(
        responses[3]["result"]["content"][0]["text"]
            .as_str()
            .expect("second overview text should exist"),
    )
    .expect("second overview payload should be valid json");
    let second_task: Value = serde_json::from_str(
        responses[4]["result"]["content"][0]["text"]
            .as_str()
            .expect("second task text should exist"),
    )
    .expect("second task payload should be valid json");

    assert_eq!(first_overview, second_overview);
    assert_eq!(first_task, second_task);
}

#[test]
fn assemble_context_capsule_caches_honor_sync_invalidation_over_stdio() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
    fs::write(
        repo_root.join("lib.rs"),
        "struct Widget {\n    id: usize,\n}\n",
    )
    .expect("rust file should write");

    let sync_request = json_rpc_request(
        51,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let overview_request = json_rpc_request(
        52,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "overview",
                "limit": 2
            }
        }),
    );
    let task_request = json_rpc_request(
        53,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "task_capsule",
                "query": "Widget",
                "limit": 2
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        overview_request.len(),
        overview_request,
        task_request.len(),
        task_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("initial capsule modes should succeed");

    let responses = decode_responses(&output);
    let first_overview: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("overview text should exist"),
    )
    .expect("overview payload should be valid json");
    let first_task: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("task text should exist"),
    )
    .expect("task payload should be valid json");

    fs::write(repo_root.join("alpha.txt"), "fresh overview after sync\n")
        .expect("alpha file should rewrite");
    fs::write(
        repo_root.join("lib.rs"),
        "struct Widget {\n    id: u32,\n}\n",
    )
    .expect("rust file should rewrite");
    let resync_request = json_rpc_request(
        54,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let second_framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            resync_request.len(),
            resync_request,
            overview_request.len(),
            overview_request,
            overview_request.len(),
            overview_request,
            task_request.len(),
            task_request,
            task_request.len(),
            task_request
        );
    let mut second_output = Vec::new();

    run_stdio(Cursor::new(second_framed.into_bytes()), &mut second_output)
        .expect("refreshed capsule modes should succeed");

    let second_responses = decode_responses(&second_output);
    let refreshed_overview: Value = serde_json::from_str(
        second_responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("refreshed overview text should exist"),
    )
    .expect("refreshed overview payload should be valid json");
    let cached_overview: Value = serde_json::from_str(
        second_responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("cached overview text should exist"),
    )
    .expect("cached overview payload should be valid json");
    let refreshed_task: Value = serde_json::from_str(
        second_responses[3]["result"]["content"][0]["text"]
            .as_str()
            .expect("refreshed task text should exist"),
    )
    .expect("refreshed task payload should be valid json");
    let cached_task: Value = serde_json::from_str(
        second_responses[4]["result"]["content"][0]["text"]
            .as_str()
            .expect("cached task text should exist"),
    )
    .expect("cached task payload should be valid json");

    assert_eq!(refreshed_overview, cached_overview);
    assert_eq!(refreshed_task, cached_task);
    assert_eq!(
        refreshed_overview["documents"][0]["contents"],
        "fresh overview after sync\n"
    );
    assert!(refreshed_task["documents"][0]["contents"]
        .as_str()
        .expect("refreshed task contents should exist")
        .contains("id: u32"));
    assert_ne!(refreshed_overview, first_overview);
    assert_ne!(refreshed_task, first_task);
}

#[tokio::test]
async fn backend_truth_endpoint_exposes_current_contract() {
    let response = http_router()
        .oneshot(
            Request::builder()
                .uri("/truth")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("truth endpoint should respond");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let payload: Value = serde_json::from_slice(&body).expect("truth payload should be valid json");

    assert_eq!(
        payload["tools"].as_array().map(|items| items.len()),
        Some(26)
    );
    assert_eq!(
        payload["retrieval"]["modes"],
        json!(["exact_search", "overview", "task_capsule"])
    );
    assert_eq!(payload["retrieval"]["limits"]["max_context_items"], 5);
    assert_eq!(payload["retrieval"]["durable_memory_enabled"], true);
    assert_eq!(payload["retrieval"]["cache"]["exact_search_enabled"], true);
    assert_eq!(payload["retrieval"]["cache"]["overview_enabled"], true);
    assert_eq!(payload["retrieval"]["cache"]["task_capsule_enabled"], true);
    assert_eq!(
        payload["retrieval"]["cache"]["sync_invalidates_caches"],
        true
    );
    assert_eq!(payload["cache"]["exact_search_enabled"], true);
    assert_eq!(payload["cache"]["overview_enabled"], true);
    assert_eq!(payload["cache"]["task_capsule_enabled"], true);
    assert_eq!(payload["cache"]["sync_invalidates_caches"], true);
    assert_eq!(payload["config"]["workspace_profiles_enabled"], true);
    assert_eq!(
        payload["config"]["workspace_profiles_apply_to_retrieval"],
        false
    );
    assert_eq!(payload["config"]["max_workspace_profiles"], 20);
    assert_eq!(payload["config"]["max_profile_repo_roots"], 5);
    assert_eq!(payload["config"]["default_mode"], "exact_search");
    assert_eq!(payload["config"]["default_limit"], 3);
    assert_eq!(payload["config"]["per_repo_limit"], 3);
    assert_eq!(payload["cli"]["local_entrypoint_enabled"], true);
    assert!(
        payload["cli"]["commands"]
            .as_array()
            .map(|items| items.len())
            .unwrap_or_default()
            >= 5
    );
    assert_eq!(
        payload["cli"]["commands"][0]["command"],
        "cargo run -p mcp-server -- --cli truth"
    );
    assert!(
        payload["proofs"]
            .as_array()
            .map(|items| items.len())
            .unwrap_or_default()
            >= 5
    );
}

#[tokio::test]
async fn repository_state_endpoint_returns_registered_repo_truth() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
    register_repository(&state_root, &repo_root).expect("registration should succeed");
    repo_index::sync_repo(&repo_root).expect("sync should succeed");
    assemble_context(&repo_root, "needle", 2).expect("assembly should succeed");

    let uri = format!(
        "/repositories?root={}&limit=5",
        state_root.to_string_lossy()
    );
    let response = http_router()
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("repository state endpoint should respond");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let payload: Value =
        serde_json::from_slice(&body).expect("repository state payload should be valid json");

    assert_eq!(payload.as_array().map(Vec::len), Some(1));
    assert_eq!(payload[0]["sync"]["status"], "indexed");
    assert_eq!(payload[0]["sync"]["indexed_files"], 1);
    assert_eq!(payload[0]["recent_context_run"]["query"], "needle");
}

#[tokio::test]
async fn memory_search_endpoint_returns_bounded_memory_truth() {
    let root = temp_repo();
    memory_write(
        &root,
        "Needle note",
        "needle memory",
        &["memory".to_string()],
    )
    .expect("memory write should succeed");

    let uri = format!(
        "/memory?root={}&query=needle&limit=3",
        root.to_string_lossy()
    );
    let response = http_router()
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("memory endpoint should respond");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let payload: Value =
        serde_json::from_slice(&body).expect("memory payload should be valid json");

    assert_eq!(payload["notes"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["notes"][0]["title"], "Needle note");
    assert_eq!(payload["omitted_count"], 0);
}

#[tokio::test]
async fn context_runs_endpoint_returns_bounded_explainable_history() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle one\nneedle two\n").expect("repo file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");
    assemble_context(&root, "needle", 1).expect("assembly should succeed");

    let uri = format!("/context-runs?root={}&limit=5", root.to_string_lossy());
    let response = http_router()
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("context runs endpoint should respond");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let payload: Value =
        serde_json::from_slice(&body).expect("context run payload should be valid json");

    assert_eq!(payload.as_array().map(Vec::len), Some(1));
    assert_eq!(payload[0]["query"], "needle");
    assert_eq!(
        payload[0]["snippets"][0]["reason"]["kind"],
        "query_line_match"
    );
    assert_eq!(payload[0]["omissions"][0]["kind"], "item_limit_reached");
}

#[tokio::test]
async fn local_operator_workflow_is_visible_through_truth_and_stdio() {
    let state_root = temp_repo();
    let repo_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

    let register_request = json_rpc_request(
        41,
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
        42,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        43,
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
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        register_request.len(),
        register_request,
        sync_request.len(),
        sync_request,
        assemble_request.len(),
        assemble_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("local operator workflow should succeed");

    let responses = decode_responses(&output);
    let registered: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("register text should exist"),
    )
    .expect("register payload should be valid json");
    let assembly: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("assembly text should exist"),
    )
    .expect("assembly payload should be valid json");

    assert_eq!(
        registered["repository"]["name"],
        repo_root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
    );
    assert_eq!(assembly["query"], "needle");
    assert_eq!(
        assembly["snippets"].as_array().map(|items| items.len()),
        Some(1)
    );

    let response = http_router()
        .oneshot(
            Request::builder()
                .uri("/truth")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("truth endpoint should respond");
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let payload: Value = serde_json::from_slice(&body).expect("truth payload should be valid json");

    assert!(payload["proofs"]
        .as_array()
        .expect("proofs should be an array")
        .iter()
        .any(|proof| proof["id"] == "local_operator_workflow"));
}

#[test]
fn assemble_context_reports_byte_budget_omissions_over_stdio() {
    let repo_root = temp_repo();
    let long_line = format!("needle {}", "x".repeat(300));
    fs::write(repo_root.join("alpha.txt"), format!("{long_line}\n"))
        .expect("repo file should write");

    let sync_request = json_rpc_request(
        16,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        17,
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
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        assemble_request.len(),
        assemble_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("assemble_context should succeed");

    let responses = decode_responses(&output);
    let assembly: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("assembly text should exist"),
    )
    .expect("assembly payload should be valid json");
    assert_eq!(assembly["omissions"][0]["kind"], "byte_budget_reached");
    assert!(assembly["snippets"][0]["line"]
        .as_str()
        .expect("snippet line should exist")
        .ends_with("..."));
}

#[test]
fn context_run_history_works_over_stdio() {
    let repo_root = temp_repo();
    fs::write(
        repo_root.join("src.txt"),
        "alpha needle\nbeta needle\ngamma needle\n",
    )
    .expect("repo file should write");

    let sync_request = json_rpc_request(
        9,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        10,
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
    let history_request = json_rpc_request(
        11,
        "tools/call",
        json!({
            "name": "context_run_history",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "limit": 1
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        assemble_request.len(),
        assemble_request,
        history_request.len(),
        history_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("history call should succeed");

    let responses = decode_responses(&output);
    assert_eq!(responses[2]["id"], 11);
    let history: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("history text should exist"),
    )
    .expect("history payload should be valid json");
    assert_eq!(history.as_array().map(|items| items.len()), Some(1));
    assert_eq!(history[0]["query"], "needle");
    assert_eq!(
        history[0]["snippets"][0]["reason"]["kind"],
        "query_line_match"
    );
    assert_eq!(history[0]["omissions"][0]["kind"], "item_limit_reached");
}

#[test]
fn context_run_detail_works_over_stdio() {
    let repo_root = temp_repo();
    fs::write(
        repo_root.join("src.txt"),
        "alpha needle\nbeta needle\ngamma needle\n",
    )
    .expect("repo file should write");

    let sync_request = json_rpc_request(
        35,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        36,
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
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        assemble_request.len(),
        assemble_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("assembly should succeed");
    let responses = decode_responses(&output);
    let assembly: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("assembly text should exist"),
    )
    .expect("assembly payload should be valid json");

    let detail_request = json_rpc_request(
        37,
        "tools/call",
        json!({
            "name": "context_run_detail",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "generated_at_epoch_ms": assembly["generated_at_epoch_ms"]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        detail_request.len(),
        detail_request
    );
    let mut output = Vec::new();
    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("detail call should succeed");

    let detail_response = decode_response(&output);
    let detail: Value = serde_json::from_str(
        detail_response["result"]["content"][0]["text"]
            .as_str()
            .expect("detail text should exist"),
    )
    .expect("detail payload should be valid json");
    assert_eq!(
        detail["generated_at_epoch_ms"],
        assembly["generated_at_epoch_ms"]
    );
    assert_eq!(detail["snippets"][0]["reason"]["kind"], "query_line_match");
    assert_eq!(detail["omissions"][0]["kind"], "item_limit_reached");
}

fn decode_response(output: &[u8]) -> Value {
    let response = String::from_utf8(output.to_vec()).expect("response should be utf8");
    let (_, body) = response
        .split_once("\r\n\r\n")
        .expect("response should be framed");
    serde_json::from_str(body).expect("response body should be valid json")
}

fn decode_responses(output: &[u8]) -> Vec<Value> {
    let mut remaining = output;
    let mut responses = Vec::new();

    while !remaining.is_empty() {
        let header_end = remaining
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("frame terminator should exist");
        let header = std::str::from_utf8(&remaining[..header_end]).expect("header should be utf8");
        let content_length = header
            .strip_prefix("Content-Length: ")
            .and_then(|value| value.parse::<usize>().ok())
            .expect("content length should parse");
        let body_start = header_end + 4;
        let body_end = body_start + content_length;
        responses.push(
            serde_json::from_slice(&remaining[body_start..body_end]).expect("body should be json"),
        );
        remaining = &remaining[body_end..];
    }

    responses
}

fn json_rpc_request(id: u64, method: &str, params: Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    })
    .to_string()
}

fn temp_repo() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("quotarelay-repo-{unique}"));
    fs::create_dir_all(&root).expect("temp repo should create");
    root
}
