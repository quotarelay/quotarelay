use serde::{Deserialize, Serialize};

pub(crate) const MAX_INDEXED_CONTENT_BYTES: usize = 8 * 1024;
pub(crate) const TRUNCATED_INDEX_MARKER: &str = "\n// ... truncated indexed contents ...";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IndexedFile {
    pub(crate) path: String,
    pub(crate) contents: String,
    #[serde(default)]
    pub(crate) modified_at_epoch_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredIndex {
    pub(crate) indexed_at_epoch_ms: u128,
    pub(crate) files: Vec<IndexedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInventory {
    pub indexed_at_epoch_ms: u128,
    pub indexed_files: usize,
    pub sample_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoMapDirectory {
    pub path: String,
    pub indexed_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoMapRustFile {
    pub path: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoMap {
    pub indexed_at_epoch_ms: u128,
    pub indexed_files: usize,
    pub directories: Vec<RepoMapDirectory>,
    pub rust_files: Vec<RepoMapRustFile>,
    pub omitted_directory_count: usize,
    pub omitted_rust_file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub indexed_files: usize,
    pub index_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexFreshness {
    pub is_stale: bool,
    pub changed_count: usize,
    pub missing_count: usize,
    pub new_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub line_number: usize,
    pub line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexedDocument {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangedDocumentStatus {
    Changed,
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangedDocument {
    pub path: String,
    pub contents: String,
    pub status: ChangedDocumentStatus,
}
