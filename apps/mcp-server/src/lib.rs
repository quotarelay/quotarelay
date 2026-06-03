use std::io::{self, BufRead, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use context_engine::{
    context_run_history, memory_search, registered_repository_state, retrieval_truth,
    ContextAssembly, MemorySearchResult, RegisteredRepositoryState, RetrievalTruth,
};
use serde::Deserialize;
use serde_json::{json, Value};

mod args;
mod cli;
mod tool_calls;
mod tools;

pub use cli::run_cli;
use tool_calls::bootstrap_tool_call;
use tools::current_tool_registry;

#[derive(Debug, Clone, serde::Serialize)]
struct BackendTruthPayload {
    tools: Vec<Value>,
    retrieval: BackendRetrievalTruth,
    cache: CacheTruth,
    config: ConfigTruth,
    cli: CliTruth,
    proofs: Vec<BackendProof>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct BackendRetrievalTruth {
    #[serde(flatten)]
    base: RetrievalTruth,
    cache: CacheTruth,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CacheTruth {
    exact_search_enabled: bool,
    overview_enabled: bool,
    task_capsule_enabled: bool,
    sync_invalidates_caches: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ConfigTruth {
    workspace_profiles_enabled: bool,
    workspace_profiles_apply_to_retrieval: bool,
    max_workspace_profiles: usize,
    max_profile_repo_roots: usize,
    default_mode: &'static str,
    default_limit: usize,
    per_repo_limit: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CliTruth {
    local_entrypoint_enabled: bool,
    commands: Vec<CliCommandTruth>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CliCommandTruth {
    label: &'static str,
    command: &'static str,
}

#[derive(Debug, Clone, serde::Serialize)]
struct BackendProof {
    id: &'static str,
    command: &'static str,
}

#[derive(Debug, Deserialize)]
struct RepositoryStateQuery {
    root: String,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct MemorySearchQuery {
    root: String,
    query: String,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ContextRunsQuery {
    root: String,
    limit: Option<usize>,
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
    Router::new()
        .route("/truth", get(truth_handler))
        .route("/repositories", get(repository_state_handler))
        .route("/memory", get(memory_search_handler))
        .route("/context-runs", get(context_runs_handler))
}

async fn truth_handler() -> Json<BackendTruthPayload> {
    Json(backend_truth_payload())
}

async fn repository_state_handler(
    Query(query): Query<RepositoryStateQuery>,
) -> Result<Json<Vec<RegisteredRepositoryState>>, (StatusCode, String)> {
    if query.root.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "root query parameter is required".to_string(),
        ));
    }

    registered_repository_state(&PathBuf::from(query.root), query.limit.unwrap_or(20))
        .map(Json)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

async fn memory_search_handler(
    Query(query): Query<MemorySearchQuery>,
) -> Result<Json<MemorySearchResult>, (StatusCode, String)> {
    if query.root.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "root query parameter is required".to_string(),
        ));
    }
    if query.query.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "query parameter is required".to_string(),
        ));
    }

    memory_search(
        &PathBuf::from(query.root),
        &query.query,
        query.limit.unwrap_or(3),
    )
    .map(Json)
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

async fn context_runs_handler(
    Query(query): Query<ContextRunsQuery>,
) -> Result<Json<Vec<ContextAssembly>>, (StatusCode, String)> {
    if query.root.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "root query parameter is required".to_string(),
        ));
    }

    context_run_history(&PathBuf::from(query.root), query.limit.unwrap_or(5))
        .map(Json)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

fn backend_truth_payload() -> BackendTruthPayload {
    let cache = cache_truth();

    BackendTruthPayload {
        tools: current_tool_registry(),
        retrieval: BackendRetrievalTruth {
            base: retrieval_truth(),
            cache: cache.clone(),
        },
        cache,
        config: config_truth(),
        cli: cli_truth(),
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
            BackendProof {
                id: "local_operator_workflow",
                command: "cargo test -p mcp-server local_operator_workflow_is_visible_through_truth_and_stdio",
            },
        ],
    }
}

fn cache_truth() -> CacheTruth {
    CacheTruth {
        exact_search_enabled: true,
        overview_enabled: true,
        task_capsule_enabled: true,
        sync_invalidates_caches: true,
    }
}

fn config_truth() -> ConfigTruth {
    ConfigTruth {
        workspace_profiles_enabled: true,
        workspace_profiles_apply_to_retrieval: false,
        max_workspace_profiles: 20,
        max_profile_repo_roots: 5,
        default_mode: "exact_search",
        default_limit: 3,
        per_repo_limit: 3,
    }
}

fn cli_truth() -> CliTruth {
    CliTruth {
        local_entrypoint_enabled: true,
        commands: vec![
            CliCommandTruth {
                label: "Truth",
                command: "cargo run -p mcp-server -- --cli truth",
            },
            CliCommandTruth {
                label: "Register repository",
                command: "cargo run -p mcp-server -- --cli register <state-root> <repo-root>",
            },
            CliCommandTruth {
                label: "Sync repository",
                command: "cargo run -p mcp-server -- --cli sync <repo-root>",
            },
            CliCommandTruth {
                label: "Inspect local state",
                command: "cargo run -p mcp-server -- --cli state <root>",
            },
            CliCommandTruth {
                label: "Assemble context",
                command: "cargo run -p mcp-server -- --cli assemble <repo-root> <query>",
            },
        ],
    }
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
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid content length: {error}"),
                )
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

#[cfg(test)]
mod tests;
