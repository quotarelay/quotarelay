use std::fs;

use super::common::temp_repo;
use crate::*;

#[test]
fn handoff_packet_is_bounded_and_explained() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle handoff\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");
    memory_write_with_profile(
        &root,
        "Decision note",
        "needle memory decision",
        &["decision".to_string()],
        MemoryProfile::Decision,
    )
    .expect("memory note should write");

    let packet = assemble_handoff_packet(
        &root,
        "Continue T102",
        RetrievalMode::ExactSearch,
        Some("needle"),
        2,
    )
    .expect("handoff should succeed");

    assert_eq!(packet.active_task, "Continue T102");
    assert_eq!(packet.context.mode, RetrievalMode::ExactSearch);
    assert_eq!(packet.context.snippets.len(), 1);
    assert_eq!(packet.memory_decisions.len(), 1);
    assert_eq!(packet.memory_decisions[0].title, "Decision note");
    assert!(packet
        .validation_commands
        .iter()
        .any(|command| command.command == "cargo test -p context-engine"));
    assert!(packet.known_blockers.is_empty());
    assert_eq!(packet.omissions.len(), packet.context.omissions.len());
    assert!(packet.context.budget.included_bytes > 0);
}
