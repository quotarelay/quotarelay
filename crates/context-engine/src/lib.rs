use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use repo_index::search_code;
use serde::{Deserialize, Serialize};

pub struct EngineInfo {
    name: &'static str,
    mode: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnippet {
    pub path: String,
    pub line_number: usize,
    pub line: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAssembly {
    pub query: String,
    pub generated_at_epoch_ms: u128,
    pub snippets: Vec<ContextSnippet>,
    pub omissions: Vec<String>,
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
    let snippets = hits
        .into_iter()
        .take(capped_limit)
        .map(|hit| ContextSnippet {
            path: hit.path,
            line_number: hit.line_number,
            line: hit.line,
            reason: format!("Included because this line matched query '{query}'."),
        })
        .collect::<Vec<_>>();
    let omissions = build_omissions(query, capped_limit, hits_exceeded_limit(root, query, capped_limit)?, snippets.is_empty());
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

fn build_omissions(query: &str, limit: usize, exceeded_limit: bool, no_matches: bool) -> Vec<String> {
    let mut omissions = Vec::new();
    if no_matches {
        omissions.push(format!("Omitted because no indexed lines matched query '{query}'."));
    }
    if exceeded_limit {
        omissions.push(format!("Omitted additional matching lines because limit {limit} was reached."));
    }
    omissions
}

fn hits_exceeded_limit(root: &Path, query: &str, limit: usize) -> io::Result<bool> {
    Ok(search_code(root, query, limit + 1)?.len() > limit)
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
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{assemble_context, context_run_history, EngineInfo};

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
        assert!(assembly.snippets[0].reason.contains("matched query 'needle'"));
        assert!(assembly.omissions.contains(&"Omitted additional matching lines because limit 2 was reached.".to_string()));
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
        assert!(history[0].omissions.contains(&"Omitted because no indexed lines matched query 'missing'.".to_string()));
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