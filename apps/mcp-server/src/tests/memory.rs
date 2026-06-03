use std::fs;
use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_response, decode_responses, json_rpc_request, temp_repo};
use crate::run_stdio;

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
