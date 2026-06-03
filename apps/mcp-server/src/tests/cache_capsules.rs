use std::fs;
use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_responses, json_rpc_request, temp_repo};
use crate::run_stdio;

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

    assert_eq!(first_overview["cache_status"]["kind"], "miss");
    assert_eq!(second_overview["cache_status"]["kind"], "hit");
    assert_eq!(first_overview["documents"], second_overview["documents"]);
    assert_eq!(first_overview["omissions"], second_overview["omissions"]);
    assert_eq!(first_task["cache_status"]["kind"], "miss");
    assert_eq!(second_task["cache_status"]["kind"], "hit");
    assert_eq!(first_task["documents"], second_task["documents"]);
    assert_eq!(first_task["omissions"], second_task["omissions"]);
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

    assert_eq!(refreshed_overview["cache_status"]["kind"], "miss");
    assert_eq!(cached_overview["cache_status"]["kind"], "hit");
    assert_eq!(
        refreshed_overview["documents"],
        cached_overview["documents"]
    );
    assert_eq!(
        refreshed_overview["omissions"],
        cached_overview["omissions"]
    );
    assert_eq!(refreshed_task["cache_status"]["kind"], "miss");
    assert_eq!(cached_task["cache_status"]["kind"], "hit");
    assert_eq!(refreshed_task["documents"], cached_task["documents"]);
    assert_eq!(refreshed_task["omissions"], cached_task["omissions"]);
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
