use std::path::PathBuf;

use context_engine::{
    assemble_context_for_registered_repositories, assemble_handoff_packet, clear_retrieval_caches,
    context_run_detail, context_run_history, inspect_local_state, inspect_retrieval_caches,
    invalidate_exact_match_cache, list_registered_repositories, list_workspace_profiles,
    memory_delete, memory_export, memory_import, memory_read, memory_search,
    memory_update_with_profile, memory_write_with_profile, register_repository,
    registered_repository_detail, registered_repository_state, remove_registered_repository,
    retrieve_context, save_workspace_profile, update_registered_repository_metadata,
    CacheClearResult, CacheInspection, ContextAssembly, HandoffPacket, MemoryDeleteResult,
    MemoryExportPayload, MemoryExportResult, MemoryImportResult, MemoryNote, MemoryProfile,
    MemorySearchResult, MemoryUpdateResult, MemoryWriteResult, MultiRepositoryContextAssembly,
    RegisteredRepository, RegisteredRepositoryState, RepositoryMetadataUpdateResult,
    RepositoryRegistrationResult, RepositoryRemovalResult, RetrievalMode, RetrievedContext,
    WorkspaceProfile, WorkspaceProfileSaveResult,
};
use repo_index::{repo_inventory, search_code, sync_repo};
use serde_json::Value;

pub(crate) fn sync_repo_from_args(arguments: &Value) -> Result<repo_index::SyncResult, String> {
    let root = parse_root(arguments)?;
    let result = sync_repo(&root).map_err(|error| format!("sync_repo failed: {error}"))?;
    invalidate_exact_match_cache(&root)
        .map_err(|error| format!("sync_repo cache invalidation failed: {error}"))?;
    Ok(result)
}

pub(crate) fn search_code_from_args(
    arguments: &Value,
) -> Result<Vec<repo_index::SearchHit>, String> {
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

pub(crate) fn repo_inventory_from_args(
    arguments: &Value,
) -> Result<repo_index::RepoInventory, String> {
    let root = parse_root(arguments)?;
    repo_inventory(&root).map_err(|error| format!("repo_inventory failed: {error}"))
}

pub(crate) fn inspect_local_state_from_args(
    arguments: &Value,
) -> Result<context_engine::LocalStateInspection, String> {
    let root = parse_root(arguments)?;
    inspect_local_state(&root).map_err(|error| format!("inspect_local_state failed: {error}"))
}

pub(crate) fn cache_inspect_from_args(arguments: &Value) -> Result<CacheInspection, String> {
    let root = parse_root(arguments)?;
    inspect_retrieval_caches(&root).map_err(|error| format!("cache_inspect failed: {error}"))
}

pub(crate) fn cache_clear_from_args(arguments: &Value) -> Result<CacheClearResult, String> {
    let root = parse_root(arguments)?;
    clear_retrieval_caches(&root).map_err(|error| format!("cache_clear failed: {error}"))
}

pub(crate) fn register_repository_from_args(
    arguments: &Value,
) -> Result<RepositoryRegistrationResult, String> {
    let root = parse_root(arguments)?;
    let repo_root = parse_repo_root(arguments)?;
    register_repository(&root, &repo_root)
        .map_err(|error| format!("register_repository failed: {error}"))
}

pub(crate) fn list_repositories_from_args(
    arguments: &Value,
) -> Result<Vec<RegisteredRepository>, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(10);

    list_registered_repositories(&root, limit)
        .map_err(|error| format!("list_repositories failed: {error}"))
}

pub(crate) fn repository_state_from_args(
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

pub(crate) fn repository_detail_from_args(
    arguments: &Value,
) -> Result<Option<RegisteredRepositoryState>, String> {
    let root = parse_root(arguments)?;
    let repo_root = parse_repo_root(arguments)?;

    registered_repository_detail(&root, &repo_root)
        .map_err(|error| format!("repository_detail failed: {error}"))
}

pub(crate) fn repository_update_metadata_from_args(
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

pub(crate) fn remove_repository_from_args(
    arguments: &Value,
) -> Result<RepositoryRemovalResult, String> {
    let root = parse_root(arguments)?;
    let repo_root = parse_repo_root(arguments)?;
    remove_registered_repository(&root, &repo_root)
        .map_err(|error| format!("remove_repository failed: {error}"))
}

pub(crate) fn workspace_profile_save_from_args(
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

    save_workspace_profile(
        &root,
        name,
        &repo_roots,
        default_mode,
        default_limit,
        per_repo_limit,
    )
    .map_err(|error| format!("workspace_profile_save failed: {error}"))
}

pub(crate) fn workspace_profile_list_from_args(
    arguments: &Value,
) -> Result<Vec<WorkspaceProfile>, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(10);

    list_workspace_profiles(&root, limit)
        .map_err(|error| format!("workspace_profile_list failed: {error}"))
}

pub(crate) fn memory_write_from_args(arguments: &Value) -> Result<MemoryWriteResult, String> {
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
                            value.as_str().map(ToString::to_string).ok_or_else(|| {
                                "memory_write tags must be an array of strings".to_string()
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
        })
        .transpose()?
        .unwrap_or_default();
    let profile = parse_memory_profile(arguments)?;

    memory_write_with_profile(&root, title, content, &tags, profile)
        .map_err(|error| format!("memory_write failed: {error}"))
}

pub(crate) fn memory_read_from_args(arguments: &Value) -> Result<Option<MemoryNote>, String> {
    let root = parse_root(arguments)?;
    let id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "memory_read requires a string id".to_string())?;

    memory_read(&root, id).map_err(|error| format!("memory_read failed: {error}"))
}

pub(crate) fn memory_update_from_args(arguments: &Value) -> Result<MemoryUpdateResult, String> {
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
                            value.as_str().map(ToString::to_string).ok_or_else(|| {
                                "memory_update tags must be an array of strings".to_string()
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
        })
        .transpose()?;
    let profile = parse_optional_memory_profile(arguments)?;

    memory_update_with_profile(&root, id, title, content, tags.as_deref(), profile)
        .map_err(|error| format!("memory_update failed: {error}"))
}

pub(crate) fn memory_delete_from_args(arguments: &Value) -> Result<MemoryDeleteResult, String> {
    let root = parse_root(arguments)?;
    let id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "memory_delete requires a string id".to_string())?;

    memory_delete(&root, id).map_err(|error| format!("memory_delete failed: {error}"))
}

pub(crate) fn memory_export_from_args(arguments: &Value) -> Result<MemoryExportResult, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(50);

    memory_export(&root, limit).map_err(|error| format!("memory_export failed: {error}"))
}

pub(crate) fn memory_import_from_args(arguments: &Value) -> Result<MemoryImportResult, String> {
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

pub(crate) fn memory_search_from_args(arguments: &Value) -> Result<MemorySearchResult, String> {
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

pub(crate) fn context_run_detail_from_args(
    arguments: &Value,
) -> Result<Option<ContextAssembly>, String> {
    let root = parse_root(arguments)?;
    let generated_at_epoch_ms = arguments
        .get("generated_at_epoch_ms")
        .and_then(Value::as_u64)
        .map(u128::from)
        .ok_or_else(|| "context_run_detail requires a numeric generated_at_epoch_ms".to_string())?;

    context_run_detail(&root, generated_at_epoch_ms)
        .map_err(|error| format!("context_run_detail failed: {error}"))
}

pub(crate) fn context_run_history_from_args(
    arguments: &Value,
) -> Result<Vec<ContextAssembly>, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(3);

    context_run_history(&root, limit)
        .map_err(|error| format!("context_run_history failed: {error}"))
}

pub(crate) fn multi_repo_assemble_context_from_args(
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

pub(crate) fn assemble_context_from_args(arguments: &Value) -> Result<RetrievedContext, String> {
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

pub(crate) fn handoff_packet_from_args(arguments: &Value) -> Result<HandoffPacket, String> {
    let root = parse_root(arguments)?;
    let active_task = arguments
        .get("active_task")
        .and_then(Value::as_str)
        .ok_or_else(|| "handoff_packet requires active_task string".to_string())?;
    let mode = parse_retrieval_mode(arguments)?;
    let query = arguments.get("query").and_then(Value::as_str);
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(3);

    assemble_handoff_packet(&root, active_task, mode, query, limit)
        .map_err(|error| format!("handoff_packet failed: {error}"))
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

fn parse_memory_profile(arguments: &Value) -> Result<MemoryProfile, String> {
    parse_optional_memory_profile(arguments).map(|profile| profile.unwrap_or(MemoryProfile::Normal))
}

fn parse_optional_memory_profile(arguments: &Value) -> Result<Option<MemoryProfile>, String> {
    match arguments.get("profile").and_then(Value::as_str) {
        None => Ok(None),
        Some("normal") => Ok(Some(MemoryProfile::Normal)),
        Some("decision") => Ok(Some(MemoryProfile::Decision)),
        Some("guardrail") => Ok(Some(MemoryProfile::Guardrail)),
        Some(other) => Err(format!(
            "profile must be one of normal, decision, guardrail; got {other}"
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
