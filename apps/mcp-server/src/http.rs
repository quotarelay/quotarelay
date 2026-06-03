use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use context_engine::{
    context_run_history, memory_search, registered_repository_state, ContextAssembly,
    MemorySearchResult, RegisteredRepositoryState,
};
use serde::Deserialize;

use crate::truth::{backend_truth_payload, BackendTruthPayload};

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

pub async fn run_http(addr: SocketAddr) -> io::Result<()> {
    if !addr.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "local HTTP control plane must bind to a loopback address",
        ));
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(io::Error::other)?;
    axum::serve(listener, http_router())
        .await
        .map_err(io::Error::other)
}

pub(crate) fn http_router() -> Router {
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
