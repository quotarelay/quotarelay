use std::fs;
use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_response, json_rpc_request, temp_repo};
use crate::run_stdio;

#[test]
fn assemble_context_returns_clarification_for_broad_request_over_stdio() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
    repo_index::sync_repo(&repo_root).expect("sync should succeed");

    let request = json_rpc_request(
        158,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "exact_search",
                "query": "fix",
                "limit": 3
            }
        }),
    );
    let framed = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("broad request should clarify over stdio");

    let response = decode_response(&output);
    let context: Value = serde_json::from_str(
        response["result"]["content"][0]["text"]
            .as_str()
            .expect("context text should exist"),
    )
    .expect("context payload should parse");

    assert_eq!(context["cache_status"]["kind"], "not_applicable");
    assert!(
        context["clarification"]["questions"]
            .as_array()
            .expect("questions should exist")
            .len()
            <= 3
    );
    assert_eq!(context["snippets"].as_array().map(Vec::len), Some(0));
}

#[test]
fn handoff_packet_returns_clarification_for_missing_query_over_stdio() {
    let root = temp_repo();
    fs::write(root.join("src.txt"), "needle handoff\n").expect("repo file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let handoff_request = json_rpc_request(
        159,
        "tools/call",
        json!({
            "name": "handoff_packet",
            "arguments": {
                "root": root.to_string_lossy(),
                "active_task": "Continue work",
                "mode": "exact_search",
                "limit": 2
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}",
        handoff_request.len(),
        handoff_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("handoff clarification should succeed");

    let response = decode_response(&output);
    let packet: Value = serde_json::from_str(
        response["result"]["content"][0]["text"]
            .as_str()
            .expect("handoff text should exist"),
    )
    .expect("handoff payload should be valid json");

    assert_eq!(packet["context"]["cache_status"]["kind"], "not_applicable");
    assert!(
        packet["context"]["clarification"]["questions"]
            .as_array()
            .expect("questions should exist")
            .len()
            <= 3
    );
    assert_eq!(
        packet["context"]["snippets"].as_array().map(Vec::len),
        Some(0)
    );
}
