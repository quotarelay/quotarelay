use std::io;
use std::path::Path;

use super::*;

pub fn assemble_handoff_packet(
    root: &Path,
    active_task: &str,
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
        context,
        memory_decisions,
        validation_commands: validation_commands(),
        known_blockers: Vec::new(),
        omissions,
    })
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
