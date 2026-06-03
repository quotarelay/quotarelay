use context_engine::EngineInfo;
use serde_json::{json, Value};

use crate::args::*;
use crate::feedback_args::*;
use crate::validation_args::*;

pub(crate) fn bootstrap_tool_call(request: &Value) -> Value {
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
        Some("repo_map") => match repo_map_from_args(&arguments) {
            Ok(map) => json!({
                "type": "text",
                "text": serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
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
        Some("repository_update_metadata") => {
            match repository_update_metadata_from_args(&arguments) {
                Ok(result) => json!({
                    "type": "text",
                    "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
                }),
                Err(error) => json!({
                    "type": "text",
                    "text": error
                }),
            }
        }
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
        Some("context_feedback_write") => match context_feedback_write_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("context_feedback_list") => match context_feedback_list_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("validation_recommend") => match validation_recommend_from_args(&arguments) {
            Ok(result) => json!({
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
            }),
            Err(error) => json!({
                "type": "text",
                "text": error
            }),
        },
        Some("multi_repo_assemble_context") => {
            match multi_repo_assemble_context_from_args(&arguments) {
                Ok(results) => json!({
                    "type": "text",
                    "text": serde_json::to_string(&results).unwrap_or_else(|_| "{}".to_string())
                }),
                Err(error) => json!({
                    "type": "text",
                    "text": error
                }),
            }
        }
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
        Some("handoff_packet") => match handoff_packet_from_args(&arguments) {
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
