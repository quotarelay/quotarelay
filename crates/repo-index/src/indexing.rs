use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::rust_capsule::build_rust_capsule;

use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LocalIgnoreConfig {
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    prefixes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct IgnoreRules {
    paths: Vec<String>,
    prefixes: Vec<String>,
}

pub(crate) fn collect_indexed_files(
    root: &Path,
    previous_files: &HashMap<String, IndexedFile>,
) -> io::Result<Vec<IndexedFile>> {
    let ignore_rules = load_ignore_rules(root)?;
    let mut discovered_paths = Vec::new();
    collect_file_paths(root, root, &ignore_rules, &mut discovered_paths)?;
    let mut files = discovered_paths
        .into_iter()
        .filter_map(|path| build_indexed_file(root, &path, previous_files).transpose())
        .collect::<io::Result<Vec<_>>>()?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_file_paths(
    root: &Path,
    current: &Path,
    ignore_rules: &IgnoreRules,
    files: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let relative = relative_index_path(root, &path)?;

        if file_type.is_dir() {
            if should_skip_dir(&path) || should_ignore_path(&relative, ignore_rules) {
                continue;
            }
            collect_file_paths(root, &path, ignore_rules, files)?;
            continue;
        }

        if file_type.is_file() && !should_ignore_path(&relative, ignore_rules) {
            files.push(path);
        }
    }

    Ok(())
}

fn load_ignore_rules(root: &Path) -> io::Result<IgnoreRules> {
    let path = ignore_config_path(root);
    if !path.exists() {
        return Ok(IgnoreRules::default());
    }

    let bytes = fs::read(path)?;
    let config: LocalIgnoreConfig = serde_json::from_slice(&bytes).map_err(io::Error::other)?;

    Ok(IgnoreRules {
        paths: config
            .paths
            .into_iter()
            .filter_map(|entry| normalize_ignore_entry(&entry))
            .collect(),
        prefixes: config
            .prefixes
            .into_iter()
            .filter_map(|entry| normalize_ignore_entry(&entry))
            .collect(),
    })
}

fn normalize_ignore_entry(entry: &str) -> Option<String> {
    let normalized = entry
        .trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_string();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn should_ignore_path(relative: &str, rules: &IgnoreRules) -> bool {
    rules.paths.iter().any(|path| path == relative)
        || rules.prefixes.iter().any(|prefix| {
            relative == prefix
                || relative
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('/'))
        })
}

fn build_indexed_file(
    root: &Path,
    path: &Path,
    previous_files: &HashMap<String, IndexedFile>,
) -> io::Result<Option<IndexedFile>> {
    let relative = relative_index_path(root, path)?;
    let modified_at_epoch_ms = modified_at_epoch_ms(path)?;

    if let Some(existing) = previous_files.get(&relative) {
        if existing.modified_at_epoch_ms == modified_at_epoch_ms {
            return Ok(Some(existing.clone()));
        }
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return Ok(None);
    };
    let indexed_contents = build_indexed_contents(path, &contents);
    Ok(Some(IndexedFile {
        path: relative,
        contents: indexed_contents,
        modified_at_epoch_ms,
    }))
}

fn build_indexed_contents(path: &Path, contents: &str) -> String {
    let indexed = match path.extension().and_then(|extension| extension.to_str()) {
        Some("rs") => build_rust_capsule(contents).unwrap_or_else(|| contents.to_string()),
        _ => contents.to_string(),
    };
    bound_indexed_contents(indexed)
}

fn modified_at_epoch_ms(path: &Path) -> io::Result<u128> {
    fs::metadata(path)?
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_millis())
}

fn bound_indexed_contents(contents: String) -> String {
    if contents.len() <= MAX_INDEXED_CONTENT_BYTES {
        return contents;
    }

    let cutoff = MAX_INDEXED_CONTENT_BYTES.saturating_sub(TRUNCATED_INDEX_MARKER.len());
    let mut boundary = cutoff;
    while boundary > 0 && !contents.is_char_boundary(boundary) {
        boundary -= 1;
    }

    let mut bounded = contents[..boundary].to_string();
    bounded.push_str(TRUNCATED_INDEX_MARKER);
    bounded
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | ".quotarelay" | "target" | "node_modules")
    )
}

fn relative_index_path(root: &Path, path: &Path) -> io::Result<String> {
    Ok(path
        .strip_prefix(root)
        .map_err(io::Error::other)?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn ignore_config_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("ignore.json")
}

pub(crate) fn now_epoch_ms() -> io::Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_millis())
}
