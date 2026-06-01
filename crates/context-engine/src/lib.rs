use std::io;
use std::path::Path;

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

pub fn assemble_context(root: &Path, query: &str, limit: usize) -> io::Result<Vec<ContextSnippet>> {
    let capped_limit = limit.clamp(1, 5);
    let hits = search_code(root, query, capped_limit)?;

    Ok(hits
        .into_iter()
        .map(|hit| ContextSnippet {
            path: hit.path,
            line_number: hit.line_number,
            line: hit.line,
            reason: format!("Included because this line matched query '{query}'."),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{assemble_context, EngineInfo};

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

        let snippets = assemble_context(&root, "needle", 2).expect("assembly should succeed");

        assert_eq!(snippets.len(), 2);
        assert_eq!(snippets[0].path, "alpha.txt");
        assert!(snippets[0].reason.contains("matched query 'needle'"));
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