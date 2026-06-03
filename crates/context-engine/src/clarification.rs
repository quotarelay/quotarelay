use std::io;

use super::*;
use crate::retrieval::{cache_status, normalize_query};

pub(crate) fn clarification_context(
    mode: RetrievalMode,
    query: Option<&str>,
    clarification: ClarificationRequest,
) -> io::Result<RetrievedContext> {
    Ok(RetrievedContext {
        mode,
        query: query.map(normalize_query),
        generated_at_epoch_ms: now_epoch_ms()?,
        snippets: Vec::new(),
        memory_notes: Vec::new(),
        documents: Vec::new(),
        omissions: vec![OmissionReason {
            kind: OmissionReasonKind::NoDocumentMatches,
            detail: "Request needs clarification before bounded context can be assembled."
                .to_string(),
        }],
        cache_status: cache_status(
            CacheStatusKind::NotApplicable,
            "Clarification response did not read or write retrieval cache.",
        ),
        budget: ContextBudgetEstimate::default(),
        stale: ContextStaleStatus::default(),
        clarification: Some(clarification),
    })
}

pub(crate) fn clarification_for_request(
    mode: RetrievalMode,
    query: Option<&str>,
) -> Option<ClarificationRequest> {
    if !matches!(
        mode,
        RetrievalMode::ExactSearch | RetrievalMode::TaskCapsule
    ) {
        return None;
    }

    let normalized = query.map(normalize_query).unwrap_or_default();
    if normalized.is_empty() {
        return Some(clarification_request(
            "exact or task retrieval needs a concrete query.",
            vec![
                (
                    "target",
                    "Which file, symbol, or workflow should the context focus on?",
                ),
                (
                    "intent",
                    "Are you looking for implementation, tests, docs, or debugging context?",
                ),
            ],
        ));
    }

    if is_broad_query(&normalized) {
        return Some(clarification_request(
            "request is too broad for a bounded context pack.",
            vec![
                (
                    "target",
                    "Which component, path, or symbol should be inspected first?",
                ),
                (
                    "change",
                    "What specific change or question should the context support?",
                ),
                (
                    "proof",
                    "Which validation command or behavior should be prioritized?",
                ),
            ],
        ));
    }

    None
}

fn clarification_request(reason: &str, questions: Vec<(&str, &str)>) -> ClarificationRequest {
    ClarificationRequest {
        reason: reason.to_string(),
        questions: questions
            .into_iter()
            .take(3)
            .map(|(id, question)| ClarificationQuestion {
                id: id.to_string(),
                question: question.to_string(),
            })
            .collect(),
    }
}

fn is_broad_query(query: &str) -> bool {
    matches!(
        query,
        "fix" | "help" | "change" | "update" | "work" | "continue" | "refactor" | "improve"
    ) || query.len() < 3
}
