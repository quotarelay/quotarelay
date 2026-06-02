use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use repo_index::{indexed_documents, matching_documents, search_code};
use serde::{Deserialize, Serialize};

const MAX_SNIPPET_PACK_BYTES: usize = 160;
const MAX_DOCUMENT_PACK_BYTES: usize = 640;
const TRUNCATED_PACK_MARKER: &str = "...";

pub struct EngineInfo {
    name: &'static str,
    mode: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMode {
    ExactSearch,
    Overview,
    TaskCapsule,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InclusionReasonKind {
    QueryLineMatch,
    OverviewDocument,
    TaskCapsuleMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InclusionReason {
    pub kind: InclusionReasonKind,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OmissionReasonKind {
    NoLineMatches,
    NoDocumentMatches,
    ItemLimitReached,
    ByteBudgetReached,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OmissionReason {
    pub kind: OmissionReasonKind,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnippet {
    pub path: String,
    pub line_number: usize,
    pub line: String,
    pub reason: InclusionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAssembly {
    pub query: String,
    pub generated_at_epoch_ms: u128,
    pub snippets: Vec<ContextSnippet>,
    pub omissions: Vec<OmissionReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDocument {
    pub path: String,
    pub contents: String,
    pub reason: InclusionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCapsule {
    pub generated_at_epoch_ms: u128,
    pub documents: Vec<ContextDocument>,
    pub omissions: Vec<OmissionReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedContext {
    pub mode: RetrievalMode,
    pub query: Option<String>,
    pub generated_at_epoch_ms: u128,
    pub snippets: Vec<ContextSnippet>,
    pub documents: Vec<ContextDocument>,
    pub omissions: Vec<OmissionReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoredRunHistory {
    runs: Vec<ContextAssembly>,
}

impl EngineInfo {
    pub fn quotarelay() -> Self {
        Self {
            name: "quotarelay",
            mode: "bootstrap",
        }
    }

    pub fn banner(&self) -> String {
        format!("{} {}", self.name, self.mode)
    }
}

pub fn assemble_context(root: &Path, query: &str, limit: usize) -> io::Result<ContextAssembly> {
    let capped_limit = limit.clamp(1, 5);
    let hits = search_code(root, query, capped_limit + 1)?;
    let (snippets, omissions) = pack_snippets(query, hits, capped_limit);
    let assembly = ContextAssembly {
        query: query.to_string(),
        generated_at_epoch_ms: now_epoch_ms()?,
        snippets,
        omissions,
    };

    append_history(root, &assembly)?;

    Ok(assembly)
}

pub fn context_run_history(root: &Path, limit: usize) -> io::Result<Vec<ContextAssembly>> {
    let capped_limit = limit.clamp(1, 5);
    let history = load_history(root)?;

    Ok(history
        .runs
        .into_iter()
        .rev()
        .take(capped_limit)
        .collect())
}

pub fn retrieve_context(
    root: &Path,
    mode: RetrievalMode,
    query: Option<&str>,
    limit: usize,
) -> io::Result<RetrievedContext> {
    match mode {
        RetrievalMode::ExactSearch => {
            let query = query.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "exact_search requires a query")
            })?;
            let assembly = assemble_context(root, query, limit)?;
            Ok(RetrievedContext {
                mode,
                query: Some(assembly.query.clone()),
                generated_at_epoch_ms: assembly.generated_at_epoch_ms,
                snippets: assembly.snippets,
                documents: Vec::new(),
                omissions: assembly.omissions,
            })
        }
        RetrievalMode::Overview => {
            let capsule = assemble_overview(root, limit)?;
            Ok(RetrievedContext {
                mode,
                query: None,
                generated_at_epoch_ms: capsule.generated_at_epoch_ms,
                snippets: Vec::new(),
                documents: capsule.documents,
                omissions: capsule.omissions,
            })
        }
        RetrievalMode::TaskCapsule => {
            let query = query.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "task_capsule requires a query")
            })?;
            let capsule = assemble_task_capsule(root, query, limit)?;
            Ok(RetrievedContext {
                mode,
                query: Some(query.to_string()),
                generated_at_epoch_ms: capsule.generated_at_epoch_ms,
                snippets: Vec::new(),
                documents: capsule.documents,
                omissions: capsule.omissions,
            })
        }
    }
}

pub fn assemble_overview(root: &Path, limit: usize) -> io::Result<ContextCapsule> {
    let capped_limit = limit.clamp(1, 5);
    let documents = indexed_documents(root, capped_limit + 1)?;
    let (documents, omissions) = pack_documents(
        documents,
        capped_limit,
        InclusionReasonKind::OverviewDocument,
        "Indexed document supports repository overview.",
        "repository overview",
    );
    Ok(ContextCapsule {
        generated_at_epoch_ms: now_epoch_ms()?,
        documents,
        omissions,
    })
}

pub fn assemble_task_capsule(root: &Path, query: &str, limit: usize) -> io::Result<ContextCapsule> {
    let capped_limit = limit.clamp(1, 5);
    let documents = matching_documents(root, query, capped_limit + 1)?;
    let reason_detail = format!("Indexed document contents matched query '{query}'.");
    let scope = format!("query '{query}'");
    let (documents, omissions) = pack_documents(
        documents,
        capped_limit,
        InclusionReasonKind::TaskCapsuleMatch,
        &reason_detail,
        &scope,
    );
    Ok(ContextCapsule {
        generated_at_epoch_ms: now_epoch_ms()?,
        documents,
        omissions,
    })
}

fn pack_snippets(query: &str, hits: Vec<repo_index::SearchHit>, limit: usize) -> (Vec<ContextSnippet>, Vec<OmissionReason>) {
    let total_hits = hits.len();
    let mut snippets = Vec::new();
    let mut omissions = Vec::new();
    let mut remaining_bytes = MAX_SNIPPET_PACK_BYTES;
    let mut byte_budget_hit = false;

    for hit in hits.into_iter().take(limit) {
        let Some((line, truncated)) = fit_within_budget(&hit.line, &mut remaining_bytes) else {
            byte_budget_hit = true;
            break;
        };
        snippets.push(ContextSnippet {
            path: hit.path,
            line_number: hit.line_number,
            line,
            reason: InclusionReason {
                kind: InclusionReasonKind::QueryLineMatch,
                detail: format!("Indexed line matched query '{query}'."),
            },
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if snippets.is_empty() && total_hits == 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoLineMatches,
            detail: format!("No indexed lines matched query '{query}'."),
        });
    }
    if total_hits > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!("Additional matching lines omitted because item limit {limit} was reached."),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!("Packed matching lines reached byte budget {MAX_SNIPPET_PACK_BYTES}."),
        });
    }

    (snippets, omissions)
}

fn pack_documents(
    documents: Vec<repo_index::IndexedDocument>,
    limit: usize,
    reason_kind: InclusionReasonKind,
    reason_detail: &str,
    scope: &str,
) -> (Vec<ContextDocument>, Vec<OmissionReason>) {
    let total_documents = documents.len();
    let mut packed = Vec::new();
    let mut omissions = Vec::new();
    let mut remaining_bytes = MAX_DOCUMENT_PACK_BYTES;
    let mut byte_budget_hit = false;

    for document in documents.into_iter().take(limit) {
        let Some((contents, truncated)) = fit_within_budget(&document.contents, &mut remaining_bytes) else {
            byte_budget_hit = true;
            break;
        };
        packed.push(ContextDocument {
            path: document.path,
            contents,
            reason: InclusionReason {
                kind: reason_kind.clone(),
                detail: reason_detail.to_string(),
            },
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if packed.is_empty() && total_documents == 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoDocumentMatches,
            detail: format!("No indexed documents matched {scope}."),
        });
    }
    if total_documents > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!("Additional indexed documents omitted because item limit {limit} was reached."),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!("Packed indexed documents reached byte budget {MAX_DOCUMENT_PACK_BYTES}."),
        });
    }

    (packed, omissions)
}

fn fit_within_budget(value: &str, remaining_bytes: &mut usize) -> Option<(String, bool)> {
    if *remaining_bytes == 0 {
        return None;
    }

    if value.len() <= *remaining_bytes {
        *remaining_bytes -= value.len();
        return Some((value.to_string(), false));
    }

    if *remaining_bytes <= TRUNCATED_PACK_MARKER.len() {
        return None;
    }

    let cutoff = *remaining_bytes - TRUNCATED_PACK_MARKER.len();
    let mut boundary = cutoff;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    if boundary == 0 {
        return None;
    }

    let mut truncated = value[..boundary].to_string();
    truncated.push_str(TRUNCATED_PACK_MARKER);
    *remaining_bytes = 0;
    Some((truncated, true))
}

fn append_history(root: &Path, assembly: &ContextAssembly) -> io::Result<()> {
    let mut history = load_history(root)?;
    history.runs.push(assembly.clone());
    if history.runs.len() > 10 {
        let overflow = history.runs.len() - 10;
        history.runs.drain(0..overflow);
    }

    let path = history_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(&history).map_err(io::Error::other)?)
}

fn load_history(root: &Path) -> io::Result<StoredRunHistory> {
    let path = history_path(root);
    if !path.exists() {
        return Ok(StoredRunHistory::default());
    }

    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

fn history_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("context_runs.json")
}

fn now_epoch_ms() -> io::Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_millis())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::ErrorKind;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        assemble_context, assemble_overview, assemble_task_capsule, context_run_history,
        retrieve_context, EngineInfo, InclusionReasonKind, OmissionReasonKind, RetrievalMode,
        TRUNCATED_PACK_MARKER,
    };

    #[test]
    fn banner_is_stable() {
        let info = EngineInfo::quotarelay();

        assert_eq!(info.banner(), "quotarelay bootstrap");
    }

    #[test]
    fn assembled_context_is_bounded_and_explained() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle one\nneedle two\nneedle three\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let assembly = assemble_context(&root, "needle", 2).expect("assembly should succeed");

        assert_eq!(assembly.snippets.len(), 2);
        assert_eq!(assembly.snippets[0].path, "alpha.txt");
        assert_eq!(assembly.snippets[0].reason.kind, InclusionReasonKind::QueryLineMatch);
        assert!(assembly.snippets[0].reason.detail.contains("query 'needle'"));
        assert_eq!(assembly.omissions[0].kind, OmissionReasonKind::ItemLimitReached);
        assert!(assembly.omissions[0].detail.contains("limit 2 was reached"));
    }

    #[test]
    fn context_run_history_is_bounded_and_keeps_omission_reasons() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle one\nneedle two\nneedle three\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        assemble_context(&root, "needle", 2).expect("first assembly should succeed");
        assemble_context(&root, "missing", 2).expect("second assembly should succeed");

        let history = context_run_history(&root, 1).expect("history should succeed");

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].query, "missing");
        assert!(history[0].snippets.is_empty());
        assert_eq!(history[0].omissions[0].kind, OmissionReasonKind::NoLineMatches);
        assert!(history[0].omissions[0].detail.contains("query 'missing'"));
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
        assert!(assembly.snippets[0].line.contains("struct Widget { id: usize }"));

        let body_query = assemble_context(&root, "body_only_term", 2).expect("body query should succeed");
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
        assert_eq!(overview.documents[0].reason.kind, InclusionReasonKind::OverviewDocument);
        assert_eq!(overview.omissions[0].kind, OmissionReasonKind::ItemLimitReached);
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
        assert_eq!(capsule.documents[0].reason.kind, InclusionReasonKind::TaskCapsuleMatch);
        assert!(capsule.documents[0].contents.contains("struct Widget { id: usize }"));
        assert!(!capsule.documents[0].contents.contains("body_only_term"));

        let fallback = assemble_task_capsule(&root, "raw_fallback_term", 2).expect("fallback capsule should succeed");
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
        assert!(exact.documents.is_empty());

        let overview = retrieve_context(&root, RetrievalMode::Overview, None, 2)
            .expect("overview should succeed");
        assert_eq!(overview.mode, RetrievalMode::Overview);
        assert_eq!(overview.documents.len(), 1);
        assert!(overview.snippets.is_empty());

        let missing = retrieve_context(&root, RetrievalMode::TaskCapsule, None, 2)
            .expect_err("task_capsule without query should fail");
        assert_eq!(missing.kind(), ErrorKind::InvalidInput);
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
        assert_eq!(assembly.omissions[0].kind, OmissionReasonKind::ByteBudgetReached);
    }

    #[test]
    fn task_capsule_reports_document_byte_budget_omission() {
        let root = temp_repo();
        let mut rust_file = String::new();
        for index in 0..80 {
            rust_file.push_str(&format!("fn widget_{index}() -> usize {{\n    {index}\n}}\n\n"));
        }
        fs::write(root.join("lib.rs"), rust_file).expect("rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let capsule = assemble_task_capsule(&root, "widget_0", 2).expect("task capsule should succeed");

        assert_eq!(capsule.documents.len(), 1);
        assert!(capsule.documents[0].contents.ends_with(TRUNCATED_PACK_MARKER));
        assert_eq!(capsule.omissions[0].kind, OmissionReasonKind::ByteBudgetReached);
    }

    #[test]
    fn task_capsule_prefers_symbol_matches_before_raw_text_mentions() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "Widget appears in planning notes\n")
            .expect("alpha file should write");
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let capsule = assemble_task_capsule(&root, "Widget", 2).expect("task capsule should succeed");

        assert_eq!(capsule.documents[0].path, "lib.rs");
    }

    fn temp_repo() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("quotarelay-context-{unique}"));
        fs::create_dir_all(&root).expect("temp repo should create");
        root
    }
}