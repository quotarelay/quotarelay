use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use context_engine::{
    context_run_history, retrieve_context, ContextAssembly, EngineInfo, RetrievalMode,
    RetrievedContext,
};
use repo_index::{repo_inventory, search_code, sync_repo};
use serde_json::{json, Value};

pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_stdio(stdin.lock(), stdout.lock())
}

fn run_stdio<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    while let Some(message) = read_message(&mut reader)? {
        if let Some(response) = handle_message(&message) {
            write_message(&mut writer, &response)?;
            writer.flush()?;
        }
    }

    Ok(())
}

fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut content_length = None;
    let mut saw_header = false;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }

        saw_header = true;
        let mut parts = trimmed.splitn(2, ':');
        let name = parts.next().unwrap_or_default().trim();
        let value = parts.next().unwrap_or_default().trim();

        if name.eq_ignore_ascii_case("Content-Length") {
            let parsed = value.parse::<usize>().map_err(|error| {
                io::Error::new(io::ErrorKind::InvalidData, format!("invalid content length: {error}"))
            })?;
            content_length = Some(parsed);
        }
    }

    if !saw_header {
        return Ok(None);
    }

    let len = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header")
    })?;

    let mut body = vec![0_u8; len];
    reader.read_exact(&mut body)?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn write_message<W: Write>(writer: &mut W, body: &str) -> io::Result<()> {
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)
}

fn handle_message(message: &str) -> Option<String> {
    let request: Value = serde_json::from_str(message).ok()?;
    let method = request.get("method")?.as_str()?;
    let id = request.get("id").cloned().unwrap_or(Value::Null);

    let response = match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "quotarelay",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {}
                }
            }
        }),
        "tools/list" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    bootstrap_tool(),
                    sync_repo_tool(),
                    repo_inventory_tool(),
                    search_code_tool(),
                    context_run_history_tool(),
                    assemble_context_tool()
                ]
            }
        }),
        "tools/call" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [bootstrap_tool_call(&request)]
            }
        }),
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": "method not found"
            }
        }),
    };

    Some(response.to_string())
}

fn bootstrap_tool() -> Value {
    json!({
        "name": "bootstrap_status",
        "description": "Returns the current Quotarelay bootstrap banner.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }
    })
}

fn sync_repo_tool() -> Value {
    json!({
        "name": "sync_repo",
        "description": "Scans a repository and persists a local search index under .quotarelay/index.json.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" }
            },
            "required": ["root"],
            "additionalProperties": false
        }
    })
}

fn search_code_tool() -> Value {
    json!({
        "name": "search_code",
        "description": "Searches the persisted local repository index with an explicit result limit.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "query": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 10 }
            },
            "required": ["root", "query"],
            "additionalProperties": false
        }
    })
}

fn repo_inventory_tool() -> Value {
    json!({
        "name": "repo_inventory",
        "description": "Returns bounded repository inventory truth from the persisted local repository index.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" }
            },
            "required": ["root"],
            "additionalProperties": false
        }
    })
}

fn context_run_history_tool() -> Value {
    json!({
        "name": "context_run_history",
        "description": "Returns bounded recent context runs with inclusion and omission reasons.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 5 }
            },
            "required": ["root"],
            "additionalProperties": false
        }
    })
}

fn assemble_context_tool() -> Value {
    json!({
        "name": "assemble_context",
        "description": "Builds a bounded context pack from the persisted local repository index using explicit retrieval modes.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "mode": {
                    "type": "string",
                    "enum": ["exact_search", "overview", "task_capsule"],
                    "default": "exact_search"
                },
                "query": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 5 }
            },
            "required": ["root"],
            "additionalProperties": false
        }
    })
}

fn bootstrap_tool_call(request: &Value) -> Value {
    let info = EngineInfo::quotarelay();
    let tool_name = request
        .get("params")
        .and_then(|params| params.get("name"))
        .and_then(Value::as_str);
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .cloned()
        .unwrap_or_else(|| json!({}));

    match tool_name {
        Some("bootstrap_status") => json!({
            "type": "text",
            "text": info.banner()
        }),
        Some("sync_repo") => match sync_repo_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": format!("synced {} files into {}", result.indexed_files, result.index_path)
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("search_code") => match search_code_from_args(&arguments) {
            Ok(results) => json!({
                "type": "text",
                "text": serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("repo_inventory") => match repo_inventory_from_args(&arguments) {
            Ok(inventory) => json!({
                "type": "text",
                "text": serde_json::to_string(&inventory).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("context_run_history") => match context_run_history_from_args(&arguments) {
            Ok(history) => json!({
                "type": "text",
                "text": serde_json::to_string(&history).unwrap_or_else(|_| "[]".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("assemble_context") => match assemble_context_from_args(&arguments) {
            Ok(results) => json!({
                "type": "text",
                "text": serde_json::to_string(&results).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some(name) => json!({
            "type": "text",
            "text": format!("unknown tool: {name}")
        }),
        None => json!({
            "type": "text",
            "text": "missing tool name"
        }),
    }
}

fn sync_repo_from_args(arguments: &Value) -> Result<repo_index::SyncResult, String> {
    let root = parse_root(arguments)?;
    sync_repo(&root).map_err(|error| format!("sync_repo failed: {error}"))
}

fn search_code_from_args(arguments: &Value) -> Result<Vec<repo_index::SearchHit>, String> {
    let root = parse_root(arguments)?;
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| "search_code requires a string query".to_string())?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(5);

    search_code(&root, query, limit).map_err(|error| format!("search_code failed: {error}"))
}

fn repo_inventory_from_args(arguments: &Value) -> Result<repo_index::RepoInventory, String> {
    let root = parse_root(arguments)?;
    repo_inventory(&root).map_err(|error| format!("repo_inventory failed: {error}"))
}

fn context_run_history_from_args(arguments: &Value) -> Result<Vec<ContextAssembly>, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(3);

    context_run_history(&root, limit).map_err(|error| format!("context_run_history failed: {error}"))
}

fn assemble_context_from_args(arguments: &Value) -> Result<RetrievedContext, String> {
    let root = parse_root(arguments)?;
    let mode = parse_retrieval_mode(arguments)?;
    let query = arguments.get("query").and_then(Value::as_str);
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(3);

    retrieve_context(&root, mode, query, limit)
        .map_err(|error| format!("assemble_context failed: {error}"))
}

fn parse_retrieval_mode(arguments: &Value) -> Result<RetrievalMode, String> {
    match arguments.get("mode").and_then(Value::as_str) {
        None => Ok(RetrievalMode::ExactSearch),
        Some("exact_search") => Ok(RetrievalMode::ExactSearch),
        Some("overview") => Ok(RetrievalMode::Overview),
        Some("task_capsule") => Ok(RetrievalMode::TaskCapsule),
        Some(other) => Err(format!(
            "assemble_context mode must be one of exact_search, overview, task_capsule; got {other}"
        )),
    }
}

fn parse_root(arguments: &Value) -> Result<PathBuf, String> {
    let root = arguments
        .get("root")
        .and_then(Value::as_str)
        .ok_or_else(|| "tool requires a string root".to_string())?;
    Ok(PathBuf::from(root))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::fs;

    use serde_json::{json, Value};

    use super::run_stdio;

    #[test]
    fn responds_to_initialize_over_stdio() {
        let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let framed = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("stdio transport should succeed");

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
        assert_eq!(response["result"]["content"][0]["text"], "quotarelay bootstrap");
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
                "search_code",
                "context_run_history",
                "assemble_context",
            ]
        );

        let assemble_tool = tools
            .iter()
            .find(|tool| tool["name"] == "assemble_context")
            .expect("assemble_context tool should exist");
        assert_eq!(
            assemble_tool["inputSchema"]["properties"]["mode"]["enum"],
            json!(["exact_search", "overview", "task_capsule"])
        );
    }

    #[test]
    fn sync_and_search_work_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "bootstrap marker\nsearch target\n").expect("repo file should write");

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

        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("sync and search should succeed");

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
        fs::write(repo_root.join("src.txt"), "bootstrap marker\nsearch target\n").expect("repo file should write");

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

        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("inventory call should succeed");

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
        fs::write(repo_root.join("src.txt"), "alpha needle\nbeta needle\ngamma needle\n").expect("repo file should write");

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

        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("assemble_context should succeed");

        let responses = decode_responses(&output);
        assert_eq!(responses[1]["id"], 6);
        let assembly: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("assemble text should exist"),
        )
        .expect("assembly payload should be valid json");
        assert_eq!(assembly["mode"], "exact_search");
        assert_eq!(assembly["snippets"].as_array().map(|items| items.len()), Some(2));
        assert_eq!(assembly["snippets"][0]["reason"]["kind"], "query_line_match");
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
        assert_eq!(overview["documents"].as_array().map(|items| items.len()), Some(1));

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
    fn assemble_context_reports_byte_budget_omissions_over_stdio() {
        let repo_root = temp_repo();
        let long_line = format!("needle {}", "x".repeat(300));
        fs::write(repo_root.join("alpha.txt"), format!("{long_line}\n")).expect("repo file should write");

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
        assert!(assembly["snippets"][0]["line"]
            .as_str()
            .expect("snippet line should exist")
            .ends_with("..."));
    }

    #[test]
    fn context_run_history_works_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "alpha needle\nbeta needle\ngamma needle\n").expect("repo file should write");

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
        assert_eq!(history[0]["snippets"][0]["reason"]["kind"], "query_line_match");
        assert_eq!(history[0]["omissions"][0]["kind"], "item_limit_reached");
    }

    fn decode_response(output: &[u8]) -> Value {
        let response = String::from_utf8(output.to_vec()).expect("response should be utf8");
        let (_, body) = response.split_once("\r\n\r\n").expect("response should be framed");
        serde_json::from_str(body).expect("response body should be valid json")
    }

    fn decode_responses(output: &[u8]) -> Vec<Value> {
        let mut remaining = output;
        let mut responses = Vec::new();

        while !remaining.is_empty() {
            let header_end = remaining
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .expect("frame terminator should exist");
            let header = std::str::from_utf8(&remaining[..header_end]).expect("header should be utf8");
            let content_length = header
                .strip_prefix("Content-Length: ")
                .and_then(|value| value.parse::<usize>().ok())
                .expect("content length should parse");
            let body_start = header_end + 4;
            let body_end = body_start + content_length;
            responses.push(serde_json::from_slice(&remaining[body_start..body_end]).expect("body should be json"));
            remaining = &remaining[body_end..];
        }

        responses
    }

    fn json_rpc_request(id: u64, method: &str, params: Value) -> String {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        })
        .to_string()
    }

    fn temp_repo() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("quotarelay-repo-{unique}"));
        fs::create_dir_all(&root).expect("temp repo should create");
        root
    }
}
