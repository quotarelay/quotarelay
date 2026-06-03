use std::fs;

use super::common::temp_repo;
use crate::*;

#[test]
fn durable_memory_persists_and_reads_notes() {
    let root = temp_repo();

    let created = memory_write(
        &root,
        "Design note",
        "Durable memory must survive process restarts.",
        &["memory".to_string(), "design".to_string()],
    )
    .expect("memory write should succeed");

    let loaded = memory_read(&root, &created.note.id)
        .expect("memory read should succeed")
        .expect("note should exist");

    assert_eq!(loaded.title, "Design note");
    assert_eq!(loaded.tags, vec!["memory", "design"]);
}

#[test]
fn durable_memory_updates_and_deletes_notes() {
    let root = temp_repo();

    let created = memory_write(
        &root,
        "Design note",
        "Durable memory must survive process restarts.",
        &["memory".to_string(), "design".to_string()],
    )
    .expect("memory write should succeed");

    let updated = memory_update(
        &root,
        &created.note.id,
        Some("Updated note"),
        Some("Updated memory content."),
        Some(&["updated".to_string()]),
    )
    .expect("memory update should succeed");

    assert_eq!(updated.note.id, created.note.id);
    assert_eq!(updated.note.title, "Updated note");
    assert_eq!(updated.note.content, "Updated memory content.");
    assert_eq!(updated.note.tags, vec!["updated"]);
    assert!(updated.note.updated_at_epoch_ms >= created.note.updated_at_epoch_ms);

    let loaded = memory_read(&root, &created.note.id)
        .expect("memory read should succeed")
        .expect("updated note should exist");

    assert_eq!(loaded.title, "Updated note");
    assert_eq!(loaded.content, "Updated memory content.");
    assert_eq!(loaded.tags, vec!["updated"]);

    let deleted = memory_delete(&root, &created.note.id).expect("memory delete should succeed");
    assert_eq!(
        deleted.note.as_ref().map(|note| note.id.as_str()),
        Some(created.note.id.as_str())
    );
    assert!(memory_read(&root, &created.note.id)
        .expect("memory read should succeed")
        .is_none());
}

#[test]
fn durable_memory_exports_and_imports_bounded_json() {
    let source = temp_repo();
    let target = temp_repo();

    let first = memory_write(
        &source,
        "Design note",
        "Durable memory must survive migration.",
        &["memory".to_string()],
    )
    .expect("first memory write should succeed");
    memory_write(
        &source,
        "Second note",
        "Second migrated memory.",
        &["migration".to_string()],
    )
    .expect("second memory write should succeed");

    let exported = memory_export(&source, 10).expect("memory export should succeed");
    assert_eq!(exported.payload.notes.len(), 2);
    assert_eq!(exported.omitted_count, 0);

    let imported = memory_import(&target, exported.payload).expect("memory import should succeed");
    assert_eq!(imported.imported_count, 2);
    assert_eq!(imported.replaced_count, 0);
    assert_eq!(imported.omitted_count, 0);

    let loaded = memory_read(&target, &first.note.id)
        .expect("memory read should succeed")
        .expect("imported note should exist");
    assert_eq!(loaded.title, "Design note");
}

#[test]
fn durable_memory_import_replaces_duplicate_ids() {
    let root = temp_repo();

    let created = memory_write(
        &root,
        "Original note",
        "Original content.",
        &["original".to_string()],
    )
    .expect("memory write should succeed");

    let mut replacement = created.note.clone();
    replacement.title = "Replacement note".to_string();
    replacement.content = "Replacement content.".to_string();
    replacement.tags = vec!["replacement".to_string()];

    let imported = memory_import(
        &root,
        MemoryExportPayload {
            notes: vec![replacement],
        },
    )
    .expect("memory import should succeed");
    assert_eq!(imported.imported_count, 0);
    assert_eq!(imported.replaced_count, 1);
    assert_eq!(imported.omitted_count, 0);

    let loaded = memory_read(&root, &created.note.id)
        .expect("memory read should succeed")
        .expect("replacement note should exist");
    assert_eq!(loaded.title, "Replacement note");
    assert_eq!(loaded.content, "Replacement content.");
    assert_eq!(loaded.tags, vec!["replacement"]);
}

#[test]
fn durable_memory_search_is_bounded_and_persistent() {
    let root = temp_repo();
    for index in 0..6 {
        memory_write(
            &root,
            &format!("Memory {index}"),
            "Context memory entry",
            &["memory".to_string()],
        )
        .expect("memory write should succeed");
    }

    let results = memory_search(&root, "memory", 3).expect("memory search should succeed");

    assert_eq!(results.notes.len(), 3);
    assert_eq!(results.omitted_count, 3);
    assert!(root.join(".quotarelay").join("memory_notes.json").exists());
}

#[test]
fn exact_search_includes_bounded_memory_notes_with_typed_reasons() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");
    memory_write(
        &root,
        "Needle note",
        &format!("needle {}", "x".repeat(400)),
        &["memory".to_string()],
    )
    .expect("memory write should succeed");

    let assembly = assemble_context(&root, "needle", 2).expect("assembly should succeed");

    assert_eq!(assembly.memory_notes.len(), 1);
    assert_eq!(
        assembly.memory_notes[0].reason.kind,
        InclusionReasonKind::MemoryNoteMatch
    );
    assert!(assembly.memory_notes[0]
        .content
        .ends_with(TRUNCATED_PACK_MARKER));
    assert!(assembly
        .omissions
        .iter()
        .any(|item| item.kind == OmissionReasonKind::ByteBudgetReached));
}
