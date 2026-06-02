use std::io::{self, BufRead, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{routing::get, Json, Router};
use context_engine::{
    context_run_history, list_registered_repositories, memory_read, memory_search,
    memory_write, register_repository, registered_repository_state, remove_registered_repository,
    ContextAssembly, EngineInfo, MemoryNote, MemorySearchResult, MemoryWriteResult,
    RegisteredRepository, RegisteredRepositoryState, RepositoryRegistrationResult,
    RepositoryRemovalResult, RetrievalMode, RetrievedContext, RetrievalTruth,
    retrieve_context, retrieval_truth,
};
use repo_index::{repo_inventory, search_code, sync_repo};
use serde_json::{json, Value};

#[derive(Debug, Clone, serde::Serialize)]
struct BackendTruthPayload {
    tools: Vec<Value>,
    retrieval: RetrievalTruth,
    proofs: Vec<BackendProof>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct BackendProof {
    id: &'static str,
    command: &'static str,
}

pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_stdio(stdin.lock(), stdout.lock())
}

pub async fn run_http(addr: SocketAddr) -> io::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(io::Error::other)?;
    axum::serve(listener, http_router())
        .await
        .map_err(io::Error::other)
}

pub fn http_router() -> Router {
    Router::new().route("/truth", get(truth_handler))
}

async fn truth_handler() -> Json<BackendTruthPayload> {
    Json(backend_truth_payload())
}

fn backend_truth_payload() -> BackendTruthPayload {
    BackendTruthPayload {
        tools: current_tool_registry(),
        retrieval: retrieval_truth(),
        proofs: vec![
            BackendProof {
                id: "tools_list",
                command: "cargo test -p mcp-server tools_list_exposes_current_backend_truth_surface",
            },
            BackendProof {
                id: "memory_tools",
                command: "cargo test -p mcp-server durable_memory_tools_work_over_stdio",
            },
            BackendProof {
                id: "memory_aware_context",
                command: "cargo test -p mcp-server assemble_context_includes_matching_memory_over_stdio",
            },
            BackendProof {
                id: "registered_repositories",
                command: "cargo test -p mcp-server repository_registration_tools_work_over_stdio",
            },
            BackendProof {
                id: "repository_state",
                command: "cargo test -p mcp-server repository_state_tool_reports_sync_and_recent_run_truth_over_stdio",
            },
        ],
    }
}

fn current_tool_registry() -> Vec<Value> {
    vec![
        bootstrap_tool(),
        sync_repo_tool(),
        repo_inventory_tool(),
        register_repository_tool(),
        list_repositories_tool(),
        repository_state_tool(),
        remove_repository_tool(),
        search_code_tool(),
        memory_write_tool(),
        memory_read_tool(),
        memory_search_tool(),
        context_run_history_tool(),
        assemble_context_tool(),
    ]
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
                "tools": current_tool_registry()
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

fn register_repository_tool() -> Value {
    json!({
        "name": "register_repository",
        "description": "Registers a repository root in bounded local Quotarelay state.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "repo_root": { "type": "string" }
            },
            "required": ["root", "repo_root"],
            "additionalProperties": false
        }
    })
}

fn list_repositories_tool() -> Value {
    json!({
        "name": "list_repositories",
        "description": "Lists registered repositories from bounded local Quotarelay state.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 20 }
            },
            "required": ["root"],
            "additionalProperties": false
        }
    })
}

fn repository_state_tool() -> Value {
    json!({
        "name": "repository_state",
        "description": "Returns bounded registered repository sync state and recent run truth.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 20 }
            },
            "required": ["root"],
            "additionalProperties": false
        }
    })
}

fn remove_repository_tool() -> Value {
    json!({
        "name": "remove_repository",
        "description": "Removes a repository root from bounded local Quotarelay state.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "repo_root": { "type": "string" }
            },
            "required": ["root", "repo_root"],
            "additionalProperties": false
        }
    })
}

fn memory_write_tool() -> Value {
    json!({
        "name": "memory_write",
        "description": "Persists a durable memory note under the local Quotarelay state directory.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "title": { "type": "string" },
                "content": { "type": "string" },
                "tags": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["root", "title", "content"],
            "additionalProperties": false
        }
    })
}

fn memory_read_tool() -> Value {
    json!({
        "name": "memory_read",
        "description": "Reads one durable memory note by id from the local Quotarelay state directory.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "id": { "type": "string" }
            },
            "required": ["root", "id"],
            "additionalProperties": false
        }
    })
}

fn memory_search_tool() -> Value {
    json!({
        "name": "memory_search",
        "description": "Searches durable memory notes with an explicit result limit.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "query": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 5 }
            },
            "required": ["root", "query"],
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
        Some("memory_write") => match memory_write_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("memory_read") => match memory_read_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("memory_search") => match memory_search_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
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
        Some("register_repository") => match register_repository_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("list_repositories") => match list_repositories_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("repository_state") => match repository_state_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("remove_repository") => match remove_repository_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
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

fn register_repository_from_args(
    arguments: &Value,
) -> Result<RepositoryRegistrationResult, String> {
    let root = parse_root(arguments)?;
    let repo_root = parse_repo_root(arguments)?;
    register_repository(&root, &repo_root)
        .map_err(|error| format!("register_repository failed: {error}"))
}

fn list_repositories_from_args(arguments: &Value) -> Result<Vec<RegisteredRepository>, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(10);

    list_registered_repositories(&root, limit)
        .map_err(|error| format!("list_repositories failed: {error}"))
}

fn repository_state_from_args(
    arguments: &Value,
) -> Result<Vec<RegisteredRepositoryState>, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(10);

    registered_repository_state(&root, limit)
        .map_err(|error| format!("repository_state failed: {error}"))
}

fn remove_repository_from_args(arguments: &Value) -> Result<RepositoryRemovalResult, String> {
    let root = parse_root(arguments)?;
    let repo_root = parse_repo_root(arguments)?;
    remove_registered_repository(&root, &repo_root)
        .map_err(|error| format!("remove_repository failed: {error}"))
}

fn memory_write_from_args(arguments: &Value) -> Result<MemoryWriteResult, String> {
    let root = parse_root(arguments)?;
    let title = arguments
        .get("title")
        .and_then(Value::as_str)
        .ok_or_else(|| "memory_write requires a string title".to_string())?;
    let content = arguments
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| "memory_write requires a string content".to_string())?;
    let tags = arguments
        .get("tags")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    memory_write(&root, title, content, &tags)
        .map_err(|error| format!("memory_write failed: {error}"))
}

fn memory_read_from_args(arguments: &Value) -> Result<Option<MemoryNote>, String> {
    let root = parse_root(arguments)?;
    let id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "memory_read requires a string id".to_string())?;

    memory_read(&root, id).map_err(|error| format!("memory_read failed: {error}"))
}

fn memory_search_from_args(arguments: &Value) -> Result<MemorySearchResult, String> {
    let root = parse_root(arguments)?;
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| "memory_search requires a string query".to_string())?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(3);

    memory_search(&root, query, limit).map_err(|error| format!("memory_search failed: {error}"))
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

fn parse_repo_root(arguments: &Value) -> Result<PathBuf, String> {
    let root = arguments
        .get("repo_root")
        .and_then(Value::as_str)
        .ok_or_else(|| "tool requires a string repo_root".to_string())?;
    Ok(PathBuf::from(root))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::fs;

    use axum::body::Body;
    use axum::http::Request;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use super::{http_router, run_stdio};

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
                "register_repository",
                "list_repositories",
                "repository_state",
                "remove_repository",
                "search_code",
                "memory_write",
                "memory_read",
                "memory_search",
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
        assert_eq!(assembly["memory_notes"].as_array().map(|items| items.len()), Some(0));
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
    fn durable_memory_tools_work_over_stdio() {
        let repo_root = temp_repo();

        let write_request = json_rpc_request(
            18,
            "tools/call",
            json!({
                "name": "memory_write",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "title": "Design note",
                    "content": "Persistent memory should survive restarts.",
                    "tags": ["memory", "design"]
                }
            }),
        );
        let framed = format!("Content-Length: {}\r\n\r\n{}", write_request.len(), write_request);
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_write should succeed");

        let write_response = decode_response(&output);
        let created: Value = serde_json::from_str(
            write_response["result"]["content"][0]["text"]
                .as_str()
                .expect("memory write text should exist"),
        )
        .expect("memory write payload should be valid json");
        let note_id = created["note"]["id"].as_str().expect("note id should exist");

        let read_request = json_rpc_request(
            19,
            "tools/call",
            json!({
                "name": "memory_read",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "id": note_id
                }
            }),
        );
        let search_request = json_rpc_request(
            20,
            "tools/call",
            json!({
                "name": "memory_search",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "query": "memory",
                    "limit": 2
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            read_request.len(),
            read_request,
            search_request.len(),
            search_request
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("memory read and search should succeed");

        let responses = decode_responses(&output);
        let loaded: Value = serde_json::from_str(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .expect("memory read text should exist"),
        )
        .expect("memory read payload should be valid json");
        assert_eq!(loaded["title"], "Design note");

        let search_results: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("memory search text should exist"),
        )
        .expect("memory search payload should be valid json");
        assert_eq!(search_results["notes"].as_array().map(|items| items.len()), Some(1));
        assert_eq!(search_results["notes"][0]["title"], "Design note");
    }

    #[test]
    fn repository_registration_tools_work_over_stdio() {
        let state_root = temp_repo();
        let repo_root = temp_repo();

        let register_request = json_rpc_request(
            24,
            "tools/call",
            json!({
                "name": "register_repository",
                "arguments": {
                    "root": state_root.to_string_lossy(),
                    "repo_root": repo_root.to_string_lossy()
                }
            }),
        );
        let list_request = json_rpc_request(
            25,
            "tools/call",
            json!({
                "name": "list_repositories",
                "arguments": {
                    "root": state_root.to_string_lossy(),
                    "limit": 5
                }
            }),
        );
        let remove_request = json_rpc_request(
            26,
            "tools/call",
            json!({
                "name": "remove_repository",
                "arguments": {
                    "root": state_root.to_string_lossy(),
                    "repo_root": repo_root.to_string_lossy()
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            register_request.len(),
            register_request,
            list_request.len(),
            list_request,
            remove_request.len(),
            remove_request
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("repository registration tools should succeed");

        let responses = decode_responses(&output);
        let registered: Value = serde_json::from_str(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .expect("register text should exist"),
        )
        .expect("registered payload should be valid json");
        let listed: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("list text should exist"),
        )
        .expect("list payload should be valid json");
        let removed: Value = serde_json::from_str(
            responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("remove text should exist"),
        )
        .expect("remove payload should be valid json");

        assert_eq!(listed.as_array().map(|items| items.len()), Some(1));
        assert_eq!(listed[0]["root"], registered["repository"]["root"]);
        assert_eq!(removed["repository"]["root"], registered["repository"]["root"]);
    }

    #[test]
    fn repository_state_tool_reports_sync_and_recent_run_truth_over_stdio() {
        let state_root = temp_repo();
        let repo_root = temp_repo();
        fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

        let register_request = json_rpc_request(
            27,
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
            28,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let assemble_request = json_rpc_request(
            29,
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
        let state_request = json_rpc_request(
            30,
            "tools/call",
            json!({
                "name": "repository_state",
                "arguments": {
                    "root": state_root.to_string_lossy(),
                    "limit": 5
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            register_request.len(),
            register_request,
            sync_request.len(),
            sync_request,
            assemble_request.len(),
            assemble_request,
            state_request.len(),
            state_request
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("repository_state should succeed");

        let responses = decode_responses(&output);
        let state: Value = serde_json::from_str(
            responses[3]["result"]["content"][0]["text"]
                .as_str()
                .expect("repository_state text should exist"),
        )
        .expect("repository_state payload should be valid json");

        assert_eq!(state.as_array().map(|items| items.len()), Some(1));
        assert_eq!(state[0]["sync"]["status"], "indexed");
        assert_eq!(state[0]["sync"]["indexed_files"], 1);
        assert_eq!(state[0]["recent_context_run"]["query"], "needle");
    }

    #[test]
    fn assemble_context_includes_matching_memory_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

        let sync_request = json_rpc_request(
            21,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let write_request = json_rpc_request(
            22,
            "tools/call",
            json!({
                "name": "memory_write",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "title": "Needle note",
                    "content": "needle should appear in assembled context memory",
                    "tags": ["memory"]
                }
            }),
        );
        let assemble_request = json_rpc_request(
            23,
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
            sync_request.len(),
            sync_request,
            write_request.len(),
            write_request,
            assemble_request.len(),
            assemble_request
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("memory-aware assemble_context should succeed");

        let responses = decode_responses(&output);
        let assembly: Value = serde_json::from_str(
            responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("assembly text should exist"),
        )
        .expect("assembly payload should be valid json");
        assert_eq!(assembly["memory_notes"].as_array().map(|items| items.len()), Some(1));
        assert_eq!(assembly["memory_notes"][0]["reason"]["kind"], "memory_note_match");
        assert_eq!(assembly["memory_notes"][0]["title"], "Needle note");
    }

    #[tokio::test]
    async fn backend_truth_endpoint_exposes_current_contract() {
        let response = http_router()
            .oneshot(Request::builder().uri("/truth").body(Body::empty()).expect("request should build"))
            .await
            .expect("truth endpoint should respond");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        let payload: Value = serde_json::from_slice(&body).expect("truth payload should be valid json");

        assert_eq!(payload["tools"].as_array().map(|items| items.len()), Some(13));
        assert_eq!(payload["retrieval"]["modes"], json!(["exact_search", "overview", "task_capsule"]));
        assert_eq!(payload["retrieval"]["limits"]["max_context_items"], 5);
        assert_eq!(payload["retrieval"]["durable_memory_enabled"], true);
        assert!(payload["proofs"].as_array().map(|items| items.len()).unwrap_or_default() >= 5);
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
