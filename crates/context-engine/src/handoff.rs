use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HandoffTemplate {
    #[default]
    General,
    BugFix,
    FeatureSlice,
    Review,
    Refactor,
    Release,
}

pub fn assemble_handoff_packet(
    root: &Path,
    active_task: &str,
    mode: RetrievalMode,
    query: Option<&str>,
    limit: usize,
) -> io::Result<HandoffPacket> {
    assemble_handoff_packet_with_template(
        root,
        active_task,
        HandoffTemplate::General,
        mode,
        query,
        limit,
    )
}

pub fn assemble_handoff_packet_with_template(
    root: &Path,
    active_task: &str,
    template: HandoffTemplate,
    mode: RetrievalMode,
    query: Option<&str>,
    limit: usize,
) -> io::Result<HandoffPacket> {
    let context = retrieve_context(root, mode, query, limit)?;
    let memory_decisions = context
        .memory_notes
        .iter()
        .filter(|note| {
            matches!(
                note.profile,
                MemoryProfile::Decision | MemoryProfile::Guardrail
            )
        })
        .cloned()
        .collect();
    let omissions = context.omissions.clone();

    Ok(HandoffPacket {
        active_task: active_task.trim().to_string(),
        template: template.clone(),
        template_focus: template_focus(&template),
        context,
        memory_decisions,
        validation_commands: validation_commands(),
        known_blockers: Vec::new(),
        omissions,
    })
}

pub fn parse_handoff_template(value: &str) -> Option<HandoffTemplate> {
    match value {
        "general" => Some(HandoffTemplate::General),
        "bug_fix" => Some(HandoffTemplate::BugFix),
        "feature_slice" => Some(HandoffTemplate::FeatureSlice),
        "review" => Some(HandoffTemplate::Review),
        "refactor" => Some(HandoffTemplate::Refactor),
        "release" => Some(HandoffTemplate::Release),
        _ => None,
    }
}

fn template_focus(template: &HandoffTemplate) -> Vec<String> {
    match template {
        HandoffTemplate::General => vec![
            "Summarize the active task, relevant context, decisions, blockers, and validation path."
                .to_string(),
        ],
        HandoffTemplate::BugFix => vec![
            "State the observed failure or regression.".to_string(),
            "Identify the smallest likely code path and related tests.".to_string(),
            "Call out reproduction steps and validation commands.".to_string(),
        ],
        HandoffTemplate::FeatureSlice => vec![
            "State the user-facing behavior being added.".to_string(),
            "List the owning modules and contracts expected to change.".to_string(),
            "Keep non-goals and rollout boundaries explicit.".to_string(),
        ],
        HandoffTemplate::Review => vec![
            "Prioritize defects, regressions, security risks, and missing tests.".to_string(),
            "Ground findings in concrete files or commands.".to_string(),
            "Separate findings from summary context.".to_string(),
        ],
        HandoffTemplate::Refactor => vec![
            "Name the responsibility being separated.".to_string(),
            "Preserve behavior while reducing coupling or file size.".to_string(),
            "Include focused before/after validation.".to_string(),
        ],
        HandoffTemplate::Release => vec![
            "Confirm shipped behavior, validation evidence, and known limitations.".to_string(),
            "Avoid publish, tag, deploy, or readiness claims outside the approved scope."
                .to_string(),
            "List exact release or rollback blockers.".to_string(),
        ],
    }
}

fn validation_commands() -> Vec<HandoffValidationCommand> {
    vec![
        HandoffValidationCommand {
            command: "cargo test -p context-engine".to_string(),
            reason: "Run when engine retrieval, memory, cache, repository state, or handoff behavior changes.".to_string(),
        },
        HandoffValidationCommand {
            command: "cargo test -p mcp-server".to_string(),
            reason: "Run when MCP, CLI, HTTP, tool schema, or adapter serialization changes.".to_string(),
        },
        HandoffValidationCommand {
            command: "npm --prefix web/controlplane run build".to_string(),
            reason: "Run when the control-plane truth mirror or frontend rendering changes.".to_string(),
        },
        HandoffValidationCommand {
            command: "powershell -NoProfile -ExecutionPolicy Bypass -File scripts\\check-line-counts.ps1".to_string(),
            reason: "Run after source changes to keep the 500-line maintainability guardrail visible.".to_string(),
        },
    ]
}
