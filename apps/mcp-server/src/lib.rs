use std::io::{self, BufRead, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use context_engine::{
    assemble_context_for_registered_repositories,
    clear_retrieval_caches,
    context_run_detail, context_run_history, inspect_local_state, list_registered_repositories, memory_export,
    memory_import, memory_read, memory_search, memory_delete, memory_update, memory_write,
    list_workspace_profiles, save_workspace_profile,
    register_repository, registered_repository_state, remove_registered_repository,
    registered_repository_detail, update_registered_repository_metadata,
    inspect_retrieval_caches, invalidate_exact_match_cache, CacheClearResult, CacheInspection,
    ContextAssembly, EngineInfo, MemoryNote, MemorySearchResult,
    MemoryWriteResult, RegisteredRepository, RegisteredRepositoryState, MemoryDeleteResult,
    MemoryExportPayload, MemoryExportResult, MemoryImportResult, MemoryUpdateResult,
    MultiRepositoryContextAssembly,
    RepositoryMetadataUpdateResult, RepositoryRegistrationResult, RepositoryRemovalResult, RetrievalMode, RetrievedContext,
    RetrievalTruth, WorkspaceProfile, WorkspaceProfileSaveResult, retrieve_context, retrieval_truth,
};
use repo_index::{repo_inventory, search_code, sync_repo};
use serde::Deserialize;
use serde_json::{json, Value};

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
        return Err((StatusCode::BAD_REQUEST, "root query parameter is required".to_string()));
    }

    registered_repository_state(&PathBuf::from(query.root), query.limit.unwrap_or(20))
        .map(Json)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

async fn memory_search_handler(
    Query(query): Query<MemorySearchQuery>,
) -> Result<Json<MemorySearchResult>, (StatusCode, String)> {
    if query.root.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "root query parameter is required".to_string()));
    }
    if query.query.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "query parameter is required".to_string()));
    }

    memory_search(&PathBuf::from(query.root), &query.query, query.limit.unwrap_or(3))
        .map(Json)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

async fn context_runs_handler(
    Query(query): Query<ContextRunsQuery>,
) -> Result<Json<Vec<ContextAssembly>>, (StatusCode, String)> {
    if query.root.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "root query parameter is required".to_string()));
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

fn current_tool_registry() -> Vec<Value> {
    vec![
        bootstrap_tool(),
        sync_repo_tool(),
        repo_inventory_tool(),
        inspect_local_state_tool(),
        cache_inspect_tool(),
        cache_clear_tool(),
        register_repository_tool(),
        list_repositories_tool(),
        repository_state_tool(),
        repository_detail_tool(),
        repository_update_metadata_tool(),
        remove_repository_tool(),
        workspace_profile_save_tool(),
        workspace_profile_list_tool(),
        search_code_tool(),
        memory_write_tool(),
        memory_read_tool(),
        memory_update_tool(),
        memory_delete_tool(),
        memory_export_tool(),
        memory_import_tool(),
        memory_search_tool(),
        context_run_detail_tool(),
        context_run_history_tool(),
        multi_repo_assemble_context_tool(),
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

fn inspect_local_state_tool() -> Value {
    json!({
        "name": "inspect_local_state",
        "description": "Returns bounded local Quotarelay state presence and counts without dumping persisted contents.",
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

fn cache_inspect_tool() -> Value {
    json!({
        "name": "cache_inspect",
        "description": "Returns retrieval cache file presence and counts without reporting hit rates or optimization metrics.",
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

fn cache_clear_tool() -> Value {
    json!({
        "name": "cache_clear",
        "description": "Explicitly clears local retrieval cache files owned by the context engine.",
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

fn context_run_detail_tool() -> Value {
    json!({
        "name": "context_run_detail",
        "description": "Reads one recent context run by generated_at_epoch_ms with inclusion and omission reasons.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "generated_at_epoch_ms": { "type": "integer", "minimum": 0 }
            },
            "required": ["root", "generated_at_epoch_ms"],
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

fn repository_detail_tool() -> Value {
    json!({
        "name": "repository_detail",
        "description": "Returns one registered repository's sync state and recent run truth.",
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

fn repository_update_metadata_tool() -> Value {
    json!({
        "name": "repository_update_metadata",
        "description": "Updates display metadata for one registered repository without changing its root identity.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "repo_root": { "type": "string" },
                "name": { "type": "string" }
            },
            "required": ["root", "repo_root", "name"],
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

fn workspace_profile_save_tool() -> Value {
    json!({
        "name": "workspace_profile_save",
        "description": "Persists a local named workspace profile with repo roots and default retrieval settings.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "name": { "type": "string" },
                "repo_roots": { "type": "array", "items": { "type": "string" }, "minItems": 1, "maxItems": 5 },
                "default_mode": { "type": "string", "enum": ["exact_search", "overview", "task_capsule"] },
                "default_limit": { "type": "integer", "minimum": 1, "maximum": 5 },
                "per_repo_limit": { "type": "integer", "minimum": 1, "maximum": 5 }
            },
            "required": ["root", "name", "repo_roots"],
            "additionalProperties": false
        }
    })
}

fn workspace_profile_list_tool() -> Value {
    json!({
        "name": "workspace_profile_list",
        "description": "Lists bounded local workspace profiles without applying them to retrieval.",
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

fn memory_update_tool() -> Value {
    json!({
        "name": "memory_update",
        "description": "Updates a durable memory note by id in the local Quotarelay state directory.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "id": { "type": "string" },
                "title": { "type": "string" },
                "content": { "type": "string" },
                "tags": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["root", "id"],
            "additionalProperties": false
        }
    })
}

fn memory_delete_tool() -> Value {
    json!({
        "name": "memory_delete",
        "description": "Deletes one durable memory note by id from the local Quotarelay state directory.",
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

fn memory_export_tool() -> Value {
    json!({
        "name": "memory_export",
        "description": "Exports bounded durable memory notes as a local JSON payload for backup or migration.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 50 }
            },
            "required": ["root"],
            "additionalProperties": false
        }
    })
}

fn memory_import_tool() -> Value {
    json!({
        "name": "memory_import",
        "description": "Imports a bounded durable memory JSON payload into the local Quotarelay state directory.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "payload": {
                    "type": "object",
                    "properties": {
                        "notes": { "type": "array" }
                    },
                    "required": ["notes"],
                    "additionalProperties": false
                }
            },
            "required": ["root", "payload"],
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

fn multi_repo_assemble_context_tool() -> Value {
    json!({
        "name": "multi_repo_assemble_context",
        "description": "Builds bounded context packs from explicitly selected registered repositories with per-repo limits.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "repo_roots": { "type": "array", "items": { "type": "string" }, "minItems": 1, "maxItems": 5 },
                "query": { "type": "string" },
                "per_repo_limit": { "type": "integer", "minimum": 1, "maximum": 5 }
            },
            "required": ["root", "repo_roots", "query"],
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
        Some("memory_update") => match memory_update_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("memory_delete") => match memory_delete_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("memory_export") => match memory_export_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("memory_import") => match memory_import_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
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
        Some("inspect_local_state") => match inspect_local_state_from_args(&arguments) {
            Ok(state) => json!({
                "type": "text",
                "text": serde_json::to_string(&state).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("cache_inspect") => match cache_inspect_from_args(&arguments) {
            Ok(state) => json!({
                "type": "text",
                "text": serde_json::to_string(&state).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("cache_clear") => match cache_clear_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
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
        Some("repository_detail") => match repository_detail_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("repository_update_metadata") => match repository_update_metadata_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
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
        Some("workspace_profile_save") => match workspace_profile_save_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("workspace_profile_list") => match workspace_profile_list_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("context_run_detail") => match context_run_detail_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string())
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
        Some("multi_repo_assemble_context") => match multi_repo_assemble_context_from_args(&arguments) {
            Ok(results) => json!({
                "type": "text",
                "text": serde_json::to_string(&results).unwrap_or_else(|_| "{}".to_string())
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
    let result = sync_repo(&root).map_err(|error| format!("sync_repo failed: {error}"))?;
    invalidate_exact_match_cache(&root)
        .map_err(|error| format!("sync_repo cache invalidation failed: {error}"))?;
    Ok(result)
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

fn inspect_local_state_from_args(
    arguments: &Value,
) -> Result<context_engine::LocalStateInspection, String> {
    let root = parse_root(arguments)?;
    inspect_local_state(&root).map_err(|error| format!("inspect_local_state failed: {error}"))
}

fn cache_inspect_from_args(arguments: &Value) -> Result<CacheInspection, String> {
    let root = parse_root(arguments)?;
    inspect_retrieval_caches(&root).map_err(|error| format!("cache_inspect failed: {error}"))
}

fn cache_clear_from_args(arguments: &Value) -> Result<CacheClearResult, String> {
    let root = parse_root(arguments)?;
    clear_retrieval_caches(&root).map_err(|error| format!("cache_clear failed: {error}"))
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

fn repository_detail_from_args(
    arguments: &Value,
) -> Result<Option<RegisteredRepositoryState>, String> {
    let root = parse_root(arguments)?;
    let repo_root = parse_repo_root(arguments)?;

    registered_repository_detail(&root, &repo_root)
        .map_err(|error| format!("repository_detail failed: {error}"))
}

fn repository_update_metadata_from_args(
    arguments: &Value,
) -> Result<RepositoryMetadataUpdateResult, String> {
    let root = parse_root(arguments)?;
    let repo_root = parse_repo_root(arguments)?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "repository_update_metadata requires a string name".to_string())?;

    update_registered_repository_metadata(&root, &repo_root, Some(name))
        .map_err(|error| format!("repository_update_metadata failed: {error}"))
}

fn remove_repository_from_args(arguments: &Value) -> Result<RepositoryRemovalResult, String> {
    let root = parse_root(arguments)?;
    let repo_root = parse_repo_root(arguments)?;
    remove_registered_repository(&root, &repo_root)
        .map_err(|error| format!("remove_repository failed: {error}"))
}

fn workspace_profile_save_from_args(
    arguments: &Value,
) -> Result<WorkspaceProfileSaveResult, String> {
    let root = parse_root(arguments)?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "workspace_profile_save requires a string name".to_string())?;
    let repo_roots = arguments
        .get("repo_roots")
        .and_then(Value::as_array)
        .ok_or_else(|| "workspace_profile_save requires repo_roots array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| "workspace_profile_save repo_roots must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let default_mode = parse_optional_retrieval_mode(arguments, "default_mode")?
        .unwrap_or(RetrievalMode::ExactSearch);
    let default_limit = arguments
        .get("default_limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(3);
    let per_repo_limit = arguments
        .get("per_repo_limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(default_limit);

    save_workspace_profile(&root, name, &repo_roots, default_mode, default_limit, per_repo_limit)
        .map_err(|error| format!("workspace_profile_save failed: {error}"))
}

fn workspace_profile_list_from_args(arguments: &Value) -> Result<Vec<WorkspaceProfile>, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(10);

    list_workspace_profiles(&root, limit)
        .map_err(|error| format!("workspace_profile_list failed: {error}"))
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
        .map(|values| {
            values
                .as_array()
                .ok_or_else(|| "memory_write tags must be an array of strings".to_string())
                .and_then(|items| {
                    items
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .map(ToString::to_string)
                                .ok_or_else(|| "memory_write tags must be an array of strings".to_string())
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
        })
        .transpose()?
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

fn memory_update_from_args(arguments: &Value) -> Result<MemoryUpdateResult, String> {
    let root = parse_root(arguments)?;
    let id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "memory_update requires a string id".to_string())?;
    let title = arguments.get("title").and_then(Value::as_str);
    let content = arguments.get("content").and_then(Value::as_str);
    let tags = arguments
        .get("tags")
        .map(|values| {
            values
                .as_array()
                .ok_or_else(|| "memory_update tags must be an array of strings".to_string())
                .and_then(|items| {
                    items
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .map(ToString::to_string)
                                .ok_or_else(|| "memory_update tags must be an array of strings".to_string())
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
        })
        .transpose()?;

    memory_update(&root, id, title, content, tags.as_deref())
        .map_err(|error| format!("memory_update failed: {error}"))
}

fn memory_delete_from_args(arguments: &Value) -> Result<MemoryDeleteResult, String> {
    let root = parse_root(arguments)?;
    let id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "memory_delete requires a string id".to_string())?;

    memory_delete(&root, id).map_err(|error| format!("memory_delete failed: {error}"))
}

fn memory_export_from_args(arguments: &Value) -> Result<MemoryExportResult, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(50);

    memory_export(&root, limit).map_err(|error| format!("memory_export failed: {error}"))
}

fn memory_import_from_args(arguments: &Value) -> Result<MemoryImportResult, String> {
    let root = parse_root(arguments)?;
    let payload = arguments
        .get("payload")
        .cloned()
        .ok_or_else(|| "memory_import requires a payload object".to_string())
        .and_then(|value| {
            serde_json::from_value::<MemoryExportPayload>(value)
                .map_err(|error| format!("memory_import payload is invalid: {error}"))
        })?;

    memory_import(&root, payload).map_err(|error| format!("memory_import failed: {error}"))
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

fn context_run_detail_from_args(arguments: &Value) -> Result<Option<ContextAssembly>, String> {
    let root = parse_root(arguments)?;
    let generated_at_epoch_ms = arguments
        .get("generated_at_epoch_ms")
        .and_then(Value::as_u64)
        .map(u128::from)
        .ok_or_else(|| "context_run_detail requires a numeric generated_at_epoch_ms".to_string())?;

    context_run_detail(&root, generated_at_epoch_ms)
        .map_err(|error| format!("context_run_detail failed: {error}"))
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

fn multi_repo_assemble_context_from_args(
    arguments: &Value,
) -> Result<MultiRepositoryContextAssembly, String> {
    let root = parse_root(arguments)?;
    let repo_roots = arguments
        .get("repo_roots")
        .and_then(Value::as_array)
        .ok_or_else(|| "multi_repo_assemble_context requires repo_roots array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| "multi_repo_assemble_context repo_roots must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| "multi_repo_assemble_context requires a string query".to_string())?;
    let per_repo_limit = arguments
        .get("per_repo_limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(2);

    assemble_context_for_registered_repositories(&root, &repo_roots, query, per_repo_limit)
        .map_err(|error| format!("multi_repo_assemble_context failed: {error}"))
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

fn parse_optional_retrieval_mode(
    arguments: &Value,
    field: &str,
) -> Result<Option<RetrievalMode>, String> {
    match arguments.get(field).and_then(Value::as_str) {
        None => Ok(None),
        Some("exact_search") => Ok(Some(RetrievalMode::ExactSearch)),
        Some("overview") => Ok(Some(RetrievalMode::Overview)),
        Some("task_capsule") => Ok(Some(RetrievalMode::TaskCapsule)),
        Some(other) => Err(format!(
            "{field} must be one of exact_search, overview, task_capsule; got {other}"
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

pub fn run_cli<I, S, W>(args: I, mut stdout: W) -> io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    W: Write,
{
    let mut args_iter = args.into_iter();
    let command = match args_iter.next() {
        Some(arg) => arg.as_ref().to_string(),
        None => {
            write_cli_error(&mut stdout, None, "missing command")?;
            return Ok(());
        }
    };

    let result = match command.as_str() {
        "register" => run_cli_register(&mut args_iter),
        "sync" => run_cli_sync(&mut args_iter),
        "state" => run_cli_state(&mut args_iter),
        "search" => run_cli_search(&mut args_iter),
        "assemble" => run_cli_assemble(&mut args_iter),
        "truth" => Ok(json!({"truth": backend_truth_payload()})),
        _ => return write_cli_error(&mut stdout, Some(&command), &format!("unknown command: {command}")),
    };

    match result {
        Ok(result) => {
            serde_json::to_writer(&mut stdout, &json!({
                "ok": true,
                "command": command,
                "result": result,
            }))?;
            stdout.write_all(b"\n")?;
        }
        Err(error) => {
            write_cli_error(&mut stdout, Some(&command), &error)?;
        }
    }

    Ok(())
}

fn run_cli_register<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let state_root = parse_cli_arg(args, "state_root")?;
    let repo_root = parse_cli_arg(args, "repo_root")?;
    let registration = register_repository(&PathBuf::from(state_root), &PathBuf::from(repo_root))
        .map_err(|error| format!("register failed: {error}"))?;
    Ok(json!({"repository": registration.repository}))
}

fn run_cli_sync<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let result = sync_repo(&PathBuf::from(&root)).map_err(|error| format!("sync failed: {error}"))?;
    invalidate_exact_match_cache(&PathBuf::from(&root))
        .map_err(|error| format!("sync failed to invalidate cache: {error}"))?;
    Ok(json!({"sync": result}))
}

fn run_cli_state<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let state = inspect_local_state(&PathBuf::from(&root))
        .map_err(|error| format!("state failed: {error}"))?;
    Ok(json!({"state": state}))
}

fn run_cli_search<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let query = parse_cli_arg(args, "query")?;
    let limit = parse_cli_limit(args, 5)?;
    let hits = search_code(&PathBuf::from(&root), &query, limit)
        .map_err(|error| format!("search failed: {error}"))?;
    Ok(json!({"hits": hits}))
}

fn run_cli_assemble<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let mode = parse_cli_arg(args, "mode")?;
    let query = match mode.as_str() {
        "overview" => None,
        "exact_search" | "task_capsule" => Some(parse_cli_arg(args, "query")?),
        other => return Err(format!(
            "assemble mode must be exact_search, overview, or task_capsule; got {other}"
        )),
    };
    let limit = parse_cli_limit(args, 3)?;
    let retrieval = retrieve_context(
        &PathBuf::from(&root),
        parse_retrieval_mode_from_str(&mode)?,
        query.as_deref(),
        limit,
    )
    .map_err(|error| format!("assemble failed: {error}"))?;
    Ok(json!({"context": retrieval}))
}

fn parse_cli_arg<I, S>(args: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    args
        .next()
        .map(|value| value.as_ref().to_string())
        .ok_or_else(|| format!("{name} is required"))
}

fn parse_cli_arg_opt<I, S>(args: &mut I) -> Option<String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    args.next().map(|value| value.as_ref().to_string())
}

fn parse_cli_limit<I, S>(args: &mut I, default: usize) -> Result<usize, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    match parse_cli_arg_opt(args) {
        Some(value) => value
            .parse::<usize>()
            .map_err(|error| format!("limit must be a positive integer: {error}")),
        None => Ok(default),
    }
}

fn parse_retrieval_mode_from_str(mode: &str) -> Result<RetrievalMode, String> {
    match mode {
        "exact_search" => Ok(RetrievalMode::ExactSearch),
        "overview" => Ok(RetrievalMode::Overview),
        "task_capsule" => Ok(RetrievalMode::TaskCapsule),
        other => Err(format!(
            "assemble mode must be exact_search, overview, or task_capsule; got {other}"
        )),
    }
}

fn write_cli_error<W: Write>(stdout: &mut W, command: Option<&str>, message: &str) -> io::Result<()> {
    serde_json::to_writer(
        &mut *stdout,
        &json!({
            "ok": false,
            "command": command,
            "error_category": classify_error(message),
            "error": message,
        }),
    )?;
    stdout.write_all(b"\n")
}

fn classify_error(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("requires")
        || lower.contains("missing")
        || lower.contains("invalid")
        || lower.contains("must be")
        || lower.contains("unknown command")
        || lower.contains("unsupported")
        || lower.contains("got ")
    {
        "invalid_args"
    } else if lower.contains("not found") || lower.contains("not registered") {
        "missing_state"
    } else if lower.contains("corrupt") || lower.contains("expected") || lower.contains("eof") {
        "corrupt_state"
    } else {
        "backend_error"
    }
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

    use context_engine::{assemble_context, memory_write, register_repository};

    use super::{classify_error, http_router, run_stdio};

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
                "multi_repo_assemble_context",
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
    fn durable_memory_update_and_delete_work_over_stdio() {
        let repo_root = temp_repo();

        let write_request = json_rpc_request(
            27,
            "tools/call",
            json!({
                "name": "memory_write",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "title": "Design note",
                    "content": "Original memory content.",
                    "tags": ["memory"]
                }
            }),
        );
        let framed = format!("Content-Length: {}\r\n\r\n{}", write_request.len(), write_request);
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_write should succeed");
        let created_response = decode_response(&output);
        let created: Value = serde_json::from_str(
            created_response["result"]["content"][0]["text"]
                .as_str()
                .expect("memory write text should exist"),
        )
        .expect("memory write payload should be valid json");
        let note_id = created["note"]["id"]
            .as_str()
            .expect("note id should exist")
            .to_string();

        let update_request = json_rpc_request(
            28,
            "tools/call",
            json!({
                "name": "memory_update",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "id": note_id,
                    "title": "Updated note",
                    "content": "Updated memory content.",
                    "tags": ["updated"]
                }
            }),
        );
        let delete_request = json_rpc_request(
            29,
            "tools/call",
            json!({
                "name": "memory_delete",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "id": created["note"]["id"]
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            update_request.len(),
            update_request,
            delete_request.len(),
            delete_request
        );
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("memory update and delete should succeed");
        let responses = decode_responses(&output);
        let updated: Value = serde_json::from_str(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .expect("update text should exist"),
        )
        .expect("update payload should be valid json");
        let deleted: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("delete text should exist"),
        )
        .expect("delete payload should be valid json");

        assert_eq!(updated["note"]["title"], "Updated note");
        assert_eq!(updated["note"]["content"], "Updated memory content.");
        assert_eq!(updated["note"]["tags"], json!(["updated"]));
        assert_eq!(deleted["note"]["id"], updated["note"]["id"]);

        let read_request = json_rpc_request(
            30,
            "tools/call",
            json!({
                "name": "memory_read",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "id": updated["note"]["id"]
                }
            }),
        );
        let framed = format!("Content-Length: {}\r\n\r\n{}", read_request.len(), read_request);
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory read should succeed");
        let read_response = decode_response(&output);
        assert_eq!(read_response["result"]["content"][0]["text"], "null");
    }

    #[test]
    fn durable_memory_export_and_import_work_over_stdio() {
        let source_root = temp_repo();
        let target_root = temp_repo();

        let write_request = json_rpc_request(
            31,
            "tools/call",
            json!({
                "name": "memory_write",
                "arguments": {
                    "root": source_root.to_string_lossy(),
                    "title": "Design note",
                    "content": "Memory content for migration.",
                    "tags": ["memory"]
                }
            }),
        );
        let framed = format!("Content-Length: {}\r\n\r\n{}", write_request.len(), write_request);
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_write should succeed");

        let export_request = json_rpc_request(
            32,
            "tools/call",
            json!({
                "name": "memory_export",
                "arguments": {
                    "root": source_root.to_string_lossy(),
                    "limit": 10
                }
            }),
        );
        let framed = format!("Content-Length: {}\r\n\r\n{}", export_request.len(), export_request);
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_export should succeed");
        let export_response = decode_response(&output);
        let exported: Value = serde_json::from_str(
            export_response["result"]["content"][0]["text"]
                .as_str()
                .expect("export text should exist"),
        )
        .expect("export payload should be valid json");

        assert_eq!(exported["payload"]["notes"].as_array().map(Vec::len), Some(1));
        assert_eq!(exported["omitted_count"], 0);

        let import_request = json_rpc_request(
            33,
            "tools/call",
            json!({
                "name": "memory_import",
                "arguments": {
                    "root": target_root.to_string_lossy(),
                    "payload": exported["payload"]
                }
            }),
        );
        let framed = format!("Content-Length: {}\r\n\r\n{}", import_request.len(), import_request);
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("memory_import should succeed");
        let import_response = decode_response(&output);
        let imported: Value = serde_json::from_str(
            import_response["result"]["content"][0]["text"]
                .as_str()
                .expect("import text should exist"),
        )
        .expect("import payload should be valid json");

        assert_eq!(imported["imported_count"], 1);
        assert_eq!(imported["replaced_count"], 0);
        assert_eq!(imported["omitted_count"], 0);

        let invalid_import_request = json_rpc_request(
            34,
            "tools/call",
            json!({
                "name": "memory_import",
                "arguments": {
                    "root": target_root.to_string_lossy(),
                    "payload": { "notes": [{ "id": 7 }] }
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}",
            invalid_import_request.len(),
            invalid_import_request
        );
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("invalid memory_import request should still produce a response");
        let invalid_response = decode_response(&output);
        assert!(
            invalid_response["result"]["content"][0]["text"]
                .as_str()
                .expect("invalid import text should exist")
                .contains("memory_import payload is invalid")
        );
    }

    #[test]
    fn memory_write_rejects_non_string_tags_over_stdio() {
        let repo_root = temp_repo();

        let write_request = json_rpc_request(
            46,
            "tools/call",
            json!({
                "name": "memory_write",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "title": "Design note",
                    "content": "Memory content.",
                    "tags": ["memory", 7]
                }
            }),
        );
        let framed = format!("Content-Length: {}\r\n\r\n{}", write_request.len(), write_request);
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("memory_write should produce an error response");
        let response = decode_response(&output);

        assert_eq!(
            response["result"]["content"][0]["text"],
            "memory_write tags must be an array of strings"
        );
    }

    #[test]
    fn corrupt_memory_state_reports_recoverable_error_over_stdio() {
        let repo_root = temp_repo();
        let state_dir = repo_root.join(".quotarelay");
        fs::create_dir_all(&state_dir).expect("state dir should create");
        fs::write(state_dir.join("memory_notes.json"), "{ not json").expect("corrupt memory should write");

        let search_request = json_rpc_request(
            47,
            "tools/call",
            json!({
                "name": "memory_search",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "query": "needle",
                    "limit": 3
                }
            }),
        );
        let framed = format!("Content-Length: {}\r\n\r\n{}", search_request.len(), search_request);
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("corrupt memory search should produce an error response");
        let response = decode_response(&output);
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("error text should exist");

        assert!(text.contains("memory_search failed: corrupt local state file"));
        assert!(text.contains("repair or remove the file to recover"));
    }

    #[test]
    fn cache_inspect_and_clear_work_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

        let sync_request = json_rpc_request(
            48,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let assemble_request = json_rpc_request(
            49,
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
        let inspect_request = json_rpc_request(
            50,
            "tools/call",
            json!({
                "name": "cache_inspect",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let clear_request = json_rpc_request(
            51,
            "tools/call",
            json!({
                "name": "cache_clear",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let inspect_after_request = json_rpc_request(
            52,
            "tools/call",
            json!({
                "name": "cache_inspect",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            sync_request.len(),
            sync_request,
            assemble_request.len(),
            assemble_request,
            inspect_request.len(),
            inspect_request,
            clear_request.len(),
            clear_request,
            inspect_after_request.len(),
            inspect_after_request
        );
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("cache inspect and clear should succeed");
        let responses = decode_responses(&output);
        let before: Value = serde_json::from_str(
            responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("cache inspect text should exist"),
        )
        .expect("cache inspect payload should be valid json");
        let cleared: Value = serde_json::from_str(
            responses[3]["result"]["content"][0]["text"]
                .as_str()
                .expect("cache clear text should exist"),
        )
        .expect("cache clear payload should be valid json");
        let after: Value = serde_json::from_str(
            responses[4]["result"]["content"][0]["text"]
                .as_str()
                .expect("cache inspect after text should exist"),
        )
        .expect("cache inspect after payload should be valid json");

        assert_eq!(before["exact_search_cache"]["present"], true);
        assert_eq!(before["exact_search_cache"]["item_count"], 1);
        assert_eq!(cleared["exact_search_cache_cleared"], true);
        assert_eq!(after["exact_search_cache"]["present"], false);
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
    fn local_cli_register_sync_assemble_and_truth_workflow() {
        let state_root = temp_repo();
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

        let mut output = Vec::new();
        super::run_cli(
            [
                "register".to_string(),
                state_root.to_string_lossy().to_string(),
                repo_root.to_string_lossy().to_string(),
            ],
            &mut output,
        )
        .expect("cli register should succeed");

        let registered: Value = serde_json::from_slice(&output).expect("cli register payload should parse");
        assert!(registered["ok"].as_bool().unwrap_or(false));
        assert_eq!(registered["command"], "register");

        output.clear();
        super::run_cli([
            "sync".to_string(),
            repo_root.to_string_lossy().to_string(),
        ], &mut output)
        .expect("cli sync should succeed");

        let synced: Value = serde_json::from_slice(&output).expect("cli sync payload should parse");
        assert!(synced["ok"].as_bool().unwrap_or(false));
        assert_eq!(synced["command"], "sync");
        assert_eq!(synced["result"]["sync"]["indexed_files"], 1);

        output.clear();
        super::run_cli([
            "state".to_string(),
            repo_root.to_string_lossy().to_string(),
        ], &mut output)
        .expect("cli state should succeed");

        let state: Value = serde_json::from_slice(&output).expect("cli state payload should parse");
        assert!(state["ok"].as_bool().unwrap_or(false));
        assert_eq!(state["command"], "state");
        assert_eq!(state["result"]["state"]["index"]["present"], true);
        assert_eq!(state["result"]["state"]["index"]["item_count"], 1);

        output.clear();
        super::run_cli(
            [
                "assemble".to_string(),
                repo_root.to_string_lossy().to_string(),
                "exact_search".to_string(),
                "needle".to_string(),
                "2".to_string(),
            ],
            &mut output,
        )
        .expect("cli assemble should succeed");

        let assembly: Value = serde_json::from_slice(&output).expect("cli assemble payload should parse");
        assert!(assembly["ok"].as_bool().unwrap_or(false));
        assert_eq!(assembly["command"], "assemble");
        assert_eq!(assembly["result"]["context"]["mode"], "exact_search");
        assert_eq!(assembly["result"]["context"]["snippets"].as_array().map(|items| items.len()), Some(1));

        output.clear();
        super::run_cli(["truth".to_string()], &mut output)
            .expect("cli truth should succeed");

        let truth: Value = serde_json::from_slice(&output).expect("cli truth payload should parse");
        assert!(truth["ok"].as_bool().unwrap_or(false));
        assert_eq!(truth["command"], "truth");
        assert_eq!(truth["result"]["truth"]["cache"]["exact_search_enabled"], true);
        assert_eq!(truth["result"]["truth"]["cache"]["sync_invalidates_caches"], true);
    }

    #[test]
    fn local_cli_uses_stable_json_success_and_error_contract() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");
        repo_index::sync_repo(&repo_root).expect("sync should succeed");

        let mut output = Vec::new();
        super::run_cli(["truth".to_string()], &mut output)
            .expect("cli truth should succeed");
        let truth: Value = serde_json::from_slice(&output).expect("truth payload should parse");
        assert_eq!(truth["ok"], true);
        assert_eq!(truth["command"], "truth");
        assert!(truth["result"]["truth"]["tools"].is_array());

        output.clear();
        super::run_cli(
            [
                "search".to_string(),
                repo_root.to_string_lossy().to_string(),
                "needle".to_string(),
                "1".to_string(),
            ],
            &mut output,
        )
        .expect("cli search should succeed");
        let search: Value = serde_json::from_slice(&output).expect("search payload should parse");
        assert_eq!(search["ok"], true);
        assert_eq!(search["command"], "search");
        assert_eq!(search["result"]["hits"].as_array().map(|items| items.len()), Some(1));

        output.clear();
        super::run_cli(
            [
                "assemble".to_string(),
                repo_root.to_string_lossy().to_string(),
                "overview".to_string(),
                "1".to_string(),
            ],
            &mut output,
        )
        .expect("cli overview should succeed");
        let overview: Value = serde_json::from_slice(&output).expect("overview payload should parse");
        assert_eq!(overview["ok"], true);
        assert_eq!(overview["command"], "assemble");
        assert_eq!(overview["result"]["context"]["mode"], "overview");

        output.clear();
        super::run_cli(Vec::<String>::new(), &mut output)
            .expect("missing command should produce json error");
        let missing: Value = serde_json::from_slice(&output).expect("missing command payload should parse");
        assert_eq!(missing["ok"], false);
        assert!(missing["command"].is_null());
        assert_eq!(missing["error_category"], "invalid_args");
        assert_eq!(missing["error"], "missing command");

        output.clear();
        super::run_cli(["unknown".to_string()], &mut output)
            .expect("unknown command should produce json error");
        let unknown: Value = serde_json::from_slice(&output).expect("unknown command payload should parse");
        assert_eq!(unknown["ok"], false);
        assert_eq!(unknown["command"], "unknown");
        assert_eq!(unknown["error_category"], "invalid_args");
        assert_eq!(unknown["error"], "unknown command: unknown");

        output.clear();
        super::run_cli(
            [
                "search".to_string(),
                repo_root.to_string_lossy().to_string(),
                "needle".to_string(),
                "not-a-number".to_string(),
            ],
            &mut output,
        )
        .expect("invalid limit should produce json error");
        let invalid_limit: Value = serde_json::from_slice(&output).expect("invalid limit payload should parse");
        assert_eq!(invalid_limit["ok"], false);
        assert_eq!(invalid_limit["command"], "search");
        assert_eq!(invalid_limit["error_category"], "invalid_args");
        assert!(invalid_limit["error"]
            .as_str()
            .expect("error should be a string")
            .starts_with("limit must be a positive integer"));
    }

    #[test]
    fn error_classifier_uses_stable_categories() {
        assert_eq!(classify_error("missing command"), "invalid_args");
        assert_eq!(classify_error("repository C:\\repo is not registered"), "missing_state");
        assert_eq!(classify_error("corrupt memory_notes.json"), "corrupt_state");
        assert_eq!(classify_error("sync failed: disk unavailable"), "backend_error");
    }

    #[test]
    fn inspect_local_state_works_over_stdio() {
        let state_root = temp_repo();
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

        let sync_request = json_rpc_request(
            61,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let register_request = json_rpc_request(
            62,
            "tools/call",
            json!({
                "name": "register_repository",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "repo_root": state_root.to_string_lossy()
                }
            }),
        );
        let memory_request = json_rpc_request(
            63,
            "tools/call",
            json!({
                "name": "memory_write",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "title": "Needle note",
                    "content": "needle memory"
                }
            }),
        );
        let assemble_request = json_rpc_request(
            64,
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
        let inspect_request = json_rpc_request(
            65,
            "tools/call",
            json!({
                "name": "inspect_local_state",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            sync_request.len(),
            sync_request,
            register_request.len(),
            register_request,
            memory_request.len(),
            memory_request,
            assemble_request.len(),
            assemble_request,
            inspect_request.len(),
            inspect_request
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("local state inspection workflow should succeed");

        let responses = decode_responses(&output);
        let state: Value = serde_json::from_str(
            responses[4]["result"]["content"][0]["text"]
                .as_str()
                .expect("state text should exist"),
        )
        .expect("state payload should be valid json");

        assert_eq!(state["index"]["present"], true);
        assert_eq!(state["index"]["item_count"], 1);
        assert_eq!(state["memory_notes"]["present"], true);
        assert_eq!(state["memory_notes"]["item_count"], 1);
        assert_eq!(state["context_run_history"]["present"], true);
        assert_eq!(state["context_run_history"]["item_count"], 1);
        assert_eq!(state["registered_repositories"]["present"], true);
        assert_eq!(state["registered_repositories"]["item_count"], 1);
        assert_eq!(state["exact_search_cache"]["present"], true);
        assert_eq!(state["exact_search_cache"]["item_count"], 1);
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
    fn repository_detail_tool_reports_one_repo_truth_over_stdio() {
        let state_root = temp_repo();
        let repo_root = temp_repo();
        let missing_root = temp_repo();
        fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

        let register_request = json_rpc_request(
            31,
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
            32,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let assemble_request = json_rpc_request(
            33,
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
        let detail_request = json_rpc_request(
            34,
            "tools/call",
            json!({
                "name": "repository_detail",
                "arguments": {
                    "root": state_root.to_string_lossy(),
                    "repo_root": repo_root.to_string_lossy()
                }
            }),
        );
        let missing_request = json_rpc_request(
            35,
            "tools/call",
            json!({
                "name": "repository_detail",
                "arguments": {
                    "root": state_root.to_string_lossy(),
                    "repo_root": missing_root.to_string_lossy()
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            register_request.len(),
            register_request,
            sync_request.len(),
            sync_request,
            assemble_request.len(),
            assemble_request,
            detail_request.len(),
            detail_request,
            missing_request.len(),
            missing_request
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("repository_detail should succeed");

        let responses = decode_responses(&output);
        let detail: Value = serde_json::from_str(
            responses[3]["result"]["content"][0]["text"]
                .as_str()
                .expect("repository_detail text should exist"),
        )
        .expect("repository_detail payload should be valid json");

        assert_eq!(detail["repository"]["id"], detail["repository"]["root"]);
        assert!(
            detail["repository"]["root"]
                .as_str()
                .expect("repository root should exist")
                .contains("quotarelay-repo-")
        );
        assert_eq!(detail["sync"]["status"], "indexed");
        assert_eq!(detail["sync"]["indexed_files"], 1);
        assert_eq!(detail["recent_context_run"]["query"], "needle");
        assert_eq!(responses[4]["result"]["content"][0]["text"], "null");
    }

    #[test]
    fn repository_update_metadata_changes_display_name_over_stdio() {
        let state_root = temp_repo();
        let repo_root = temp_repo();

        let register_request = json_rpc_request(
            36,
            "tools/call",
            json!({
                "name": "register_repository",
                "arguments": {
                    "root": state_root.to_string_lossy(),
                    "repo_root": repo_root.to_string_lossy()
                }
            }),
        );
        let update_request = json_rpc_request(
            37,
            "tools/call",
            json!({
                "name": "repository_update_metadata",
                "arguments": {
                    "root": state_root.to_string_lossy(),
                    "repo_root": repo_root.to_string_lossy(),
                    "name": "Workspace API"
                }
            }),
        );
        let detail_request = json_rpc_request(
            38,
            "tools/call",
            json!({
                "name": "repository_detail",
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
            update_request.len(),
            update_request,
            detail_request.len(),
            detail_request
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("repository metadata update should succeed");

        let responses = decode_responses(&output);
        let registered: Value = serde_json::from_str(
            responses[0]["result"]["content"][0]["text"]
                .as_str()
                .expect("registration text should exist"),
        )
        .expect("registration payload should be valid json");
        let updated: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("metadata update text should exist"),
        )
        .expect("metadata update payload should be valid json");
        let detail: Value = serde_json::from_str(
            responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("repository detail text should exist"),
        )
        .expect("repository detail payload should be valid json");

        assert_eq!(updated["repository"]["name"], "Workspace API");
        assert_eq!(updated["repository"]["id"], registered["repository"]["id"]);
        assert_eq!(updated["repository"]["root"], registered["repository"]["root"]);
        assert_eq!(detail["repository"]["name"], "Workspace API");
        assert_eq!(detail["repository"]["id"], registered["repository"]["id"]);
    }

    #[test]
    fn multi_repo_assemble_context_works_over_stdio() {
        let state_root = temp_repo();
        let repo_one = temp_repo();
        let repo_two = temp_repo();
        fs::write(repo_one.join("alpha.txt"), "needle one\nneedle two\n").expect("repo one file should write");
        fs::write(repo_two.join("beta.txt"), "needle three\nneedle four\n").expect("repo two file should write");

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
        assert_eq!(assembly["repositories"][0]["assembly"]["snippets"].as_array().map(Vec::len), Some(1));
        assert_eq!(assembly["repositories"][1]["assembly"]["snippets"].as_array().map(Vec::len), Some(1));
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

    #[test]
    fn assemble_context_exact_search_cache_honors_sync_invalidation_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

        let sync_request = json_rpc_request(
            31,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let assemble_request = json_rpc_request(
            32,
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
            .expect("initial exact_search should succeed");

        let responses = decode_responses(&output);
        let first: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("first assembly text should exist"),
        )
        .expect("first assembly payload should be valid json");

        fs::write(repo_root.join("src.txt"), "fresh needle after sync\n")
            .expect("repo file should rewrite");
        let resync_request = json_rpc_request(
            33,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );

        let mut second_output = Vec::new();
        let second_request = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            resync_request.len(),
            resync_request,
            assemble_request.len(),
            assemble_request,
            assemble_request.len(),
            assemble_request
        );
        run_stdio(Cursor::new(second_request.into_bytes()), &mut second_output)
            .expect("repeated exact_search should succeed");

        let second_responses = decode_responses(&second_output);
        let cached: Value = serde_json::from_str(
            second_responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("cached assembly text should exist"),
        )
        .expect("cached assembly payload should be valid json");
        let refreshed: Value = serde_json::from_str(
            second_responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("refreshed assembly text should exist"),
        )
        .expect("refreshed assembly payload should be valid json");

        assert_eq!(cached, refreshed);
        assert_eq!(cached["snippets"][0]["line"], "fresh needle after sync");
        assert_ne!(cached, first);
    }

    #[test]
    fn assemble_context_exact_search_cache_normalizes_equivalent_queries_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "needle in repo\n").expect("repo file should write");

        let sync_request = json_rpc_request(
            31,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let first_assemble_request = json_rpc_request(
            32,
            "tools/call",
            json!({
                "name": "assemble_context",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "mode": "exact_search",
                    "query": "Needle",
                    "limit": 2
                }
            }),
        );
        let second_assemble_request = json_rpc_request(
            33,
            "tools/call",
            json!({
                "name": "assemble_context",
                "arguments": {
                    "root": repo_root.to_string_lossy(),
                    "mode": "exact_search",
                    "query": "  needle  ",
                    "limit": 2
                }
            }),
        );
        let framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            sync_request.len(),
            sync_request,
            first_assemble_request.len(),
            first_assemble_request,
            second_assemble_request.len(),
            second_assemble_request,
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("canonicalized exact_search should succeed");

        let responses = decode_responses(&output);
        let first: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("first assembly text should exist"),
        )
        .expect("first assembly payload should be valid json");
        let second: Value = serde_json::from_str(
            responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("second assembly text should exist"),
        )
        .expect("second assembly payload should be valid json");

        assert_eq!(first["query"], second["query"]);
        assert_eq!(first["generated_at_epoch_ms"], second["generated_at_epoch_ms"]);
        assert_eq!(first["snippets"], second["snippets"]);
    }

    #[test]
    fn assemble_context_overview_and_task_capsule_responses_are_repeatable_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
        fs::write(
            repo_root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("rust file should write");

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
        let overview_request = json_rpc_request(
            36,
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
            37,
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
        let repeated_overview_request = json_rpc_request(
            38,
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
        let repeated_task_request = json_rpc_request(
            39,
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
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            sync_request.len(),
            sync_request,
            overview_request.len(),
            overview_request,
            task_request.len(),
            task_request,
            repeated_overview_request.len(),
            repeated_overview_request,
            repeated_task_request.len(),
            repeated_task_request
        );
        let mut output = Vec::new();

        run_stdio(Cursor::new(framed.into_bytes()), &mut output)
            .expect("repeatable overview and task capsule responses should succeed");

        let responses = decode_responses(&output);
        let first_overview: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("overview text should exist"),
        )
        .expect("overview payload should be valid json");
        let first_task: Value = serde_json::from_str(
            responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("task text should exist"),
        )
        .expect("task payload should be valid json");
        let second_overview: Value = serde_json::from_str(
            responses[3]["result"]["content"][0]["text"]
                .as_str()
                .expect("second overview text should exist"),
        )
        .expect("second overview payload should be valid json");
        let second_task: Value = serde_json::from_str(
            responses[4]["result"]["content"][0]["text"]
                .as_str()
                .expect("second task text should exist"),
        )
        .expect("second task payload should be valid json");

        assert_eq!(first_overview, second_overview);
        assert_eq!(first_task, second_task);
    }

    #[test]
    fn assemble_context_capsule_caches_honor_sync_invalidation_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
        fs::write(
            repo_root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("rust file should write");

        let sync_request = json_rpc_request(
            51,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let overview_request = json_rpc_request(
            52,
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
            53,
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
            .expect("initial capsule modes should succeed");

        let responses = decode_responses(&output);
        let first_overview: Value = serde_json::from_str(
            responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("overview text should exist"),
        )
        .expect("overview payload should be valid json");
        let first_task: Value = serde_json::from_str(
            responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("task text should exist"),
        )
        .expect("task payload should be valid json");

        fs::write(repo_root.join("alpha.txt"), "fresh overview after sync\n")
            .expect("alpha file should rewrite");
        fs::write(
            repo_root.join("lib.rs"),
            "struct Widget {\n    id: u32,\n}\n",
        )
        .expect("rust file should rewrite");
        let resync_request = json_rpc_request(
            54,
            "tools/call",
            json!({
                "name": "sync_repo",
                "arguments": {
                    "root": repo_root.to_string_lossy()
                }
            }),
        );
        let second_framed = format!(
            "Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}Content-Length: {}\r\n\r\n{}",
            resync_request.len(),
            resync_request,
            overview_request.len(),
            overview_request,
            overview_request.len(),
            overview_request,
            task_request.len(),
            task_request,
            task_request.len(),
            task_request
        );
        let mut second_output = Vec::new();

        run_stdio(Cursor::new(second_framed.into_bytes()), &mut second_output)
            .expect("refreshed capsule modes should succeed");

        let second_responses = decode_responses(&second_output);
        let refreshed_overview: Value = serde_json::from_str(
            second_responses[1]["result"]["content"][0]["text"]
                .as_str()
                .expect("refreshed overview text should exist"),
        )
        .expect("refreshed overview payload should be valid json");
        let cached_overview: Value = serde_json::from_str(
            second_responses[2]["result"]["content"][0]["text"]
                .as_str()
                .expect("cached overview text should exist"),
        )
        .expect("cached overview payload should be valid json");
        let refreshed_task: Value = serde_json::from_str(
            second_responses[3]["result"]["content"][0]["text"]
                .as_str()
                .expect("refreshed task text should exist"),
        )
        .expect("refreshed task payload should be valid json");
        let cached_task: Value = serde_json::from_str(
            second_responses[4]["result"]["content"][0]["text"]
                .as_str()
                .expect("cached task text should exist"),
        )
        .expect("cached task payload should be valid json");

        assert_eq!(refreshed_overview, cached_overview);
        assert_eq!(refreshed_task, cached_task);
        assert_eq!(refreshed_overview["documents"][0]["contents"], "fresh overview after sync\n");
        assert!(refreshed_task["documents"][0]["contents"]
            .as_str()
            .expect("refreshed task contents should exist")
            .contains("id: u32"));
        assert_ne!(refreshed_overview, first_overview);
        assert_ne!(refreshed_task, first_task);
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

        assert_eq!(payload["tools"].as_array().map(|items| items.len()), Some(26));
        assert_eq!(payload["retrieval"]["modes"], json!(["exact_search", "overview", "task_capsule"]));
        assert_eq!(payload["retrieval"]["limits"]["max_context_items"], 5);
        assert_eq!(payload["retrieval"]["durable_memory_enabled"], true);
        assert_eq!(payload["retrieval"]["cache"]["exact_search_enabled"], true);
        assert_eq!(payload["retrieval"]["cache"]["overview_enabled"], true);
        assert_eq!(payload["retrieval"]["cache"]["task_capsule_enabled"], true);
        assert_eq!(payload["retrieval"]["cache"]["sync_invalidates_caches"], true);
        assert_eq!(payload["cache"]["exact_search_enabled"], true);
        assert_eq!(payload["cache"]["overview_enabled"], true);
        assert_eq!(payload["cache"]["task_capsule_enabled"], true);
        assert_eq!(payload["cache"]["sync_invalidates_caches"], true);
        assert_eq!(payload["config"]["workspace_profiles_enabled"], true);
        assert_eq!(payload["config"]["workspace_profiles_apply_to_retrieval"], false);
        assert_eq!(payload["config"]["max_workspace_profiles"], 20);
        assert_eq!(payload["config"]["max_profile_repo_roots"], 5);
        assert_eq!(payload["config"]["default_mode"], "exact_search");
        assert_eq!(payload["config"]["default_limit"], 3);
        assert_eq!(payload["config"]["per_repo_limit"], 3);
        assert_eq!(payload["cli"]["local_entrypoint_enabled"], true);
        assert!(payload["cli"]["commands"].as_array().map(|items| items.len()).unwrap_or_default() >= 5);
        assert_eq!(payload["cli"]["commands"][0]["command"], "cargo run -p mcp-server -- --cli truth");
        assert!(payload["proofs"].as_array().map(|items| items.len()).unwrap_or_default() >= 5);
    }

    #[tokio::test]
    async fn repository_state_endpoint_returns_registered_repo_truth() {
        let state_root = temp_repo();
        let repo_root = temp_repo();
        fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
        register_repository(&state_root, &repo_root).expect("registration should succeed");
        repo_index::sync_repo(&repo_root).expect("sync should succeed");
        assemble_context(&repo_root, "needle", 2).expect("assembly should succeed");

        let uri = format!("/repositories?root={}&limit=5", state_root.to_string_lossy());
        let response = http_router()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).expect("request should build"))
            .await
            .expect("repository state endpoint should respond");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        let payload: Value = serde_json::from_slice(&body).expect("repository state payload should be valid json");

        assert_eq!(payload.as_array().map(Vec::len), Some(1));
        assert_eq!(payload[0]["sync"]["status"], "indexed");
        assert_eq!(payload[0]["sync"]["indexed_files"], 1);
        assert_eq!(payload[0]["recent_context_run"]["query"], "needle");
    }

    #[tokio::test]
    async fn memory_search_endpoint_returns_bounded_memory_truth() {
        let root = temp_repo();
        memory_write(&root, "Needle note", "needle memory", &["memory".to_string()])
            .expect("memory write should succeed");

        let uri = format!("/memory?root={}&query=needle&limit=3", root.to_string_lossy());
        let response = http_router()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).expect("request should build"))
            .await
            .expect("memory endpoint should respond");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        let payload: Value = serde_json::from_slice(&body).expect("memory payload should be valid json");

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
            .oneshot(Request::builder().uri(uri).body(Body::empty()).expect("request should build"))
            .await
            .expect("context runs endpoint should respond");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        let payload: Value = serde_json::from_slice(&body).expect("context run payload should be valid json");

        assert_eq!(payload.as_array().map(Vec::len), Some(1));
        assert_eq!(payload[0]["query"], "needle");
        assert_eq!(payload[0]["snippets"][0]["reason"]["kind"], "query_line_match");
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

        assert_eq!(registered["repository"]["name"], repo_root.file_name().and_then(|value| value.to_str()).unwrap_or_default());
        assert_eq!(assembly["query"], "needle");
        assert_eq!(assembly["snippets"].as_array().map(|items| items.len()), Some(1));

        let response = http_router()
            .oneshot(Request::builder().uri("/truth").body(Body::empty()).expect("request should build"))
            .await
            .expect("truth endpoint should respond");
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should read");
        let payload: Value = serde_json::from_slice(&body).expect("truth payload should be valid json");

        assert!(payload["proofs"]
            .as_array()
            .expect("proofs should be an array")
            .iter()
            .any(|proof| proof["id"] == "local_operator_workflow"));
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

    #[test]
    fn context_run_detail_works_over_stdio() {
        let repo_root = temp_repo();
        fs::write(repo_root.join("src.txt"), "alpha needle\nbeta needle\ngamma needle\n").expect("repo file should write");

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
        let framed = format!("Content-Length: {}\r\n\r\n{}", detail_request.len(), detail_request);
        let mut output = Vec::new();
        run_stdio(Cursor::new(framed.into_bytes()), &mut output).expect("detail call should succeed");

        let detail_response = decode_response(&output);
        let detail: Value = serde_json::from_str(
            detail_response["result"]["content"][0]["text"]
                .as_str()
                .expect("detail text should exist"),
        )
        .expect("detail payload should be valid json");
        assert_eq!(detail["generated_at_epoch_ms"], assembly["generated_at_epoch_ms"]);
        assert_eq!(detail["snippets"][0]["reason"]["kind"], "query_line_match");
        assert_eq!(detail["omissions"][0]["kind"], "item_limit_reached");
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
