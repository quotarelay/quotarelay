use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{context_run_history, CacheStatusKind};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavingsReport {
    pub run_count: usize,
    pub raw_bytes_considered: usize,
    pub included_bytes: usize,
    pub approximate_tokens: usize,
    pub estimated_reduction_ratio: f64,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_not_applicable: usize,
    pub stale_runs: usize,
    pub omission_count: usize,
    pub note: String,
}

pub fn savings_report(root: &Path, limit: usize) -> io::Result<SavingsReport> {
    let runs = context_run_history(root, limit)?;
    let run_count = runs.len();
    let raw_bytes_considered = runs.iter().map(|run| run.budget.raw_bytes_considered).sum();
    let included_bytes = runs.iter().map(|run| run.budget.included_bytes).sum();
    let approximate_tokens = runs.iter().map(|run| run.budget.approximate_tokens).sum();
    let stale_runs = runs.iter().filter(|run| run.stale.is_stale).count();
    let omission_count = runs.iter().map(|run| run.omissions.len()).sum();

    let cache_hits = runs
        .iter()
        .filter(|run| run.cache_status.kind == CacheStatusKind::Hit)
        .count();
    let cache_misses = runs
        .iter()
        .filter(|run| run.cache_status.kind == CacheStatusKind::Miss)
        .count();
    let cache_not_applicable = runs
        .iter()
        .filter(|run| run.cache_status.kind == CacheStatusKind::NotApplicable)
        .count();

    Ok(SavingsReport {
        run_count,
        raw_bytes_considered,
        included_bytes,
        approximate_tokens,
        estimated_reduction_ratio: reduction_ratio(raw_bytes_considered, included_bytes),
        cache_hits,
        cache_misses,
        cache_not_applicable,
        stale_runs,
        omission_count,
        note: "Local aggregate from bounded context-run history only; not provider billing, exact tokenizer output, telemetry, or guaranteed savings.".to_string(),
    })
}

fn reduction_ratio(raw_bytes: usize, included_bytes: usize) -> f64 {
    if raw_bytes == 0 || included_bytes >= raw_bytes {
        return 0.0;
    }
    (raw_bytes - included_bytes) as f64 / raw_bytes as f64
}
