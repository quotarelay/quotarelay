use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use context_engine::{
    context_run_history, memory_search, registered_repository_state, retrieval_truth,
    ContextAssembly, MemorySearchResult, RegisteredRepositoryState, RetrievalTruth,
};
use serde::Deserialize;
use serde_json::Value;

mod args;
mod cli;
mod mcp_transport;
mod tool_calls;
mod tools;

pub use cli::run_cli;
pub(crate) use mcp_transport::run_stdio;
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

#[cfg(test)]
mod tests;
