use std::path::PathBuf;

use context_engine::{context_run_history, memory_search, registered_repository_state};
use serde_json::{json, Value};

use crate::tools::current_tool_registry;

pub(crate) fn run_cli_usage<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = args.next().map(|arg| arg.as_ref().to_string());
    let memory_query = args.next().map(|arg| arg.as_ref().to_string());
    let repo_root = root.as_ref().map(PathBuf::from);
    let repo_count = match repo_root.as_ref() {
        Some(path) => registered_repository_state(path, 20)
            .map_err(|error| format!("usage failed: {error}"))?
            .len(),
        None => 0,
    };
    let recent_runs = match repo_root.as_ref() {
        Some(path) => context_run_history(path, 5)
            .map_err(|error| format!("usage failed: {error}"))?
            .len(),
        None => 0,
    };
    let memory_matches = match (repo_root.as_ref(), memory_query.as_deref()) {
        (Some(path), Some(query)) => memory_search(path, query, 3)
            .map_err(|error| format!("usage failed: {error}"))?
            .notes
            .len(),
        _ => 0,
    };
    let repo_state = configured_state(repo_root.is_some(), repo_count, "loaded");
    let context_state = configured_state(repo_root.is_some(), recent_runs, "recent");
    let memory_state = configured_state(memory_query.is_some(), memory_matches, "matches");

    Ok(json!({
        "title": "Quotarelay console usage",
        "status": format!(
            "Backend: Ready | Repo: {repo_state} | Context: {context_state} | Memory: {memory_state}"
        ),
        "tool_usage": tool_usage_rows(),
        "provider_calls": {
            "state": "none",
            "total": 0,
            "sparkline": [0, 0, 0, 0, 0, 0, 0, 0]
        },
        "system_health": {
            "provider_calls": "none",
            "cache": "hit proven",
            "mcp_tools": current_tool_registry().len(),
            "http_bind": "loopback"
        },
        "usage": {
            "repositories": repo_count,
            "recent_runs": recent_runs,
            "memory_matches": memory_matches
        },
        "collapsed": [
            {"label": "Recent runs", "value": context_state},
            {"label": "Memory matches", "value": memory_state},
            {"label": "Cache", "value": "hit proven"},
            {"label": "Boundaries", "value": "Provider calls: none"}
        ]
    }))
}

fn configured_state(configured: bool, count: usize, label: &str) -> String {
    if configured {
        format!("{count} {label}")
    } else {
        "Not configured".to_string()
    }
}

fn tool_usage_rows() -> Vec<Value> {
    let mut counts = [0; 5];
    for tool in current_tool_registry() {
        let name = tool["name"].as_str().unwrap_or_default();
        counts[tool_group_index(name)] += 1;
    }

    ["Context", "Repository", "Memory", "Cache", "Other"]
        .into_iter()
        .zip(counts)
        .map(|(label, count)| json!({"label": label, "count": count}))
        .collect()
}

fn tool_group_index(name: &str) -> usize {
    if name.contains("context")
        || name.contains("handoff")
        || name.contains("onboarding")
        || name.contains("savings")
    {
        0
    } else if name.contains("repo") || name.contains("workspace") || name.contains("policy") {
        1
    } else if name.contains("memory") {
        2
    } else if name.contains("cache") {
        3
    } else {
        4
    }
}
