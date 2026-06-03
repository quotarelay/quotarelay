use std::io::Cursor;

use serde_json::json;

use super::common::{decode_response, json_rpc_request, temp_repo};
use crate::run_stdio;

#[test]
fn memory_search_empty_query_reports_stable_error_over_stdio() {
    let repo_root = temp_repo();
    let search_request = json_rpc_request(
        48,
        "tools/call",
        json!({
            "name": "memory_search",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "query": "   ",
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
        .expect("empty memory search should produce an error response");

    let response = decode_response(&output);
    assert_eq!(
        response["result"]["content"][0]["text"],
        "memory_search failed: memory search requires a non-empty query"
    );
}

#[test]
fn memory_update_without_fields_reports_stable_error_over_stdio() {
    let repo_root = temp_repo();
    let write_request = json_rpc_request(
        49,
        "tools/call",
        json!({
            "name": "memory_write",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "title": "Design note",
                "content": "Memory content.",
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
    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory write should succeed");
    let created_response = decode_response(&output);
    let created: serde_json::Value = serde_json::from_str(
        created_response["result"]["content"][0]["text"]
            .as_str()
            .expect("memory write text should exist"),
    )
    .expect("memory write payload should parse");

    let update_request = json_rpc_request(
        50,
        "tools/call",
        json!({
            "name": "memory_update",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "id": created["note"]["id"]
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        update_request.len(),
        update_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("fieldless memory update should produce an error response");

    let response = decode_response(&output);
    assert_eq!(
        response["result"]["content"][0]["text"],
        "memory_update failed: memory update requires at least one field to change"
    );
}
