use std::fs;
use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_response, decode_responses, json_rpc_request, temp_repo};
use crate::run_stdio;

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
    assert!(
        assembly["budget"]["raw_bytes_considered"]
            .as_u64()
            .expect("raw considered bytes should exist")
            >= assembly["budget"]["included_bytes"]
                .as_u64()
                .expect("included bytes should exist")
    );
    assert!(assembly["budget"]["estimated_reduction_ratio"].is_number());
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

#[test]
fn savings_report_works_over_stdio_without_snippets() {
    let repo_root = temp_repo();
    fs::write(repo_root.join("src.txt"), "alpha needle\nbeta needle\n")
        .expect("repo file should write");

    let sync_request = json_rpc_request(
        45,
        "tools/call",
        json!({
            "name": "sync_repo",
            "arguments": {
                "root": repo_root.to_string_lossy()
            }
        }),
    );
    let assemble_request = json_rpc_request(
        46,
        "tools/call",
        json!({
            "name": "assemble_context",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "mode": "exact_search",
                "query": "needle",
                "limit": 1
            }
        }),
    );
    let report_request = json_rpc_request(
        47,
        "tools/call",
        json!({
            "name": "savings_report",
            "arguments": {
                "root": repo_root.to_string_lossy(),
                "limit": 10
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        sync_request.len(),
        sync_request,
        assemble_request.len(),
        assemble_request,
        report_request.len(),
        report_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("report call should succeed");

    let responses = decode_responses(&output);
    assert_eq!(responses[2]["id"], 47);
    let report: Value = serde_json::from_str(
        responses[2]["result"]["content"][0]["text"]
            .as_str()
            .expect("report text should exist"),
    )
    .expect("report payload should be valid json");

    assert_eq!(report["run_count"], 1);
    assert!(report["raw_bytes_considered"].as_u64().unwrap_or(0) > 0);
    assert!(report.get("snippets").is_none());
    assert!(report["note"]
        .as_str()
        .unwrap_or("")
        .contains("not provider billing"));
}
