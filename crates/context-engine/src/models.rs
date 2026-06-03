use serde::{Deserialize, Serialize};

pub(crate) const MAX_SNIPPET_PACK_BYTES: usize = 160;
pub(crate) const MAX_DOCUMENT_PACK_BYTES: usize = 640;
pub(crate) const MAX_MEMORY_PACK_BYTES: usize = 320;
pub(crate) const TRUNCATED_PACK_MARKER: &str = "...";
pub(crate) const MAX_CONTEXT_ITEMS: usize = 5;
pub(crate) const MAX_HISTORY_RUNS: usize = 10;
pub(crate) const MAX_REGISTERED_REPOSITORIES: usize = 20;
pub(crate) const MAX_MEMORY_TRANSFER_NOTES: usize = 50;
pub(crate) const MAX_WORKSPACE_PROFILES: usize = 20;

pub struct EngineInfo {
    pub(crate) name: &'static str,
    pub(crate) mode: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
    MemoryNoteMatch,
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
    NoMemoryMatches,
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
    pub memory_notes: Vec<ContextMemoryNote>,
    pub omissions: Vec<OmissionReason>,
    #[serde(default)]
    pub budget: ContextBudgetEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMemoryNote {
    pub id: String,
    pub title: String,
    pub content: String,
    pub reason: InclusionReason,
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
    #[serde(default)]
    pub budget: ContextBudgetEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedContext {
    pub mode: RetrievalMode,
    pub query: Option<String>,
    pub generated_at_epoch_ms: u128,
    pub snippets: Vec<ContextSnippet>,
    pub memory_notes: Vec<ContextMemoryNote>,
    pub documents: Vec<ContextDocument>,
    pub omissions: Vec<OmissionReason>,
    #[serde(default)]
    pub budget: ContextBudgetEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ContextBudgetEstimate {
    pub included_bytes: usize,
    pub approximate_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryNote {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at_epoch_ms: u128,
    pub updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryWriteResult {
    pub note: MemoryNote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryUpdateResult {
    pub note: MemoryNote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryDeleteResult {
    pub note: Option<MemoryNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryExportPayload {
    pub notes: Vec<MemoryNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryExportResult {
    pub payload: MemoryExportPayload,
    pub omitted_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryImportResult {
    pub imported_count: usize,
    pub replaced_count: usize,
    pub omitted_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemorySearchResult {
    pub notes: Vec<MemoryNote>,
    pub omitted_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisteredRepository {
    pub id: String,
    pub name: String,
    pub root: String,
    pub registered_at_epoch_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryRegistrationResult {
    pub repository: RegisteredRepository,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryRemovalResult {
    pub repository: Option<RegisteredRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryMetadataUpdateResult {
    pub repository: Option<RegisteredRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositorySyncStatus {
    NotIndexed,
    Indexed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositorySyncState {
    pub status: RepositorySyncStatus,
    pub indexed_files: usize,
    pub indexed_at_epoch_ms: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LastContextRunSummary {
    pub query: String,
    pub generated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisteredRepositoryState {
    pub repository: RegisteredRepository,
    pub sync: RepositorySyncState,
    pub recent_context_run: Option<LastContextRunSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContextAssembly {
    pub repository: RegisteredRepository,
    pub assembly: ContextAssembly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRepositoryContextAssembly {
    pub query: String,
    pub generated_at_epoch_ms: u128,
    pub repositories: Vec<RepositoryContextAssembly>,
    pub omitted_repository_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceProfile {
    pub name: String,
    pub repo_roots: Vec<String>,
    pub default_mode: RetrievalMode,
    pub default_limit: usize,
    pub per_repo_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceProfileSaveResult {
    pub profile: WorkspaceProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievalLimits {
    pub max_context_items: usize,
    pub max_snippet_pack_bytes: usize,
    pub max_document_pack_bytes: usize,
    pub max_memory_pack_bytes: usize,
    pub max_history_runs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievalTruth {
    pub modes: Vec<RetrievalMode>,
    pub inclusion_reason_kinds: Vec<InclusionReasonKind>,
    pub omission_reason_kinds: Vec<OmissionReasonKind>,
    pub limits: RetrievalLimits,
    pub durable_memory_enabled: bool,
    pub budget_estimate_enabled: bool,
    pub budget_estimate_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalStateInspection {
    pub index: LocalStateArtifact,
    pub memory_notes: LocalStateArtifact,
    pub context_run_history: LocalStateArtifact,
    pub registered_repositories: LocalStateArtifact,
    pub exact_search_cache: LocalStateArtifact,
    pub retrieval_capsule_cache: LocalStateArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalStateArtifact {
    pub present: bool,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheInspection {
    pub exact_search_cache: LocalStateArtifact,
    pub retrieval_capsule_cache: LocalStateArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheClearResult {
    pub exact_search_cache_cleared: bool,
    pub retrieval_capsule_cache_cleared: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct StoredRunHistory {
    pub(crate) runs: Vec<ContextAssembly>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct StoredMemoryNotes {
    pub(crate) notes: Vec<MemoryNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct StoredExactMatchCache {
    pub(crate) entries: Vec<StoredExactMatchCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredExactMatchCacheEntry {
    pub(crate) query: String,
    pub(crate) assembly: ContextAssembly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct StoredCapsuleCache {
    pub(crate) entries: Vec<StoredCapsuleCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredCapsuleCacheEntry {
    pub(crate) mode: RetrievalMode,
    pub(crate) query: Option<String>,
    pub(crate) capsule: ContextCapsule,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct StoredRegisteredRepositories {
    pub(crate) repositories: Vec<RegisteredRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct StoredWorkspaceProfiles {
    pub(crate) profiles: Vec<WorkspaceProfile>,
}
