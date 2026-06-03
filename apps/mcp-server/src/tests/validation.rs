use std::io::Cursor;

use serde_json::{json, Value};

use super::common::{decode_response, json_rpc_request};
use crate::run_stdio;

#[test]
fn validation_recommend_tool_works_over_stdio() {
    let request = json_rpc_request(
        152,
        "tools/call",
        json!({
            "name": "validation_recommend",
            "arguments": {
                "paths": [
                    "crates/context-engine/src/lib.rs",
                    "web/controlplane/src/truth.ts"
                ]
            }
        }),
    );
    let framed = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
    let mut output = Vec::new();

    run_stdio(Cursor::new(framed.into_bytes()), &mut output)
        .expect("validation recommendation should succeed");

    let response = decode_response(&output);
    let result: Value = serde_json::from_str(
        response["result"]["content"][0]["text"]
            .as_str()
            .expect("text should exist"),
    )
    .expect("recommendation payload should parse");
    let commands = result["recommendations"]
        .as_array()
        .expect("recommendations should exist")
        .iter()
        .map(|recommendation| recommendation["command"].as_str().unwrap_or(""))
        .collect::<Vec<_>>();

    assert!(commands.contains(&"cargo test -p context-engine"));
    assert!(commands.contains(&"npm --prefix web/controlplane run build"));
}

#[test]
fn local_cli_validate_returns_stable_json_contract() {
    let mut output = Vec::new();
    super::run_cli(
        [
            "validate".to_string(),
            "apps/mcp-server/src/tools.rs".to_string(),
            "docs/MVP_SCOPE.md".to_string(),
        ],
        &mut output,
    )
    .expect("cli validate should succeed");

    let payload: Value = serde_json::from_slice(&output).expect("payload should parse");
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["command"], "validate");
    assert!(payload["result"]["validation"]["recommendations"]
        .as_array()
        .expect("recommendations should exist")
        .iter()
        .any(|recommendation| recommendation["command"] == "cargo test -p mcp-server"));
}
