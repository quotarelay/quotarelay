use serde_json::{json, Value};

pub(crate) fn context_feedback_write_tool() -> Value {
    json!({
        "name": "context_feedback_write",
        "description": "Records bounded local feedback for a generated context pack without changing retrieval behavior.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "generated_at_epoch_ms": { "type": "integer", "minimum": 0 },
                "rating": { "type": "string", "enum": ["useful", "not_useful"] },
                "reason": { "type": "string" }
            },
            "required": ["root", "generated_at_epoch_ms", "rating", "reason"],
            "additionalProperties": false
        }
    })
}

pub(crate) fn context_feedback_list_tool() -> Value {
    json!({
        "name": "context_feedback_list",
        "description": "Returns bounded local context feedback records without changing retrieval ranking.",
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

pub(crate) fn validation_recommend_tool() -> Value {
    json!({
        "name": "validation_recommend",
        "description": "Returns exact local validation commands with reasons for touched or queried paths.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": ["paths"],
            "additionalProperties": false
        }
    })
}

pub(crate) fn team_policy_profile_save_tool() -> Value {
    json!({
        "name": "team_policy_profile_save",
        "description": "Persists a local team policy profile for guardrails, validation recipes, MCP presets, and source-upload preference.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "root": { "type": "string" },
                "name": { "type": "string" },
                "guardrails": { "type": "array", "items": { "type": "string" }, "maxItems": 20 },
                "validation_recipes": { "type": "array", "items": { "type": "string" }, "maxItems": 20 },
                "mcp_client_presets": { "type": "array", "items": { "type": "string" }, "maxItems": 20 },
                "allow_source_upload": { "type": "boolean", "default": false }
            },
            "required": ["root", "name"],
            "additionalProperties": false
        }
    })
}

pub(crate) fn team_policy_profile_list_tool() -> Value {
    json!({
        "name": "team_policy_profile_list",
        "description": "Lists bounded local team policy profiles without uploading source code or local state.",
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
