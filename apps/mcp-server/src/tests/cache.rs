use std::fs;
use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_responses, json_rpc_request, temp_repo};
use crate::run_stdio;

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
