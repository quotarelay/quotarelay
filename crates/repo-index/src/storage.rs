use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::*;

pub(crate) fn load_index(root: &Path) -> io::Result<StoredIndex> {
    let bytes = fs::read(index_path(root))?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

pub(crate) fn persist_index(root: &Path, stored: &StoredIndex) -> io::Result<PathBuf> {
    let index_path = index_path(root);
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &index_path,
        serde_json::to_vec_pretty(stored).map_err(io::Error::other)?,
    )?;
    Ok(index_path)
}

fn index_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("index.json")
}
