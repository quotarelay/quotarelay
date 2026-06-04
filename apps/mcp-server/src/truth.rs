use context_engine::{retrieval_truth, RetrievalTruth};
use serde_json::Value;

use crate::tools::current_tool_registry;

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct BackendTruthPayload {
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
    team_policy_profiles_enabled: bool,
    max_team_policy_profiles: usize,
    max_team_policy_items: usize,
    team_policy_source_upload_default: bool,
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

pub(crate) fn backend_truth_payload() -> BackendTruthPayload {
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
                id: "team_policy_profiles",
                command: "cargo test -p mcp-server team_policy_profiles_save_list_and_inspect_over_stdio",
            },
            BackendProof {
                id: "team_policy_cli",
                command: "cargo test -p mcp-server local_cli_team_policy_save_and_list_are_stable_json",
            },
            BackendProof {
                id: "savings_report",
                command: "cargo test -p mcp-server savings_report_works_over_stdio_without_snippets",
            },
            BackendProof {
                id: "repository_state",
                command:
                    "cargo test -p mcp-server repository_state_tool_reports_sync_and_recent_run_truth_over_stdio",
            },
            BackendProof {
                id: "local_operator_workflow",
                command:
                    "cargo test -p mcp-server local_operator_workflow_is_visible_through_truth_and_stdio",
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
        team_policy_profiles_enabled: true,
        max_team_policy_profiles: 20,
        max_team_policy_items: 20,
        team_policy_source_upload_default: false,
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
                label: "Repo map",
                command: "cargo run -p mcp-server -- --cli map <repo-root>",
            },
            CliCommandTruth {
                label: "Recommend validation",
                command: "cargo run -p mcp-server -- --cli validate <path> [path...]",
            },
            CliCommandTruth {
                label: "Record context feedback",
                command:
                    "cargo run -p mcp-server -- --cli feedback-write <repo-root> <run-ms> useful <reason>",
            },
            CliCommandTruth {
                label: "List context feedback",
                command: "cargo run -p mcp-server -- --cli feedback-list <repo-root> [limit]",
            },
            CliCommandTruth {
                label: "Savings report",
                command: "cargo run -p mcp-server -- --cli savings-report <repo-root> [limit]",
            },
            CliCommandTruth {
                label: "Assemble context",
                command: "cargo run -p mcp-server -- --cli assemble <repo-root> <query>",
            },
            CliCommandTruth {
                label: "Agent handoff",
                command:
                    "cargo run -p mcp-server -- --cli handoff <repo-root> <active-task> exact_search <query>",
            },
            CliCommandTruth {
                label: "Templated handoff",
                command:
                    "cargo run -p mcp-server -- --cli handoff-template <repo-root> <active-task> review exact_search <query>",
            },
        ],
    }
}
