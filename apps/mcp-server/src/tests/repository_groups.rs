use std::fs;
use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_responses, json_rpc_request, temp_repo};
use crate::run_stdio;

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
fn team_policy_profiles_save_list_and_inspect_over_stdio() {
    let state_root = temp_repo();

    let save_request = json_rpc_request(
        46,
        "tools/call",
        json!({
            "name": "team_policy_profile_save",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "name": "backend-team",
                "guardrails": ["keep source local by default"],
                "validation_recipes": ["cargo test -p context-engine"],
                "mcp_client_presets": ["examples/mcp-client-presets/generic-stdio.json"]
            }
        }),
    );
    let list_request = json_rpc_request(
        47,
        "tools/call",
        json!({
            "name": "team_policy_profile_list",
            "arguments": {
                "root": state_root.to_string_lossy(),
                "limit": 10
            }
        }),
    );
    let inspect_request = json_rpc_request(
        48,
        "tools/call",
        json!({
            "name": "inspect_local_state",
            "arguments": {
                "root": state_root.to_string_lossy()
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        save_request.len(),
        save_request,
        list_request.len(),
        list_request,
        inspect_request.len(),
        inspect_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("team policy profile calls should succeed");

    let responses = decode_responses(&output);
    let saved: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("team policy profile save text should exist"),
    )
    .expect("team policy profile save payload should be valid json");
    let listed: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("team policy profile list text should exist"),
    )
    .expect("team policy profile list payload should be valid json");
    let inspected: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("local state inspection text should exist"),
    )
    .expect("local state inspection payload should be valid json");

    assert_eq!(saved["profile"]["name"], "backend-team");
    assert_eq!(saved["profile"]["allow_source_upload"], false);
    assert_eq!(listed.as_array().map(Vec::len), Some(1));
    assert_eq!(listed[0]["guardrails"].as_array().map(Vec::len), Some(1));
    assert_eq!(inspected["team_policy_profiles"]["present"], true);
    assert_eq!(inspected["team_policy_profiles"]["item_count"], 1);
    assert!(inspected.get("guardrails").is_none());
}
