use serde_json::{json, Value};

use crate::extra_tools::{
    context_feedback_list_tool, context_feedback_write_tool, team_policy_profile_list_tool,
    team_policy_profile_save_tool, validation_recommend_tool,
};

pub(crate) fn current_tool_registry() -> Vec<Value> {
    vec![
        bootstrap_tool(),
        sync_repo_tool(),
        repo_inventory_tool(),
        repo_map_tool(),
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
        team_policy_profile_save_tool(),
        team_policy_profile_list_tool(),
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
        context_feedback_write_tool(),
        context_feedback_list_tool(),
        validation_recommend_tool(),
        multi_repo_assemble_context_tool(),
        assemble_context_tool(),
        handoff_packet_tool(),
    ]
}

fn root_only_tool(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
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

fn id_tool(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
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
    root_only_tool(
        "sync_repo",
        "Scans a repository and persists a local search index under .quotarelay/index.json.",
    )
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
    root_only_tool(
        "repo_inventory",
        "Returns bounded repository inventory truth from the persisted local repository index.",
    )
}

fn repo_map_tool() -> Value {
    root_only_tool(
        "repo_map",
        "Returns a bounded repository map with top-level directories and detected Rust symbols.",
    )
}

fn inspect_local_state_tool() -> Value {
    root_only_tool(
        "inspect_local_state",
        "Returns bounded local Quotarelay state presence and counts without dumping persisted contents.",
    )
}

fn cache_inspect_tool() -> Value {
    root_only_tool(
        "cache_inspect",
        "Returns retrieval cache file presence and counts without reporting hit rates or optimization metrics.",
    )
}

fn cache_clear_tool() -> Value {
    root_only_tool(
        "cache_clear",
        "Explicitly clears local retrieval cache files owned by the context engine.",
    )
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
                "default_mode": { "type": "string", "enum": ["exact_search", "overview", "task_capsule", "diff_aware"] },
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
                "tags": { "type": "array", "items": { "type": "string" } },
                "profile": { "type": "string", "enum": ["normal", "decision", "guardrail"], "default": "normal" }
            },
            "required": ["root", "title", "content"],
            "additionalProperties": false
        }
    })
}

fn memory_read_tool() -> Value {
    id_tool(
        "memory_read",
        "Reads one durable memory note by id from the local Quotarelay state directory.",
    )
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
                "tags": { "type": "array", "items": { "type": "string" } },
                "profile": { "type": "string", "enum": ["normal", "decision", "guardrail"] }
            },
            "required": ["root", "id"],
            "additionalProperties": false
        }
    })
}

fn memory_delete_tool() -> Value {
    id_tool(
        "memory_delete",
        "Deletes one durable memory note by id from the local Quotarelay state directory.",
    )
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
                    "enum": ["exact_search", "overview", "task_capsule", "diff_aware"],
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

fn handoff_packet_tool() -> Value {
    json!({
        "name": "handoff_packet",
        "description": "Builds a bounded agent handoff packet with active task, context, memory notes, validation commands, blockers, and omission reasons.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "active_task": { "type": "string" },
                "mode": {
                    "type": "string",
                    "enum": ["exact_search", "overview", "task_capsule", "diff_aware"],
                    "default": "exact_search"
                },
                "template": {
                    "type": "string",
                    "enum": ["general", "bug_fix", "feature_slice", "review", "refactor", "release"],
                    "default": "general"
                },
                "query": { "type": "string" },
                "limit": { "type": "integer", "minimum": 1, "maximum": 5 }
            },
            "required": ["root", "active_task"],
            "additionalProperties": false
        }
    })
}
