use super::*;

pub(crate) fn budget_estimate(
    included_bytes: usize,
    raw_bytes_considered: usize,
) -> ContextBudgetEstimate {
    let raw_bytes_considered = raw_bytes_considered.max(included_bytes);
    ContextBudgetEstimate {
        raw_bytes_considered,
        included_bytes,
        approximate_tokens: included_bytes.div_ceil(4),
        estimated_reduction_ratio: reduction_ratio(included_bytes, raw_bytes_considered),
    }
}

pub(crate) fn included_snippet_bytes(snippets: &[ContextSnippet]) -> usize {
    snippets.iter().map(|snippet| snippet.line.len()).sum()
}

pub(crate) fn included_memory_bytes(memory_notes: &[ContextMemoryNote]) -> usize {
    memory_notes.iter().map(|note| note.content.len()).sum()
}

pub(crate) fn included_document_bytes(documents: &[ContextDocument]) -> usize {
    documents
        .iter()
        .map(|document| document.contents.len())
        .sum()
}

fn reduction_ratio(included_bytes: usize, raw_bytes_considered: usize) -> f64 {
    if raw_bytes_considered == 0 {
        return 0.0;
    }
    let reduction = 1.0 - (included_bytes as f64 / raw_bytes_considered as f64);
    reduction.clamp(0.0, 1.0)
}
