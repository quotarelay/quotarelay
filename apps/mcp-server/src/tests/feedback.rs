use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_responses, json_rpc_request, temp_repo};
use crate::run_stdio;

#[test]
fn context_feedback_tools_work_over_stdio() {
    let root = temp_repo();
    let write_request = json_rpc_request(
        153,
        "tools/call",
        json!({
            "name": "context_feedback_write",
            "arguments": {
                "root": root.to_string_lossy(),
                "generated_at_epoch_ms": 42,
                "rating": "useful",
                "reason": "small and relevant"
            }
        }),
    );
    let list_request = json_rpc_request(
        154,
        "tools/call",
        json!({
            "name": "context_feedback_list",
            "arguments": {
                "root": root.to_string_lossy(),
                "limit": 5
            }
        }),
    );
    let framed = format!(
        "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
        write_request.len(),
        write_request,
        list_request.len(),
        list_request
    );
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("feedback tools should succeed");

    let responses = decode_responses(&output);
    let written: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("write text should exist"),
    )
    .expect("write payload should parse");
    let listed: Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .expect("list text should exist"),
    )
    .expect("list payload should parse");

    assert_eq!(written["feedback"]["rating"], "useful");
    assert_eq!(listed["feedback"][0]["reason"], "small and relevant");
}

#[test]
fn local_cli_feedback_write_and_list_return_stable_json() {
    let root = temp_repo();
    let mut output = Vec::new();
    super::run_cli(
        [
            "feedback-write".to_string(),
            root.to_string_lossy().to_string(),
            "42".to_string(),
            "not_useful".to_string(),
            "missed the main file".to_string(),
        ],
        &mut output,
    )
    .expect("cli feedback write should succeed");
    let written: Value = serde_json::from_slice(&output).expect("write payload should parse");
    assert_eq!(written["ok"], true);
    assert_eq!(written["command"], "feedback-write");

    output.clear();
    super::run_cli(
        [
            "feedback-list".to_string(),
            root.to_string_lossy().to_string(),
            "5".to_string(),
        ],
        &mut output,
    )
    .expect("cli feedback list should succeed");
    let listed: Value = serde_json::from_slice(&output).expect("list payload should parse");
    assert_eq!(listed["ok"], true);
    assert_eq!(
        listed["result"]["feedback"]["feedback"][0]["rating"],
        "not_useful"
    );
}
