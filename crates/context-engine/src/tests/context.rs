use std::fs;
use std::thread;
use std::time::Duration;

use super::common::temp_repo;
use crate::*;

#[test]
fn banner_is_stable() {
    let info = EngineInfo::quotarelay();

    assert_eq!(info.banner(), "quotarelay bootstrap");
}

#[test]
fn assembled_context_is_bounded_and_explained() {
    let root = temp_repo();
    fs::write(
        root.join("alpha.txt"),
        "needle one\nneedle two\nneedle three\n",
    )
    .expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let assembly = assemble_context(&root, "needle", 2).expect("assembly should succeed");

    assert_eq!(assembly.snippets.len(), 2);
    assert!(assembly.memory_notes.is_empty());
    assert_eq!(assembly.snippets[0].path, "alpha.txt");
    assert_eq!(
        assembly.snippets[0].reason.kind,
        InclusionReasonKind::QueryLineMatch
    );
    assert!(assembly.snippets[0]
        .reason
        .detail
        .contains("query 'needle'"));
    assert_eq!(
        assembly.omissions[0].kind,
        OmissionReasonKind::ItemLimitReached
    );
    assert!(assembly.omissions[0].detail.contains("limit 2 was reached"));
}

#[test]
fn large_repo_context_assembly_preserves_limits_and_omissions() {
    let root = temp_repo();
    for index in 0..30 {
        fs::write(
            root.join(format!("note-{index:02}.txt")),
            format!("needle line {index}\n"),
        )
        .expect("note file should write");
        fs::write(
            root.join(format!("widget-{index:02}.rs")),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("widget file should write");
    }
    repo_index::sync_repo(&root).expect("sync should succeed");

    let exact = assemble_context(&root, "needle", 50).expect("exact assembly should succeed");
    assert_eq!(exact.snippets.len(), 5);
    assert!(exact
        .omissions
        .iter()
        .any(|omission| omission.kind == OmissionReasonKind::ItemLimitReached));

    let overview = assemble_overview(&root, 50).expect("overview should succeed");
    assert_eq!(overview.documents.len(), 5);
    assert!(overview
        .omissions
        .iter()
        .any(|omission| omission.kind == OmissionReasonKind::ItemLimitReached));

    let task = assemble_task_capsule(&root, "Widget", 50).expect("task capsule should succeed");
    assert_eq!(task.documents.len(), 5);
    assert!(task
        .omissions
        .iter()
        .any(|omission| omission.kind == OmissionReasonKind::ItemLimitReached));
}

#[test]
fn underspecified_retrieval_returns_bounded_clarification() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let missing = retrieve_context(&root, RetrievalMode::ExactSearch, None, 3)
        .expect("missing query should clarify");
    assert!(missing.snippets.is_empty());
    assert_eq!(missing.cache_status.kind, CacheStatusKind::NotApplicable);
    assert!(
        missing
            .clarification
            .as_ref()
            .expect("clarification should exist")
            .questions
            .len()
            <= 3
    );

    let broad = retrieve_context(&root, RetrievalMode::TaskCapsule, Some("fix"), 3)
        .expect("broad query should clarify");
    assert_eq!(broad.documents.len(), 0);
    assert!(broad
        .clarification
        .as_ref()
        .expect("broad clarification should exist")
        .reason
        .contains("too broad"));

    let precise = retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 3)
        .expect("precise query should retrieve");
    assert!(precise.clarification.is_none());
    assert_eq!(precise.snippets.len(), 1);
}

#[test]
fn context_run_history_is_bounded_and_keeps_omission_reasons() {
    let root = temp_repo();
    fs::write(
        root.join("alpha.txt"),
        "needle one\nneedle two\nneedle three\n",
    )
    .expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    assemble_context(&root, "needle", 2).expect("first assembly should succeed");
    assemble_context(&root, "missing", 2).expect("second assembly should succeed");

    let history = context_run_history(&root, 1).expect("history should succeed");

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].query, "missing");
    assert!(history[0].snippets.is_empty());
    assert!(history[0].memory_notes.is_empty());
    assert!(history[0]
        .omissions
        .iter()
        .any(|item| item.kind == OmissionReasonKind::NoLineMatches));
    assert!(history[0]
        .omissions
        .iter()
        .any(|item| item.kind == OmissionReasonKind::NoMemoryMatches));
}

#[test]
fn context_run_detail_reads_one_recent_run_by_timestamp() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let run = assemble_context(&root, "needle", 2).expect("assembly should succeed");
    let detail = context_run_detail(&root, run.generated_at_epoch_ms)
        .expect("detail lookup should succeed")
        .expect("run detail should exist");

    assert_eq!(detail.generated_at_epoch_ms, run.generated_at_epoch_ms);
    assert_eq!(
        detail.snippets[0].reason.kind,
        InclusionReasonKind::QueryLineMatch
    );
    assert!(context_run_detail(&root, run.generated_at_epoch_ms + 1)
        .expect("missing detail lookup should succeed")
        .is_none());
}

#[test]
fn assembled_context_prefers_rust_capsules_over_function_bodies() {
    let root = temp_repo();
    fs::write(
            root.join("lib.rs"),
            "use std::fmt;\n\nstruct Widget {\n    id: usize,\n}\n\nfn plan() -> usize {\n    let body_only_term = 41;\n    body_only_term\n}\n",
        )
        .expect("rust file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let assembly = assemble_context(&root, "Widget", 2).expect("assembly should succeed");
    assert_eq!(assembly.snippets.len(), 1);
    assert_eq!(assembly.snippets[0].path, "lib.rs");
    assert!(assembly.snippets[0]
        .line
        .contains("struct Widget { id: usize }"));

    let body_query =
        assemble_context(&root, "body_only_term", 2).expect("body query should succeed");
    assert!(body_query.snippets.is_empty());
}

#[test]
fn overview_capsule_returns_bounded_indexed_documents() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
    fs::write(root.join("beta.txt"), "beta overview\n").expect("beta file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let overview = assemble_overview(&root, 1).expect("overview should succeed");

    assert_eq!(overview.documents.len(), 1);
    assert_eq!(overview.documents[0].path, "alpha.txt");
    assert_eq!(overview.documents[0].contents, "alpha overview\n");
    assert_eq!(
        overview.documents[0].reason.kind,
        InclusionReasonKind::OverviewDocument
    );
    assert_eq!(
        overview.omissions[0].kind,
        OmissionReasonKind::ItemLimitReached
    );
}

#[test]
fn task_capsule_prefers_structural_capsules_and_falls_back_to_raw_text() {
    let root = temp_repo();
    fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n\nfn plan() -> usize {\n    let body_only_term = 41;\n    body_only_term\n}\n",
        )
        .expect("rust file should write");
    fs::write(
        root.join("broken.rs"),
        "fn broken( {\n    let raw_fallback_term = 1;\n}\n",
    )
    .expect("broken rust file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let capsule = assemble_task_capsule(&root, "Widget", 2).expect("task capsule should succeed");
    assert_eq!(capsule.documents.len(), 1);
    assert_eq!(capsule.documents[0].path, "lib.rs");
    assert_eq!(
        capsule.documents[0].reason.kind,
        InclusionReasonKind::TaskCapsuleMatch
    );
    assert!(capsule.documents[0]
        .contents
        .contains("struct Widget { id: usize }"));
    assert!(!capsule.documents[0].contents.contains("body_only_term"));

    let fallback = assemble_task_capsule(&root, "raw_fallback_term", 2)
        .expect("fallback capsule should succeed");
    assert_eq!(fallback.documents.len(), 1);
    assert_eq!(fallback.documents[0].path, "broken.rs");
    assert!(fallback.documents[0].contents.contains("raw_fallback_term"));
}

#[test]
fn retrieve_context_exposes_explicit_modes() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "alpha needle\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let exact = retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 2)
        .expect("exact_search should succeed");
    assert_eq!(exact.mode, RetrievalMode::ExactSearch);
    assert_eq!(exact.query.as_deref(), Some("needle"));
    assert_eq!(exact.snippets.len(), 1);
    assert!(exact.memory_notes.is_empty());
    assert!(exact.documents.is_empty());

    let overview =
        retrieve_context(&root, RetrievalMode::Overview, None, 2).expect("overview should succeed");
    assert_eq!(overview.mode, RetrievalMode::Overview);
    assert!(overview.memory_notes.is_empty());
    assert_eq!(overview.documents.len(), 1);
    assert!(overview.snippets.is_empty());

    let missing = retrieve_context(&root, RetrievalMode::TaskCapsule, None, 2)
        .expect("task_capsule without query should clarify");
    assert_eq!(missing.mode, RetrievalMode::TaskCapsule);
    assert_eq!(missing.cache_status.kind, CacheStatusKind::NotApplicable);
    assert!(missing.clarification.is_some());
    assert!(missing.documents.is_empty());
}

#[test]
fn context_outputs_include_local_budget_estimates() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle bytes\n").expect("alpha file should write");
    fs::write(root.join("beta.txt"), "overview bytes\n").expect("beta file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let exact = retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 2)
        .expect("exact_search should succeed");
    assert_eq!(exact.budget.included_bytes, "needle bytes".len());
    assert_eq!(exact.budget.raw_bytes_considered, "needle bytes".len());
    assert_eq!(exact.budget.approximate_tokens, 3);
    assert_eq!(exact.budget.estimated_reduction_ratio, 0.0);

    let overview =
        retrieve_context(&root, RetrievalMode::Overview, None, 1).expect("overview should succeed");
    assert_eq!(
        overview.budget.included_bytes,
        overview.documents[0].contents.len()
    );
    assert_eq!(
        overview.budget.approximate_tokens,
        overview.budget.included_bytes.div_ceil(4)
    );
    assert!(overview.budget.raw_bytes_considered >= overview.budget.included_bytes);
    assert!(overview.budget.estimated_reduction_ratio >= 0.0);
    assert!(overview.budget.estimated_reduction_ratio <= 1.0);
}

#[test]
fn context_outputs_report_fresh_stale_and_resynced_state() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle original\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let fresh = retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 2)
        .expect("fresh context should succeed");
    assert_eq!(fresh.stale.is_stale, false);

    thread::sleep(Duration::from_millis(20));
    fs::write(root.join("alpha.txt"), "needle changed\n").expect("alpha file should rewrite");
    fs::write(root.join("beta.txt"), "needle new\n").expect("beta file should write");

    let stale = retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 2)
        .expect("stale context should still return");
    assert_eq!(stale.stale.is_stale, true);
    assert_eq!(stale.stale.changed_files, 1);
    assert_eq!(stale.stale.new_files, 1);

    repo_index::sync_repo(&root).expect("resync should succeed");
    let resynced = retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 2)
        .expect("resynced context should succeed");
    assert_eq!(resynced.stale.is_stale, false);
}

#[test]
fn context_outputs_report_missing_indexed_files_as_stale() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "needle original\n").expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");
    fs::remove_file(root.join("alpha.txt")).expect("alpha file should be removed");

    let stale = retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 2)
        .expect("stale cached context should still return");

    assert_eq!(stale.stale.is_stale, true);
    assert_eq!(stale.stale.missing_files, 1);
}

#[test]
fn diff_aware_context_reports_clean_dirty_and_many_file_states() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "alpha original\n").expect("alpha file should write");
    fs::write(root.join("related.txt"), "related original\n").expect("related file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let clean = retrieve_context(&root, RetrievalMode::DiffAware, None, 2)
        .expect("clean diff-aware context should succeed");
    assert!(clean.documents.is_empty());
    assert_eq!(clean.stale.is_stale, false);
    assert_eq!(
        clean.omissions[0].kind,
        OmissionReasonKind::NoDocumentMatches
    );

    thread::sleep(Duration::from_millis(20));
    fs::write(root.join("alpha.txt"), "alpha changed needle\n").expect("alpha file should rewrite");
    fs::write(root.join("beta.txt"), "beta new needle\n").expect("beta file should write");

    let dirty = retrieve_context(&root, RetrievalMode::DiffAware, Some("related"), 5)
        .expect("dirty diff-aware context should succeed");
    assert_eq!(dirty.stale.is_stale, true);
    assert!(dirty
        .documents
        .iter()
        .any(|document| document.reason.kind == InclusionReasonKind::DiffChangedFile));
    assert!(dirty
        .documents
        .iter()
        .any(|document| document.reason.kind == InclusionReasonKind::DiffRelatedMatch));

    for index in 0..8 {
        fs::write(root.join(format!("new-{index}.txt")), "extra new\n")
            .expect("extra file should write");
    }
    let bounded = retrieve_context(&root, RetrievalMode::DiffAware, None, 3)
        .expect("bounded diff-aware context should succeed");
    assert_eq!(bounded.documents.len(), 3);
    assert!(bounded
        .omissions
        .iter()
        .any(|omission| omission.kind == OmissionReasonKind::ItemLimitReached));
}

#[test]
fn exact_search_reports_byte_budget_omission() {
    let root = temp_repo();
    let long_line = format!("needle {}", "x".repeat(300));
    fs::write(root.join("alpha.txt"), format!("{long_line}\n")).expect("alpha file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let assembly = assemble_context(&root, "needle", 2).expect("assembly should succeed");

    assert_eq!(assembly.snippets.len(), 1);
    assert!(assembly.snippets[0].line.ends_with(TRUNCATED_PACK_MARKER));
    assert_eq!(
        assembly.omissions[0].kind,
        OmissionReasonKind::ByteBudgetReached
    );
}

#[test]
fn task_capsule_reports_document_byte_budget_omission() {
    let root = temp_repo();
    let mut rust_file = String::new();
    for index in 0..80 {
        rust_file.push_str(&format!(
            "fn widget_{index}() -> usize {{\n    {index}\n}}\n\n"
        ));
    }
    fs::write(root.join("lib.rs"), rust_file).expect("rust file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let capsule = assemble_task_capsule(&root, "widget_0", 2).expect("task capsule should succeed");

    assert_eq!(capsule.documents.len(), 1);
    assert!(capsule.documents[0]
        .contents
        .ends_with(TRUNCATED_PACK_MARKER));
    assert_eq!(
        capsule.omissions[0].kind,
        OmissionReasonKind::ByteBudgetReached
    );
}

#[test]
fn task_capsule_prefers_symbol_matches_before_raw_text_mentions() {
    let root = temp_repo();
    fs::write(root.join("alpha.txt"), "Widget appears in planning notes\n")
        .expect("alpha file should write");
    fs::write(root.join("lib.rs"), "struct Widget {\n    id: usize,\n}\n")
        .expect("rust file should write");
    repo_index::sync_repo(&root).expect("sync should succeed");

    let capsule = assemble_task_capsule(&root, "Widget", 2).expect("task capsule should succeed");

    assert_eq!(capsule.documents[0].path, "lib.rs");
}
