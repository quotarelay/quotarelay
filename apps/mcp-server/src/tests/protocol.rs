use std::io::Cursor;

use serde_json::{json, Value};

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
            "repo_map",
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
            "team_policy_profile_save",
            "team_policy_profile_list",
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
            "context_feedback_write",
            "context_feedback_list",
            "validation_recommend",
            "savings_report",
            "onboarding_pack",
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

    let handoff_tool = tools
        .iter()
        .find(|tool| tool["name"] == "handoff_packet")
        .expect("handoff_packet tool should exist");
    assert_eq!(
        handoff_tool["inputSchema"]["properties"]["template"]["enum"],
        json!([
            "general",
            "bug_fix",
            "feature_slice",
            "review",
            "refactor",
            "release"
        ])
    );
}

#[test]
fn tools_list_schemas_are_closed_and_have_stable_required_fields() {
    let request = r#"{"jsonrpc":"2.0","id":13,"method":"tools/list","params":{}}"#;
    let framed = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("tools list should succeed");

    let response = decode_response(&output);
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools should be an array");
    let expected_required: &[(&str, &[&str])] = &[
        ("bootstrap_status", &[]),
        ("sync_repo", &["root"]),
        ("repo_inventory", &["root"]),
        ("repo_map", &["root"]),
        ("inspect_local_state", &["root"]),
        ("cache_inspect", &["root"]),
        ("cache_clear", &["root"]),
        ("register_repository", &["root", "repo_root"]),
        ("list_repositories", &["root"]),
        ("repository_state", &["root"]),
        ("repository_detail", &["root", "repo_root"]),
        ("repository_update_metadata", &["root", "repo_root", "name"]),
        ("remove_repository", &["root", "repo_root"]),
        ("workspace_profile_save", &["root", "name", "repo_roots"]),
        ("workspace_profile_list", &["root"]),
        ("team_policy_profile_save", &["root", "name"]),
        ("team_policy_profile_list", &["root"]),
        ("search_code", &["root", "query"]),
        ("memory_write", &["root", "title", "content"]),
        ("memory_read", &["root", "id"]),
        ("memory_update", &["root", "id"]),
        ("memory_delete", &["root", "id"]),
        ("memory_export", &["root"]),
        ("memory_import", &["root", "payload"]),
        ("memory_search", &["root", "query"]),
        ("context_run_detail", &["root", "generated_at_epoch_ms"]),
        ("context_run_history", &["root"]),
        (
            "context_feedback_write",
            &["root", "generated_at_epoch_ms", "rating", "reason"],
        ),
        ("context_feedback_list", &["root"]),
        ("validation_recommend", &["paths"]),
        ("savings_report", &["root"]),
        ("onboarding_pack", &["root"]),
        (
            "multi_repo_assemble_context",
            &["root", "repo_roots", "query"],
        ),
        ("assemble_context", &["root"]),
        ("handoff_packet", &["root", "active_task"]),
    ];

    for tool in tools {
        let name = tool["name"].as_str().expect("tool name should exist");
        let description = tool["description"]
            .as_str()
            .expect("tool description should exist");
        assert!(
            description.len() >= 20,
            "{name} should have a useful description"
        );

        let schema = &tool["inputSchema"];
        assert_eq!(schema["type"], "object", "{name} schema should be object");
        assert_eq!(
            schema["additionalProperties"], false,
            "{name} schema should reject undeclared fields"
        );

        let properties = schema["properties"]
            .as_object()
            .expect("schema properties should be an object");
        let required = required_fields(schema);
        for field in &required {
            assert!(
                properties.contains_key(*field),
                "{name} requires undeclared field {field}"
            );
        }

        let expected = expected_required
            .iter()
            .find(|(expected_name, _)| *expected_name == name)
            .map(|(_, fields)| *fields)
            .expect("tool required fields should be locked");
        assert_eq!(required, expected, "{name} required fields changed");
    }
}

fn required_fields(schema: &Value) -> Vec<&str> {
    schema
        .get("required")
        .and_then(Value::as_array)
        .map(|fields| {
            fields
                .iter()
                .map(|field| field.as_str().expect("required field should be string"))
                .collect()
        })
        .unwrap_or_default()
}
