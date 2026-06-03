use std::fs;
use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_response, decode_responses, json_rpc_request, temp_repo};
use crate::run_stdio;

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
fn assemble_context_supports_diff_aware_mode_over_stdio() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "needle original\n").expect("repo file should write");
    repo_index::sync_repo(&repo_root).expect("sync should succeed");
    fs::write(repo_root.join("src.txt"), "needle changed\n").expect("repo file should rewrite");

    let request = json_rpc_request(
        82,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "diff_aware",
                "query": "needle",
                "limit": 2
            }
        }),
    );
    let framed = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("diff-aware assemble should succeed");

    let response = decode_response(&output);
    let payload: Value = serde_json::from_str(
        response["result"]["content"][0]["text"]
            .as_str()
            .expect("diff-aware text should exist"),
    )
    .expect("diff-aware payload should be valid json");
    assert_eq!(payload["mode"], "diff_aware");
    assert_eq!(payload["stale"]["is_stale"], true);
    assert_eq!(
        payload["documents"][0]["reason"]["kind"],
        "diff_changed_file"
    );
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
fn handoff_packet_works_over_stdio() {
    let root = temp_repo();
    fs::write(root.join("src.txt"), "needle handoff\n").expect("repo file should write");
    let sync_request = json_rpc_request(
        80,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": root.to_string_lossy()
            }
        }),
    );
    let handoff_request = json_rpc_request(
        81,
        "tools/call",
        json!({
            "name": "handoff_packet",
            "arguments": {
                "root": root.to_string_lossy(),
                "active_task": "Continue T102",
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
        handoff_request.len(),
        handoff_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("handoff should succeed");

    let responses = decode_responses(&output);
    assert_eq!(responses[1]["id"], 81);
    let packet: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("handoff text should exist"),
    )
    .expect("handoff payload should be valid json");

    assert_eq!(packet["active_task"], "Continue T102");
    assert_eq!(packet["context"]["mode"], "exact_search");
    assert_eq!(
        packet["context"]["snippets"]
            .as_array()
            .map(|items| items.len()),
        Some(1)
    );
    assert!(
        packet["context"]["budget"]["raw_bytes_considered"]
            .as_u64()
            .expect("raw considered bytes should be exposed")
            >= packet["context"]["budget"]["included_bytes"]
                .as_u64()
                .expect("included bytes should be exposed")
    );
    assert!(packet["context"]["budget"]["estimated_reduction_ratio"].is_number());
    assert!(packet["validation_commands"].is_array());
    assert!(packet["known_blockers"].is_array());
    assert!(packet["omissions"].is_array());
}
