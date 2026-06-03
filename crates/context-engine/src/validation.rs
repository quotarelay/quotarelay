use super::*;

const MAX_VALIDATION_PATHS: usize = 20;

#[derive(Clone, Copy)]
struct ValidationRule {
    command: &'static str,
    reason: &'static str,
    matches: fn(&str) -> bool,
}

pub fn recommend_validation(paths: &[String]) -> ValidationRecommendationResult {
    let capped_paths = paths.iter().take(MAX_VALIDATION_PATHS).collect::<Vec<_>>();
    let omitted_path_count = paths.len().saturating_sub(MAX_VALIDATION_PATHS);
    let mut recommendations = Vec::new();

    for rule in validation_rules() {
        let matched_paths = capped_paths
            .iter()
            .filter(|path| (rule.matches)(&normalize_validation_path(path)))
            .map(|path| (*path).clone())
            .collect::<Vec<_>>();

        if !matched_paths.is_empty() {
            recommendations.push(ValidationRecommendation {
                command: rule.command.to_string(),
                reason: rule.reason.to_string(),
                matched_paths,
            });
        }
    }

    if recommendations.is_empty() {
        recommendations.push(ValidationRecommendation {
            command: "powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\clean-check.ps1"
                .to_string(),
            reason: "No repo-owned path rule matched; run the local clean check.".to_string(),
            matched_paths: capped_paths.into_iter().cloned().collect(),
        });
    }

    ValidationRecommendationResult {
        recommendations,
        omitted_path_count,
    }
}

fn validation_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule {
            command: "cargo test -p repo-index",
            reason: "Repo-index files changed.",
            matches: |path| path.starts_with("crates/repo-index/"),
        },
        ValidationRule {
            command: "cargo test -p context-engine",
            reason: "Context-engine files changed.",
            matches: |path| path.starts_with("crates/context-engine/"),
        },
        ValidationRule {
            command: "cargo test -p mcp-server",
            reason: "MCP server files changed.",
            matches: |path| path.starts_with("apps/mcp-server/"),
        },
        ValidationRule {
            command: "npm --prefix web/controlplane run build",
            reason: "Control-plane files changed.",
            matches: |path| path.starts_with("web/controlplane/"),
        },
        ValidationRule {
            command:
                "powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\check-line-counts.ps1",
            reason: "Source or docs changed; keep the local guardrail green.",
            matches: |path| {
                path.starts_with("apps/")
                    || path.starts_with("crates/")
                    || path.starts_with("web/controlplane/src/")
                    || path.starts_with("docs/")
                    || path == "README.md"
            },
        },
    ]
}

fn normalize_validation_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}
