use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

pub(super) fn decode_response(output: &[u8]) -> Value {
    let responses = decode_responses(output);
    responses
        .into_iter()
        .next()
        .expect("one response should be emitted")
}

pub(super) fn decode_responses(output: &[u8]) -> Vec<Value> {
    let text = String::from_utf8(output.to_vec()).expect("response should be utf8");
    text.split("Content-Length: ")
        .filter_map(|chunk| chunk.split_once("\r\n\r\n").map(|(_, body)| body))
        .map(|body| serde_json::from_str(body).expect("response body should be json"))
        .collect()
}

pub(super) fn json_rpc_request(id: u64, method: &str, params: Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    })
    .to_string()
}

pub(super) fn temp_repo() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("quotarelay-repo-{unique}"));
    std::fs::create_dir_all(&root).expect("temp repo should create");
    root
}
