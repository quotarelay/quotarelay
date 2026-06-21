use std::fs;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::body::Body;
use axum::http::Request;
use serde_json::{json, Value};
use tower::ServiceExt;

use context_engine::{assemble_context, memory_write, register_repository};

use super::common::{decode_responses, json_rpc_request, temp_repo};
use crate::{http_router, run_http, run_stdio};

async fn truth_payload() -> Value {
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
    serde_json::from_slice(&body).expect("truth payload should be valid json")
}

#[tokio::test]
async fn http_server_rejects_non_loopback_bind_addresses() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
    let error = run_http(addr)
        .await
        .expect_err("non-loopback bind should be rejected");

    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(error.to_string().contains("loopback"));
}

#[tokio::test]
async fn backend_truth_endpoint_exposes_current_contract() {
    let payload = truth_payload().await;

    assert_eq!(
        payload["tools"].as_array().map(|items| items.len()),
        Some(35)
    );
    assert_eq!(
        payload["retrieval"]["modes"],
        json!(["exact_search", "overview", "task_capsule", "diff_aware"])
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
    assert_eq!(payload["config"]["team_policy_profiles_enabled"], true);
    assert_eq!(payload["config"]["max_team_policy_profiles"], 20);
    assert_eq!(payload["config"]["max_team_policy_items"], 20);
    assert_eq!(
        payload["config"]["team_policy_source_upload_default"],
        false
    );
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
async fn backend_truth_contract_locks_shipped_fields_and_ids() {
    let payload = truth_payload().await;
    let mut top_level_fields = payload
        .as_object()
        .expect("truth payload should be an object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    top_level_fields.sort_unstable();

    assert_eq!(
        top_level_fields,
        vec!["cache", "cli", "config", "proofs", "retrieval", "tools"]
    );

    let tool_names = payload["tools"]
        .as_array()
        .expect("tools should be an array")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name should exist"))
        .collect::<Vec<_>>();
    assert_eq!(
        tool_names,
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

    assert_eq!(
        payload["retrieval"]["modes"],
        json!(["exact_search", "overview", "task_capsule", "diff_aware"])
    );
    assert_eq!(payload["retrieval"]["cache"], payload["cache"]);
    assert_eq!(
        payload["cache"],
        json!({
            "exact_search_enabled": true,
            "overview_enabled": true,
            "task_capsule_enabled": true,
            "sync_invalidates_caches": true
        })
    );
    assert_eq!(
        payload["config"],
        json!({
            "workspace_profiles_enabled": true,
            "workspace_profiles_apply_to_retrieval": false,
            "max_workspace_profiles": 20,
            "max_profile_repo_roots": 5,
            "team_policy_profiles_enabled": true,
            "max_team_policy_profiles": 20,
            "max_team_policy_items": 20,
            "team_policy_source_upload_default": false,
            "default_mode": "exact_search",
            "default_limit": 3,
            "per_repo_limit": 3
        })
    );
    assert_eq!(payload["cli"]["local_entrypoint_enabled"], true);
    assert_eq!(
        payload["cli"]["commands"][0],
        json!({
            "label": "Truth",
            "command": "cargo run -p mcp-server -- --cli truth"
        })
    );

    let proof_ids = payload["proofs"]
        .as_array()
        .expect("proofs should be an array")
        .iter()
        .map(|proof| proof["id"].as_str().expect("proof id should exist"))
        .collect::<Vec<_>>();
    assert_eq!(
        proof_ids,
        vec![
            "tools_list",
            "memory_tools",
            "memory_aware_context",
            "registered_repositories",
            "team_policy_profiles",
            "team_policy_cli",
            "savings_report",
            "onboarding_pack",
            "repository_state",
            "local_operator_workflow",
        ]
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

    let payload = truth_payload().await;

    assert!(payload["proofs"]
        .as_array()
        .expect("proofs should be an array")
        .iter()
        .any(|proof| proof["id"] == "local_operator_workflow"));
}
