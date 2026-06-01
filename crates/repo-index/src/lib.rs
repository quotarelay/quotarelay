use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexedFile {
    path: String,
    contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredIndex {
    indexed_at_epoch_ms: u128,
    files: Vec<IndexedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInventory {
    pub indexed_at_epoch_ms: u128,
    pub indexed_files: usize,
    pub sample_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub indexed_files: usize,
    pub index_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub line_number: usize,
    pub line: String,
}

pub fn sync_repo(root: &Path) -> io::Result<SyncResult> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;

    let stored = StoredIndex {
        indexed_at_epoch_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis(),
        files,
    };
    let index_path = index_path(root);
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&index_path, serde_json::to_vec_pretty(&stored).map_err(io::Error::other)?)?;

    Ok(SyncResult {
        indexed_files: stored.files.len(),
        index_path: index_path.display().to_string(),
    })
}

pub fn search_code(root: &Path, query: &str, limit: usize) -> io::Result<Vec<SearchHit>> {
    let index = load_index(root)?;
    let normalized_query = query.to_ascii_lowercase();
    let capped_limit = limit.clamp(1, 10);
    let mut hits = Vec::new();

    for file in index.files {
        for (offset, line) in file.contents.lines().enumerate() {
            if line.to_ascii_lowercase().contains(&normalized_query) {
                hits.push(SearchHit {
                    path: file.path.clone(),
                    line_number: offset + 1,
                    line: line.to_string(),
                });
                if hits.len() >= capped_limit {
                    return Ok(hits);
                }
            }
        }
    }

    Ok(hits)
}

pub fn repo_inventory(root: &Path) -> io::Result<RepoInventory> {
    let index = load_index(root)?;
    let sample_paths = index
        .files
        .iter()
        .take(5)
        .map(|file| file.path.clone())
        .collect();

    Ok(RepoInventory {
        indexed_at_epoch_ms: index.indexed_at_epoch_ms,
        indexed_files: index.files.len(),
        sample_paths,
    })
}

fn load_index(root: &Path) -> io::Result<StoredIndex> {
    let bytes = fs::read(index_path(root))?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

fn collect_files(root: &Path, current: &Path, files: &mut Vec<IndexedFile>) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            collect_files(root, &path, files)?;
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        let relative = path
            .strip_prefix(root)
            .map_err(io::Error::other)?
            .to_string_lossy()
            .replace('\\', "/");
        files.push(IndexedFile { path: relative, contents });
    }

    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | ".quotarelay" | "target" | "node_modules")
    )
}

fn index_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("index.json")
}

#[cfg(test)]
mod tests {
    use super::{repo_inventory, search_code, sync_repo};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sync_persists_and_search_returns_bounded_hits() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle one\nneedle two\n").expect("alpha file should write");
        fs::write(root.join("beta.txt"), "needle three\n").expect("beta file should write");

        let sync = sync_repo(&root).expect("sync should succeed");
        assert_eq!(sync.indexed_files, 2);
        assert!(root.join(".quotarelay").join("index.json").exists());

        let hits = search_code(&root, "needle", 2).expect("search should succeed");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].path, "alpha.txt");
        assert_eq!(hits[0].line_number, 1);

        let inventory = repo_inventory(&root).expect("inventory should succeed");
        assert_eq!(inventory.indexed_files, 2);
        assert_eq!(inventory.sample_paths, vec!["alpha.txt", "beta.txt"]);
        assert!(inventory.indexed_at_epoch_ms > 0);
    }

    fn temp_repo() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("quotarelay-index-{unique}"));
        fs::create_dir_all(&root).expect("temp repo should create");
        root
    }
}