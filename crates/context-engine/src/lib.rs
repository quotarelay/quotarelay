use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use repo_index::{indexed_documents, matching_documents, repo_inventory, search_code};
use serde::{Deserialize, Serialize};

const MAX_SNIPPET_PACK_BYTES: usize = 160;
const MAX_DOCUMENT_PACK_BYTES: usize = 640;
const MAX_MEMORY_PACK_BYTES: usize = 320;
const TRUNCATED_PACK_MARKER: &str = "...";
const MAX_CONTEXT_ITEMS: usize = 5;
const MAX_HISTORY_RUNS: usize = 10;
const MAX_REGISTERED_REPOSITORIES: usize = 20;
const MAX_MEMORY_TRANSFER_NOTES: usize = 50;
const MAX_WORKSPACE_PROFILES: usize = 20;

pub struct EngineInfo {
    name: &'static str,
    mode: &'static str,
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
struct StoredRunHistory {
    runs: Vec<ContextAssembly>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoredMemoryNotes {
    notes: Vec<MemoryNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoredExactMatchCache {
    entries: Vec<StoredExactMatchCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredExactMatchCacheEntry {
    query: String,
    assembly: ContextAssembly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoredCapsuleCache {
    entries: Vec<StoredCapsuleCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCapsuleCacheEntry {
    mode: RetrievalMode,
    query: Option<String>,
    capsule: ContextCapsule,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoredRegisteredRepositories {
    repositories: Vec<RegisteredRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoredWorkspaceProfiles {
    profiles: Vec<WorkspaceProfile>,
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

fn normalize_query(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub fn assemble_context(root: &Path, query: &str, limit: usize) -> io::Result<ContextAssembly> {
    let query = normalize_query(query);
    let capped_limit = limit.clamp(1, 5);
    if let Some(assembly) = read_exact_match_cache(root, &query)? {
        append_history(root, &assembly)?;
        return Ok(assembly);
    }

    let hits = search_code(root, &query, capped_limit + 1)?;
    let (snippets, omissions) = pack_snippets(&query, hits, capped_limit);
    let (memory_notes, memory_omissions) = pack_memory_notes(&query, find_memory_notes(root, &query)?, capped_limit);
    let mut omissions = omissions;
    omissions.extend(memory_omissions);
    let assembly = ContextAssembly {
        query: query.clone(),
        generated_at_epoch_ms: now_epoch_ms()?,
        snippets,
        memory_notes,
        omissions,
    };

    persist_exact_match_cache(root, &query, &assembly)?;
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

pub fn context_run_detail(root: &Path, generated_at_epoch_ms: u128) -> io::Result<Option<ContextAssembly>> {
    let history = load_history(root)?;

    Ok(history
        .runs
        .into_iter()
        .rev()
        .take(MAX_HISTORY_RUNS)
        .find(|run| run.generated_at_epoch_ms == generated_at_epoch_ms))
}

pub fn retrieve_context(
    root: &Path,
    mode: RetrievalMode,
    query: Option<&str>,
    limit: usize,
) -> io::Result<RetrievedContext> {
    match mode {
        RetrievalMode::ExactSearch => {
            let query = query.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "exact_search requires a query")
            })?;
            let assembly = assemble_context(root, query, limit)?;
            Ok(RetrievedContext {
                mode,
                query: Some(assembly.query.clone()),
                generated_at_epoch_ms: assembly.generated_at_epoch_ms,
                snippets: assembly.snippets,
                memory_notes: assembly.memory_notes,
                documents: Vec::new(),
                omissions: assembly.omissions,
            })
        }
        RetrievalMode::Overview => {
            let capsule = assemble_overview(root, limit)?;
            Ok(RetrievedContext {
                mode,
                query: None,
                generated_at_epoch_ms: capsule.generated_at_epoch_ms,
                snippets: Vec::new(),
                memory_notes: Vec::new(),
                documents: capsule.documents,
                omissions: capsule.omissions,
            })
        }
        RetrievalMode::TaskCapsule => {
            let query = query.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "task_capsule requires a query")
            })?;
            let normalized_query = normalize_query(query);
            let capsule = assemble_task_capsule(root, &normalized_query, limit)?;
            Ok(RetrievedContext {
                mode,
                query: Some(normalized_query),
                generated_at_epoch_ms: capsule.generated_at_epoch_ms,
                snippets: Vec::new(),
                memory_notes: Vec::new(),
                documents: capsule.documents,
                omissions: capsule.omissions,
            })
        }
    }
}

pub fn retrieval_truth() -> RetrievalTruth {
    RetrievalTruth {
        modes: vec![
            RetrievalMode::ExactSearch,
            RetrievalMode::Overview,
            RetrievalMode::TaskCapsule,
        ],
        inclusion_reason_kinds: vec![
            InclusionReasonKind::QueryLineMatch,
            InclusionReasonKind::OverviewDocument,
            InclusionReasonKind::TaskCapsuleMatch,
            InclusionReasonKind::MemoryNoteMatch,
        ],
        omission_reason_kinds: vec![
            OmissionReasonKind::NoLineMatches,
            OmissionReasonKind::NoDocumentMatches,
            OmissionReasonKind::NoMemoryMatches,
            OmissionReasonKind::ItemLimitReached,
            OmissionReasonKind::ByteBudgetReached,
        ],
        limits: RetrievalLimits {
            max_context_items: MAX_CONTEXT_ITEMS,
            max_snippet_pack_bytes: MAX_SNIPPET_PACK_BYTES,
            max_document_pack_bytes: MAX_DOCUMENT_PACK_BYTES,
            max_memory_pack_bytes: MAX_MEMORY_PACK_BYTES,
            max_history_runs: MAX_HISTORY_RUNS,
        },
        durable_memory_enabled: true,
    }
}

pub fn register_repository(state_root: &Path, repo_root: &Path) -> io::Result<RepositoryRegistrationResult> {
    let normalized_root = normalized_repo_root(repo_root)?;
    let mut stored = load_registered_repositories(state_root)?;

    if let Some(existing) = stored
        .repositories
        .iter()
        .find(|repository| repository.root == normalized_root)
        .cloned()
    {
        return Ok(RepositoryRegistrationResult { repository: existing });
    }

    let repository = RegisteredRepository {
        id: normalized_root.clone(),
        name: repo_name(repo_root),
        root: normalized_root,
        registered_at_epoch_ms: now_epoch_ms()?,
    };

    stored.repositories.push(repository.clone());
    stored.repositories.sort_by(|left, right| left.root.cmp(&right.root));
    persist_registered_repositories(state_root, &stored)?;

    Ok(RepositoryRegistrationResult { repository })
}

pub fn list_registered_repositories(
    state_root: &Path,
    limit: usize,
) -> io::Result<Vec<RegisteredRepository>> {
    let capped_limit = limit.clamp(1, MAX_REGISTERED_REPOSITORIES);
    let stored = load_registered_repositories(state_root)?;

    Ok(stored.repositories.into_iter().take(capped_limit).collect())
}

pub fn remove_registered_repository(
    state_root: &Path,
    repo_root: &Path,
) -> io::Result<RepositoryRemovalResult> {
    let normalized_root = normalized_repo_root(repo_root)?;
    let mut stored = load_registered_repositories(state_root)?;
    let removed = stored
        .repositories
        .iter()
        .position(|repository| repository.root == normalized_root)
        .map(|index| stored.repositories.remove(index));

    persist_registered_repositories(state_root, &stored)?;

    Ok(RepositoryRemovalResult { repository: removed })
}

pub fn update_registered_repository_metadata(
    state_root: &Path,
    repo_root: &Path,
    name: Option<&str>,
) -> io::Result<RepositoryMetadataUpdateResult> {
    let normalized_root = normalized_repo_root(repo_root)?;
    let mut stored = load_registered_repositories(state_root)?;
    let updated = stored
        .repositories
        .iter_mut()
        .find(|repository| repository.root == normalized_root)
        .map(|repository| {
            if let Some(name) = name {
                repository.name = name.to_string();
            }
            repository.clone()
        });

    if updated.is_some() {
        persist_registered_repositories(state_root, &stored)?;
    }

    Ok(RepositoryMetadataUpdateResult { repository: updated })
}

pub fn registered_repository_state(
    state_root: &Path,
    limit: usize,
) -> io::Result<Vec<RegisteredRepositoryState>> {
    list_registered_repositories(state_root, limit)?
        .into_iter()
        .map(registered_repository_state_for)
        .collect()
}

pub fn registered_repository_detail(
    state_root: &Path,
    repo_root: &Path,
) -> io::Result<Option<RegisteredRepositoryState>> {
    let normalized_root = normalized_repo_root(repo_root)?;
    let stored = load_registered_repositories(state_root)?;
    stored
        .repositories
        .into_iter()
        .find(|repository| repository.root == normalized_root)
        .map(registered_repository_state_for)
        .transpose()
}

pub fn assemble_context_for_registered_repositories(
    state_root: &Path,
    repo_roots: &[PathBuf],
    query: &str,
    per_repo_limit: usize,
) -> io::Result<MultiRepositoryContextAssembly> {
    if repo_roots.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "multi-repo assembly requires at least one repo_root",
        ));
    }

    let capped_repo_count = repo_roots.len().min(MAX_CONTEXT_ITEMS);
    let omitted_repository_count = repo_roots.len().saturating_sub(capped_repo_count);
    let capped_limit = per_repo_limit.clamp(1, MAX_CONTEXT_ITEMS);
    let stored = load_registered_repositories(state_root)?;
    let mut repositories = Vec::new();

    for repo_root in repo_roots.iter().take(capped_repo_count) {
        let normalized_root = normalized_repo_root(repo_root)?;
        let repository = stored
            .repositories
            .iter()
            .find(|repository| repository.root == normalized_root)
            .cloned()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("repository {normalized_root} is not registered"),
                )
            })?;
        let assembly = assemble_context(Path::new(&repository.root), query, capped_limit)?;
        repositories.push(RepositoryContextAssembly { repository, assembly });
    }

    Ok(MultiRepositoryContextAssembly {
        query: normalize_query(query),
        generated_at_epoch_ms: now_epoch_ms()?,
        repositories,
        omitted_repository_count,
    })
}

pub fn save_workspace_profile(
    state_root: &Path,
    name: &str,
    repo_roots: &[PathBuf],
    default_mode: RetrievalMode,
    default_limit: usize,
    per_repo_limit: usize,
) -> io::Result<WorkspaceProfileSaveResult> {
    let name = name.trim();
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "workspace profile name cannot be empty",
        ));
    }
    if repo_roots.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "workspace profile requires at least one repo_root",
        ));
    }

    let normalized_roots = repo_roots
        .iter()
        .take(MAX_CONTEXT_ITEMS)
        .map(|root| normalized_repo_root(root))
        .collect::<io::Result<Vec<_>>>()?;
    let profile = WorkspaceProfile {
        name: name.to_string(),
        repo_roots: normalized_roots,
        default_mode,
        default_limit: default_limit.clamp(1, MAX_CONTEXT_ITEMS),
        per_repo_limit: per_repo_limit.clamp(1, MAX_CONTEXT_ITEMS),
    };
    let mut stored = load_workspace_profiles(state_root)?;

    if let Some(existing) = stored.profiles.iter_mut().find(|existing| existing.name == profile.name) {
        *existing = profile.clone();
    } else {
        stored.profiles.push(profile.clone());
        stored.profiles.sort_by(|left, right| left.name.cmp(&right.name));
        if stored.profiles.len() > MAX_WORKSPACE_PROFILES {
            stored.profiles.truncate(MAX_WORKSPACE_PROFILES);
        }
    }

    persist_workspace_profiles(state_root, &stored)?;

    Ok(WorkspaceProfileSaveResult { profile })
}

pub fn list_workspace_profiles(state_root: &Path, limit: usize) -> io::Result<Vec<WorkspaceProfile>> {
    let capped_limit = limit.clamp(1, MAX_WORKSPACE_PROFILES);
    let stored = load_workspace_profiles(state_root)?;

    Ok(stored.profiles.into_iter().take(capped_limit).collect())
}

fn registered_repository_state_for(
    repository: RegisteredRepository,
) -> io::Result<RegisteredRepositoryState> {
    let repo_root = PathBuf::from(&repository.root);
    let sync = match repo_inventory(&repo_root) {
        Ok(inventory) => RepositorySyncState {
            status: RepositorySyncStatus::Indexed,
            indexed_files: inventory.indexed_files,
            indexed_at_epoch_ms: Some(inventory.indexed_at_epoch_ms),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => RepositorySyncState {
            status: RepositorySyncStatus::NotIndexed,
            indexed_files: 0,
            indexed_at_epoch_ms: None,
        },
        Err(error) => return Err(error),
    };
    let recent_context_run = context_run_history(&repo_root, 1)?
        .into_iter()
        .next()
        .map(|run| LastContextRunSummary {
            query: run.query,
            generated_at_epoch_ms: run.generated_at_epoch_ms,
        });

    Ok(RegisteredRepositoryState {
        repository,
        sync,
        recent_context_run,
    })
}

pub fn invalidate_exact_match_cache(root: &Path) -> io::Result<()> {
    clear_exact_match_cache(root)?;
    clear_capsule_cache(root)
}

pub fn assemble_overview(root: &Path, limit: usize) -> io::Result<ContextCapsule> {
    if let Some(capsule) = read_capsule_cache(root, RetrievalMode::Overview, None)? {
        return Ok(capsule);
    }

    let capped_limit = limit.clamp(1, MAX_CONTEXT_ITEMS);
    let documents = indexed_documents(root, capped_limit + 1)?;
    let (documents, omissions) = pack_documents(
        documents,
        capped_limit,
        InclusionReasonKind::OverviewDocument,
        "Indexed document supports repository overview.",
        "repository overview",
    );
    let capsule = ContextCapsule {
        generated_at_epoch_ms: now_epoch_ms()?,
        documents,
        omissions,
    };
    persist_capsule_cache(root, RetrievalMode::Overview, None, &capsule)?;
    Ok(capsule)
}

pub fn assemble_task_capsule(root: &Path, query: &str, limit: usize) -> io::Result<ContextCapsule> {
    let query = normalize_query(query);
    if let Some(capsule) = read_capsule_cache(root, RetrievalMode::TaskCapsule, Some(&query))? {
        return Ok(capsule);
    }

    let capped_limit = limit.clamp(1, MAX_CONTEXT_ITEMS);
    let documents = matching_documents(root, &query, capped_limit + 1)?;
    let reason_detail = format!("Indexed document contents matched query '{query}'.");
    let scope = format!("query '{query}'");
    let (documents, omissions) = pack_documents(
        documents,
        capped_limit,
        InclusionReasonKind::TaskCapsuleMatch,
        &reason_detail,
        &scope,
    );
    let capsule = ContextCapsule {
        generated_at_epoch_ms: now_epoch_ms()?,
        documents,
        omissions,
    };
    persist_capsule_cache(root, RetrievalMode::TaskCapsule, Some(&query), &capsule)?;
    Ok(capsule)
}

pub fn memory_write(root: &Path, title: &str, content: &str, tags: &[String]) -> io::Result<MemoryWriteResult> {
    let now = now_epoch_ms()?;
    let mut stored = load_memory_notes(root)?;
    let note = MemoryNote {
        id: format!("mem-{}", unique_epoch_nanos()?),
        title: title.to_string(),
        content: content.to_string(),
        tags: tags.to_vec(),
        created_at_epoch_ms: now,
        updated_at_epoch_ms: now,
    };
    stored.notes.push(note.clone());
    persist_memory_notes(root, &stored)?;
    Ok(MemoryWriteResult { note })
}

pub fn memory_update(
    root: &Path,
    id: &str,
    title: Option<&str>,
    content: Option<&str>,
    tags: Option<&[String]>,
) -> io::Result<MemoryUpdateResult> {
    let mut stored = load_memory_notes(root)?;
    let note = stored
        .notes
        .iter_mut()
        .find(|note| note.id == id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("memory note {id} not found")))?;

    let mut changed = false;

    if let Some(title) = title {
        note.title = title.to_string();
        changed = true;
    }
    if let Some(content) = content {
        note.content = content.to_string();
        changed = true;
    }
    if let Some(tags) = tags {
        note.tags = tags.to_vec();
        changed = true;
    }

    if !changed {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "memory update requires at least one field to change",
        ));
    }

    note.updated_at_epoch_ms = now_epoch_ms()?;
    let updated = note.clone();
    persist_memory_notes(root, &stored)?;

    Ok(MemoryUpdateResult { note: updated })
}

pub fn memory_delete(root: &Path, id: &str) -> io::Result<MemoryDeleteResult> {
    let mut stored = load_memory_notes(root)?;
    let removed = stored
        .notes
        .iter()
        .position(|note| note.id == id)
        .map(|index| stored.notes.remove(index));

    if removed.is_some() {
        persist_memory_notes(root, &stored)?;
    }

    Ok(MemoryDeleteResult { note: removed })
}

pub fn memory_export(root: &Path, limit: usize) -> io::Result<MemoryExportResult> {
    let stored = load_memory_notes(root)?;
    let capped_limit = limit.clamp(1, MAX_MEMORY_TRANSFER_NOTES);
    let total_count = stored.notes.len();
    let notes = stored.notes.into_iter().take(capped_limit).collect::<Vec<_>>();

    Ok(MemoryExportResult {
        payload: MemoryExportPayload { notes },
        omitted_count: total_count.saturating_sub(capped_limit),
    })
}

pub fn memory_import(root: &Path, payload: MemoryExportPayload) -> io::Result<MemoryImportResult> {
    let mut stored = load_memory_notes(root)?;
    let incoming_count = payload.notes.len();
    let mut imported_count = 0;
    let mut replaced_count = 0;
    let mut omitted_count = incoming_count.saturating_sub(MAX_MEMORY_TRANSFER_NOTES);

    for note in payload.notes.into_iter().take(MAX_MEMORY_TRANSFER_NOTES) {
        if let Some(existing) = stored.notes.iter_mut().find(|existing| existing.id == note.id) {
            *existing = note;
            replaced_count += 1;
        } else {
            stored.notes.push(note);
            imported_count += 1;
        }
    }

    if stored.notes.len() > MAX_MEMORY_TRANSFER_NOTES {
        omitted_count += stored.notes.len() - MAX_MEMORY_TRANSFER_NOTES;
        stored.notes.truncate(MAX_MEMORY_TRANSFER_NOTES);
    }

    persist_memory_notes(root, &stored)?;

    Ok(MemoryImportResult {
        imported_count,
        replaced_count,
        omitted_count,
    })
}

pub fn memory_read(root: &Path, id: &str) -> io::Result<Option<MemoryNote>> {
    let stored = load_memory_notes(root)?;
    Ok(stored.notes.into_iter().find(|note| note.id == id))
}

pub fn memory_search(root: &Path, query: &str, limit: usize) -> io::Result<MemorySearchResult> {
    let capped_limit = limit.clamp(1, MAX_CONTEXT_ITEMS);
    let mut matches = find_memory_notes(root, query)?;
    let omitted_count = matches.len().saturating_sub(capped_limit);
    matches.truncate(capped_limit);

    Ok(MemorySearchResult {
        notes: matches,
        omitted_count,
    })
}

pub fn inspect_local_state(root: &Path) -> io::Result<LocalStateInspection> {
    let index = match repo_inventory(root) {
        Ok(inventory) => LocalStateArtifact {
            present: true,
            item_count: inventory.indexed_files,
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => LocalStateArtifact {
            present: false,
            item_count: 0,
        },
        Err(error) => return Err(error),
    };
    let memory_notes = load_memory_notes(root)?;
    let history = load_history(root)?;
    let registered_repositories = load_registered_repositories(root)?;
    let exact_cache = load_exact_match_cache(root)?;
    let capsule_cache = load_capsule_cache(root)?;

    Ok(LocalStateInspection {
        index,
        memory_notes: LocalStateArtifact {
            present: memory_notes_path(root).exists(),
            item_count: memory_notes.notes.len(),
        },
        context_run_history: LocalStateArtifact {
            present: history_path(root).exists(),
            item_count: history.runs.len(),
        },
        registered_repositories: LocalStateArtifact {
            present: registered_repositories_path(root).exists(),
            item_count: registered_repositories.repositories.len(),
        },
        exact_search_cache: LocalStateArtifact {
            present: exact_match_cache_path(root).exists(),
            item_count: exact_cache.entries.len(),
        },
        retrieval_capsule_cache: LocalStateArtifact {
            present: capsule_cache_path(root).exists(),
            item_count: capsule_cache.entries.len(),
        },
    })
}

pub fn inspect_retrieval_caches(root: &Path) -> io::Result<CacheInspection> {
    let exact_cache = load_exact_match_cache(root)?;
    let capsule_cache = load_capsule_cache(root)?;

    Ok(CacheInspection {
        exact_search_cache: LocalStateArtifact {
            present: exact_match_cache_path(root).exists(),
            item_count: exact_cache.entries.len(),
        },
        retrieval_capsule_cache: LocalStateArtifact {
            present: capsule_cache_path(root).exists(),
            item_count: capsule_cache.entries.len(),
        },
    })
}

pub fn clear_retrieval_caches(root: &Path) -> io::Result<CacheClearResult> {
    let exact_search_cache_cleared = exact_match_cache_path(root).exists();
    let retrieval_capsule_cache_cleared = capsule_cache_path(root).exists();

    clear_exact_match_cache(root)?;
    clear_capsule_cache(root)?;

    Ok(CacheClearResult {
        exact_search_cache_cleared,
        retrieval_capsule_cache_cleared,
    })
}

fn find_memory_notes(root: &Path, query: &str) -> io::Result<Vec<MemoryNote>> {
    let normalized_query = query.to_ascii_lowercase();
    let stored = load_memory_notes(root)?;
    let mut matches = stored
        .notes
        .into_iter()
        .filter(|note| {
            note.title.to_ascii_lowercase().contains(&normalized_query)
                || note.content.to_ascii_lowercase().contains(&normalized_query)
                || note
                    .tags
                    .iter()
                    .any(|tag| tag.to_ascii_lowercase().contains(&normalized_query))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| right.updated_at_epoch_ms.cmp(&left.updated_at_epoch_ms));
    Ok(matches)
}

fn pack_snippets(query: &str, hits: Vec<repo_index::SearchHit>, limit: usize) -> (Vec<ContextSnippet>, Vec<OmissionReason>) {
    let total_hits = hits.len();
    let mut snippets = Vec::new();
    let mut omissions = Vec::new();
    let mut remaining_bytes = MAX_SNIPPET_PACK_BYTES;
    let mut byte_budget_hit = false;

    for hit in hits.into_iter().take(limit) {
        let Some((line, truncated)) = fit_within_budget(&hit.line, &mut remaining_bytes) else {
            byte_budget_hit = true;
            break;
        };
        snippets.push(ContextSnippet {
            path: hit.path,
            line_number: hit.line_number,
            line,
            reason: InclusionReason {
                kind: InclusionReasonKind::QueryLineMatch,
                detail: format!("Indexed line matched query '{query}'."),
            },
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if snippets.is_empty() && total_hits == 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoLineMatches,
            detail: format!("No indexed lines matched query '{query}'."),
        });
    }
    if total_hits > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!("Additional matching lines omitted because item limit {limit} was reached."),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!("Packed matching lines reached byte budget {MAX_SNIPPET_PACK_BYTES}."),
        });
    }

    (snippets, omissions)
}

fn pack_documents(
    documents: Vec<repo_index::IndexedDocument>,
    limit: usize,
    reason_kind: InclusionReasonKind,
    reason_detail: &str,
    scope: &str,
) -> (Vec<ContextDocument>, Vec<OmissionReason>) {
    let total_documents = documents.len();
    let mut packed = Vec::new();
    let mut omissions = Vec::new();
    let mut remaining_bytes = MAX_DOCUMENT_PACK_BYTES;
    let mut byte_budget_hit = false;

    for document in documents.into_iter().take(limit) {
        let Some((contents, truncated)) = fit_within_budget(&document.contents, &mut remaining_bytes) else {
            byte_budget_hit = true;
            break;
        };
        packed.push(ContextDocument {
            path: document.path,
            contents,
            reason: InclusionReason {
                kind: reason_kind.clone(),
                detail: reason_detail.to_string(),
            },
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if packed.is_empty() && total_documents == 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoDocumentMatches,
            detail: format!("No indexed documents matched {scope}."),
        });
    }
    if total_documents > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!("Additional indexed documents omitted because item limit {limit} was reached."),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!("Packed indexed documents reached byte budget {MAX_DOCUMENT_PACK_BYTES}."),
        });
    }

    (packed, omissions)
}

fn fit_within_budget(value: &str, remaining_bytes: &mut usize) -> Option<(String, bool)> {
    if *remaining_bytes == 0 {
        return None;
    }

    if value.len() <= *remaining_bytes {
        *remaining_bytes -= value.len();
        return Some((value.to_string(), false));
    }

    if *remaining_bytes <= TRUNCATED_PACK_MARKER.len() {
        return None;
    }

    let cutoff = *remaining_bytes - TRUNCATED_PACK_MARKER.len();
    let mut boundary = cutoff;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    if boundary == 0 {
        return None;
    }

    let mut truncated = value[..boundary].to_string();
    truncated.push_str(TRUNCATED_PACK_MARKER);
    *remaining_bytes = 0;
    Some((truncated, true))
}

fn pack_memory_notes(query: &str, notes: Vec<MemoryNote>, limit: usize) -> (Vec<ContextMemoryNote>, Vec<OmissionReason>) {
    let total_notes = notes.len();
    let mut packed = Vec::new();
    let mut omissions = Vec::new();
    let mut remaining_bytes = MAX_MEMORY_PACK_BYTES;
    let mut byte_budget_hit = false;

    for note in notes.into_iter().take(limit) {
        let Some((content, truncated)) = fit_within_budget(&note.content, &mut remaining_bytes) else {
            byte_budget_hit = true;
            break;
        };
        packed.push(ContextMemoryNote {
            id: note.id,
            title: note.title,
            content,
            reason: InclusionReason {
                kind: InclusionReasonKind::MemoryNoteMatch,
                detail: format!("Durable memory matched query '{query}'."),
            },
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if packed.is_empty() && total_notes == 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoMemoryMatches,
            detail: format!("No durable memory matched query '{query}'."),
        });
    }
    if total_notes > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!("Additional memory notes omitted because item limit {limit} was reached."),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!("Packed memory notes reached byte budget {MAX_MEMORY_PACK_BYTES}."),
        });
    }

    (packed, omissions)
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

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

fn history_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("context_runs.json")
}

fn exact_match_cache_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("exact_match_cache.json")
}

fn capsule_cache_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("retrieval_capsules.json")
}

fn memory_notes_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("memory_notes.json")
}

fn registered_repositories_path(state_root: &Path) -> PathBuf {
    state_root.join(".quotarelay").join("registered_repositories.json")
}

fn workspace_profiles_path(state_root: &Path) -> PathBuf {
    state_root.join(".quotarelay").join("workspace_profiles.json")
}

fn parse_state_json<T>(path: &Path, bytes: &[u8]) -> io::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_slice(bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "corrupt local state file {}; repair or remove the file to recover: {error}",
                path.display()
            ),
        )
    })
}

fn load_memory_notes(root: &Path) -> io::Result<StoredMemoryNotes> {
    let path = memory_notes_path(root);
    if !path.exists() {
        return Ok(StoredMemoryNotes::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

fn persist_memory_notes(root: &Path, notes: &StoredMemoryNotes) -> io::Result<()> {
    let path = memory_notes_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(notes).map_err(io::Error::other)?)
}

fn load_workspace_profiles(state_root: &Path) -> io::Result<StoredWorkspaceProfiles> {
    let path = workspace_profiles_path(state_root);
    if !path.exists() {
        return Ok(StoredWorkspaceProfiles::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

fn persist_workspace_profiles(
    state_root: &Path,
    profiles: &StoredWorkspaceProfiles,
) -> io::Result<()> {
    let path = workspace_profiles_path(state_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(profiles).map_err(io::Error::other)?)
}

fn load_exact_match_cache(root: &Path) -> io::Result<StoredExactMatchCache> {
    let path = exact_match_cache_path(root);
    if !path.exists() {
        return Ok(StoredExactMatchCache::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

fn load_capsule_cache(root: &Path) -> io::Result<StoredCapsuleCache> {
    let path = capsule_cache_path(root);
    if !path.exists() {
        return Ok(StoredCapsuleCache::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

fn read_exact_match_cache(root: &Path, query: &str) -> io::Result<Option<ContextAssembly>> {
    let query = normalize_query(query);
    let stored = load_exact_match_cache(root)?;
    Ok(stored
        .entries
        .into_iter()
        .find(|entry| entry.query == query)
        .map(|entry| entry.assembly))
}

fn persist_exact_match_cache(root: &Path, query: &str, assembly: &ContextAssembly) -> io::Result<()> {
    let query = normalize_query(query);
    let mut stored = load_exact_match_cache(root)?;
    if let Some(entry) = stored.entries.iter_mut().find(|entry| entry.query == query) {
        entry.assembly = assembly.clone();
    } else {
        stored.entries.push(StoredExactMatchCacheEntry {
            query: query.to_string(),
            assembly: assembly.clone(),
        });
        stored.entries.sort_by(|left, right| left.query.cmp(&right.query));
    }

    let path = exact_match_cache_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(&stored).map_err(io::Error::other)?)
}

fn read_capsule_cache(root: &Path, mode: RetrievalMode, query: Option<&str>) -> io::Result<Option<ContextCapsule>> {
    let query = query.map(normalize_query);
    let stored = load_capsule_cache(root)?;
    Ok(stored
        .entries
        .into_iter()
        .find(|entry| entry.mode == mode && entry.query.as_deref() == query.as_deref())
        .map(|entry| entry.capsule))
}

fn persist_capsule_cache(
    root: &Path,
    mode: RetrievalMode,
    query: Option<&str>,
    capsule: &ContextCapsule,
) -> io::Result<()> {
    let query = query.map(normalize_query);
    let mut stored = load_capsule_cache(root)?;
    if let Some(entry) = stored
        .entries
        .iter_mut()
        .find(|entry| entry.mode == mode && entry.query.as_deref() == query.as_deref())
    {
        entry.capsule = capsule.clone();
    } else {
        stored.entries.push(StoredCapsuleCacheEntry {
            mode,
            query: query.clone(),
            capsule: capsule.clone(),
        });
        stored.entries.sort_by(|left, right| {
            left.mode
                .cmp(&right.mode)
                .then_with(|| left.query.cmp(&right.query))
        });
    }

    let path = capsule_cache_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(&stored).map_err(io::Error::other)?)
}

fn clear_exact_match_cache(root: &Path) -> io::Result<()> {
    let path = exact_match_cache_path(root);
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
}

fn clear_capsule_cache(root: &Path) -> io::Result<()> {
    let path = capsule_cache_path(root);
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
}

fn load_registered_repositories(state_root: &Path) -> io::Result<StoredRegisteredRepositories> {
    let path = registered_repositories_path(state_root);
    if !path.exists() {
        return Ok(StoredRegisteredRepositories::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

fn persist_registered_repositories(
    state_root: &Path,
    repositories: &StoredRegisteredRepositories,
) -> io::Result<()> {
    let path = registered_repositories_path(state_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(repositories).map_err(io::Error::other)?)
}

fn normalized_repo_root(repo_root: &Path) -> io::Result<String> {
    let canonical = fs::canonicalize(repo_root)?;
    Ok(canonical.to_string_lossy().into_owned())
}

fn repo_name(repo_root: &Path) -> String {
    repo_root
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| repo_root.to_string_lossy().into_owned())
}

fn now_epoch_ms() -> io::Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_millis())
}

fn unique_epoch_nanos() -> io::Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_nanos())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::ErrorKind;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        assemble_context, assemble_overview, assemble_task_capsule, context_run_detail,
        context_run_history,
        inspect_local_state, invalidate_exact_match_cache,
        list_registered_repositories, memory_delete, memory_export, memory_import, memory_read,
        memory_search, memory_update, memory_write, MemoryExportPayload,
        register_repository, registered_repository_detail, registered_repository_state, remove_registered_repository,
        assemble_context_for_registered_repositories,
        clear_retrieval_caches, inspect_retrieval_caches,
        retrieve_context, save_workspace_profile, list_workspace_profiles,
        update_registered_repository_metadata, EngineInfo, InclusionReasonKind, OmissionReasonKind,
        RepositorySyncStatus, RetrievalMode, TRUNCATED_PACK_MARKER,
    };

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
        assert!(assembly.memory_notes.is_empty());
        assert_eq!(assembly.snippets[0].path, "alpha.txt");
        assert_eq!(assembly.snippets[0].reason.kind, InclusionReasonKind::QueryLineMatch);
        assert!(assembly.snippets[0].reason.detail.contains("query 'needle'"));
        assert_eq!(assembly.omissions[0].kind, OmissionReasonKind::ItemLimitReached);
        assert!(assembly.omissions[0].detail.contains("limit 2 was reached"));
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
        assert!(history[0].memory_notes.is_empty());
        assert!(history[0].omissions.iter().any(|item| item.kind == OmissionReasonKind::NoLineMatches));
        assert!(history[0].omissions.iter().any(|item| item.kind == OmissionReasonKind::NoMemoryMatches));
    }

    #[test]
    fn context_run_detail_reads_one_recent_run_by_timestamp() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let run = assemble_context(&root, "needle", 2).expect("assembly should succeed");
        let detail = context_run_detail(&root, run.generated_at_epoch_ms)
            .expect("detail lookup should succeed")
            .expect("run detail should exist");

        assert_eq!(detail.generated_at_epoch_ms, run.generated_at_epoch_ms);
        assert_eq!(detail.snippets[0].reason.kind, InclusionReasonKind::QueryLineMatch);
        assert!(context_run_detail(&root, run.generated_at_epoch_ms + 1)
            .expect("missing detail lookup should succeed")
            .is_none());
    }

    #[test]
    fn assembled_context_prefers_rust_capsules_over_function_bodies() {
        let root = temp_repo();
        fs::write(
            root.join("lib.rs"),
            "use std::fmt;\n\nstruct Widget {\n    id: usize,\n}\n\nfn plan() -> usize {\n    let body_only_term = 41;\n    body_only_term\n}\n",
        )
        .expect("rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let assembly = assemble_context(&root, "Widget", 2).expect("assembly should succeed");
        assert_eq!(assembly.snippets.len(), 1);
        assert_eq!(assembly.snippets[0].path, "lib.rs");
        assert!(assembly.snippets[0].line.contains("struct Widget { id: usize }"));

        let body_query = assemble_context(&root, "body_only_term", 2).expect("body query should succeed");
        assert!(body_query.snippets.is_empty());
    }

    #[test]
    fn overview_capsule_returns_bounded_indexed_documents() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
        fs::write(root.join("beta.txt"), "beta overview\n").expect("beta file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let overview = assemble_overview(&root, 1).expect("overview should succeed");

        assert_eq!(overview.documents.len(), 1);
        assert_eq!(overview.documents[0].path, "alpha.txt");
        assert_eq!(overview.documents[0].contents, "alpha overview\n");
        assert_eq!(overview.documents[0].reason.kind, InclusionReasonKind::OverviewDocument);
        assert_eq!(overview.omissions[0].kind, OmissionReasonKind::ItemLimitReached);
    }

    #[test]
    fn task_capsule_prefers_structural_capsules_and_falls_back_to_raw_text() {
        let root = temp_repo();
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n\nfn plan() -> usize {\n    let body_only_term = 41;\n    body_only_term\n}\n",
        )
        .expect("rust file should write");
        fs::write(
            root.join("broken.rs"),
            "fn broken( {\n    let raw_fallback_term = 1;\n}\n",
        )
        .expect("broken rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let capsule = assemble_task_capsule(&root, "Widget", 2).expect("task capsule should succeed");
        assert_eq!(capsule.documents.len(), 1);
        assert_eq!(capsule.documents[0].path, "lib.rs");
        assert_eq!(capsule.documents[0].reason.kind, InclusionReasonKind::TaskCapsuleMatch);
        assert!(capsule.documents[0].contents.contains("struct Widget { id: usize }"));
        assert!(!capsule.documents[0].contents.contains("body_only_term"));

        let fallback = assemble_task_capsule(&root, "raw_fallback_term", 2).expect("fallback capsule should succeed");
        assert_eq!(fallback.documents.len(), 1);
        assert_eq!(fallback.documents[0].path, "broken.rs");
        assert!(fallback.documents[0].contents.contains("raw_fallback_term"));
    }

    #[test]
    fn retrieve_context_exposes_explicit_modes() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "alpha needle\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let exact = retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 2)
            .expect("exact_search should succeed");
        assert_eq!(exact.mode, RetrievalMode::ExactSearch);
        assert_eq!(exact.query.as_deref(), Some("needle"));
        assert_eq!(exact.snippets.len(), 1);
        assert!(exact.memory_notes.is_empty());
        assert!(exact.documents.is_empty());

        let overview = retrieve_context(&root, RetrievalMode::Overview, None, 2)
            .expect("overview should succeed");
        assert_eq!(overview.mode, RetrievalMode::Overview);
        assert!(overview.memory_notes.is_empty());
        assert_eq!(overview.documents.len(), 1);
        assert!(overview.snippets.is_empty());

        let missing = retrieve_context(&root, RetrievalMode::TaskCapsule, None, 2)
            .expect_err("task_capsule without query should fail");
        assert_eq!(missing.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn exact_search_reports_byte_budget_omission() {
        let root = temp_repo();
        let long_line = format!("needle {}", "x".repeat(300));
        fs::write(root.join("alpha.txt"), format!("{long_line}\n")).expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let assembly = assemble_context(&root, "needle", 2).expect("assembly should succeed");

        assert_eq!(assembly.snippets.len(), 1);
        assert!(assembly.snippets[0].line.ends_with(TRUNCATED_PACK_MARKER));
        assert_eq!(assembly.omissions[0].kind, OmissionReasonKind::ByteBudgetReached);
    }

    #[test]
    fn task_capsule_reports_document_byte_budget_omission() {
        let root = temp_repo();
        let mut rust_file = String::new();
        for index in 0..80 {
            rust_file.push_str(&format!("fn widget_{index}() -> usize {{\n    {index}\n}}\n\n"));
        }
        fs::write(root.join("lib.rs"), rust_file).expect("rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let capsule = assemble_task_capsule(&root, "widget_0", 2).expect("task capsule should succeed");

        assert_eq!(capsule.documents.len(), 1);
        assert!(capsule.documents[0].contents.ends_with(TRUNCATED_PACK_MARKER));
        assert_eq!(capsule.omissions[0].kind, OmissionReasonKind::ByteBudgetReached);
    }

    #[test]
    fn task_capsule_prefers_symbol_matches_before_raw_text_mentions() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "Widget appears in planning notes\n")
            .expect("alpha file should write");
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let capsule = assemble_task_capsule(&root, "Widget", 2).expect("task capsule should succeed");

        assert_eq!(capsule.documents[0].path, "lib.rs");
    }

    #[test]
    fn durable_memory_persists_and_reads_notes() {
        let root = temp_repo();

        let created = memory_write(
            &root,
            "Design note",
            "Durable memory must survive process restarts.",
            &["memory".to_string(), "design".to_string()],
        )
        .expect("memory write should succeed");

        let loaded = memory_read(&root, &created.note.id)
            .expect("memory read should succeed")
            .expect("note should exist");

        assert_eq!(loaded.title, "Design note");
        assert_eq!(loaded.tags, vec!["memory", "design"]);
    }

    #[test]
    fn durable_memory_updates_and_deletes_notes() {
        let root = temp_repo();

        let created = memory_write(
            &root,
            "Design note",
            "Durable memory must survive process restarts.",
            &["memory".to_string(), "design".to_string()],
        )
        .expect("memory write should succeed");

        let updated = memory_update(
            &root,
            &created.note.id,
            Some("Updated note"),
            Some("Updated memory content."),
            Some(&["updated".to_string()]),
        )
        .expect("memory update should succeed");

        assert_eq!(updated.note.id, created.note.id);
        assert_eq!(updated.note.title, "Updated note");
        assert_eq!(updated.note.content, "Updated memory content.");
        assert_eq!(updated.note.tags, vec!["updated"]);
        assert!(updated.note.updated_at_epoch_ms >= created.note.updated_at_epoch_ms);

        let loaded = memory_read(&root, &created.note.id)
            .expect("memory read should succeed")
            .expect("updated note should exist");

        assert_eq!(loaded.title, "Updated note");
        assert_eq!(loaded.content, "Updated memory content.");
        assert_eq!(loaded.tags, vec!["updated"]);

        let deleted = memory_delete(&root, &created.note.id).expect("memory delete should succeed");
        assert_eq!(deleted.note.as_ref().map(|note| note.id.as_str()), Some(created.note.id.as_str()));
        assert!(memory_read(&root, &created.note.id).expect("memory read should succeed").is_none());
    }

    #[test]
    fn durable_memory_exports_and_imports_bounded_json() {
        let source = temp_repo();
        let target = temp_repo();

        let first = memory_write(
            &source,
            "Design note",
            "Durable memory must survive migration.",
            &["memory".to_string()],
        )
        .expect("first memory write should succeed");
        memory_write(
            &source,
            "Second note",
            "Second migrated memory.",
            &["migration".to_string()],
        )
        .expect("second memory write should succeed");

        let exported = memory_export(&source, 10).expect("memory export should succeed");
        assert_eq!(exported.payload.notes.len(), 2);
        assert_eq!(exported.omitted_count, 0);

        let imported = memory_import(&target, exported.payload).expect("memory import should succeed");
        assert_eq!(imported.imported_count, 2);
        assert_eq!(imported.replaced_count, 0);
        assert_eq!(imported.omitted_count, 0);

        let loaded = memory_read(&target, &first.note.id)
            .expect("memory read should succeed")
            .expect("imported note should exist");
        assert_eq!(loaded.title, "Design note");
    }

    #[test]
    fn durable_memory_import_replaces_duplicate_ids() {
        let root = temp_repo();

        let created = memory_write(
            &root,
            "Original note",
            "Original content.",
            &["original".to_string()],
        )
        .expect("memory write should succeed");

        let mut replacement = created.note.clone();
        replacement.title = "Replacement note".to_string();
        replacement.content = "Replacement content.".to_string();
        replacement.tags = vec!["replacement".to_string()];

        let imported = memory_import(&root, MemoryExportPayload { notes: vec![replacement] })
            .expect("memory import should succeed");
        assert_eq!(imported.imported_count, 0);
        assert_eq!(imported.replaced_count, 1);
        assert_eq!(imported.omitted_count, 0);

        let loaded = memory_read(&root, &created.note.id)
            .expect("memory read should succeed")
            .expect("replacement note should exist");
        assert_eq!(loaded.title, "Replacement note");
        assert_eq!(loaded.content, "Replacement content.");
        assert_eq!(loaded.tags, vec!["replacement"]);
    }

    #[test]
    fn durable_memory_search_is_bounded_and_persistent() {
        let root = temp_repo();
        for index in 0..6 {
            memory_write(
                &root,
                &format!("Memory {index}"),
                "Context memory entry",
                &["memory".to_string()],
            )
            .expect("memory write should succeed");
        }

        let results = memory_search(&root, "memory", 3).expect("memory search should succeed");

        assert_eq!(results.notes.len(), 3);
        assert_eq!(results.omitted_count, 3);
        assert!(root.join(".quotarelay").join("memory_notes.json").exists());
    }

    #[test]
    fn exact_search_includes_bounded_memory_notes_with_typed_reasons() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");
        memory_write(
            &root,
            "Needle note",
            &format!("needle {}", "x".repeat(400)),
            &["memory".to_string()],
        )
        .expect("memory write should succeed");

        let assembly = assemble_context(&root, "needle", 2).expect("assembly should succeed");

        assert_eq!(assembly.memory_notes.len(), 1);
        assert_eq!(assembly.memory_notes[0].reason.kind, InclusionReasonKind::MemoryNoteMatch);
        assert!(assembly.memory_notes[0].content.ends_with(TRUNCATED_PACK_MARKER));
        assert!(assembly.omissions.iter().any(|item| item.kind == OmissionReasonKind::ByteBudgetReached));
    }

    #[test]
    fn exact_search_cache_persists_and_reuses_previous_payload() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let first = assemble_context(&root, "needle", 2).expect("first assembly should succeed");
        fs::write(root.join("alpha.txt"), "changed contents without the query\n")
            .expect("repo file should rewrite");

        let second = assemble_context(&root, "needle", 2).expect("second assembly should succeed");

        assert_eq!(second.query, first.query);
        assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
        assert_eq!(second.snippets.len(), first.snippets.len());
        assert_eq!(second.snippets[0].path, first.snippets[0].path);
        assert_eq!(second.snippets[0].line_number, first.snippets[0].line_number);
        assert_eq!(second.snippets[0].line, first.snippets[0].line);
        assert_eq!(second.snippets[0].reason.kind, first.snippets[0].reason.kind);
        assert_eq!(second.snippets[0].reason.detail, first.snippets[0].reason.detail);
        assert_eq!(second.memory_notes.len(), first.memory_notes.len());
        assert_eq!(second.omissions, first.omissions);
        assert!(root.join(".quotarelay").join("exact_match_cache.json").exists());
    }

    #[test]
    fn exact_search_cache_misses_for_different_queries() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let cached = assemble_context(&root, "needle", 2).expect("needle assembly should succeed");
        let miss = assemble_context(&root, "missing", 2).expect("missing assembly should succeed");

        assert_eq!(cached.snippets.len(), 1);
        assert!(miss.snippets.is_empty());
        assert!(miss.omissions.iter().any(|item| item.kind == OmissionReasonKind::NoLineMatches));
    }

    #[test]
    fn exact_search_cache_canonicalizes_equivalent_queries() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let first = assemble_context(&root, "Needle", 2).expect("first assembly should succeed");
        fs::write(root.join("alpha.txt"), "changed contents without the query\n").expect("repo file should rewrite");

        let second = assemble_context(&root, "  needle  ", 2).expect("second assembly should succeed");

        assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
        assert_eq!(second.query, first.query);
        assert_eq!(second.snippets[0].path, first.snippets[0].path);
        assert_eq!(second.snippets[0].line, first.snippets[0].line);
    }

    #[test]
    fn task_capsule_cache_canonicalizes_equivalent_queries() {
        let root = temp_repo();
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let first = assemble_task_capsule(&root, "Widget", 2).expect("first task capsule should succeed");
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: u32,\n}\n",
        )
        .expect("rust file should rewrite");

        let second = assemble_task_capsule(&root, " widget ", 2).expect("second task capsule should succeed");

        assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
        assert_eq!(second.documents[0].path, first.documents[0].path);
        assert_eq!(second.documents[0].contents, first.documents[0].contents);
    }

    #[test]
    fn exact_search_cache_is_cleared_on_sync_invalidation() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle in repo\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let cached = assemble_context(&root, "needle", 2).expect("initial assembly should succeed");
        fs::write(root.join("alpha.txt"), "fresh needle after sync\n").expect("repo file should rewrite");
        repo_index::sync_repo(&root).expect("resync should succeed");
        invalidate_exact_match_cache(&root).expect("cache invalidation should succeed");

        let refreshed = assemble_context(&root, "needle", 2).expect("refreshed assembly should succeed");

        assert_ne!(refreshed.generated_at_epoch_ms, cached.generated_at_epoch_ms);
        assert_ne!(refreshed.snippets[0].line, cached.snippets[0].line);
        assert_eq!(refreshed.snippets[0].line, "fresh needle after sync");
    }

    #[test]
    fn overview_cache_persists_and_reuses_previous_payload() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let first = assemble_overview(&root, 2).expect("first overview should succeed");
        fs::write(root.join("alpha.txt"), "changed overview without sync\n")
            .expect("alpha file should rewrite");

        let second = assemble_overview(&root, 2).expect("second overview should succeed");

        assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
        assert_eq!(second.documents.len(), first.documents.len());
        assert_eq!(second.documents[0].path, first.documents[0].path);
        assert_eq!(second.documents[0].contents, first.documents[0].contents);
        assert!(root.join(".quotarelay").join("retrieval_capsules.json").exists());
    }

    #[test]
    fn task_capsule_cache_persists_and_reuses_previous_payload() {
        let root = temp_repo();
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let first = assemble_task_capsule(&root, "Widget", 2).expect("first task capsule should succeed");
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: u32,\n}\n",
        )
        .expect("rust file should rewrite");

        let second = assemble_task_capsule(&root, "Widget", 2).expect("second task capsule should succeed");

        assert_eq!(second.generated_at_epoch_ms, first.generated_at_epoch_ms);
        assert_eq!(second.documents.len(), first.documents.len());
        assert_eq!(second.documents[0].path, first.documents[0].path);
        assert_eq!(second.documents[0].contents, first.documents[0].contents);
    }

    #[test]
    fn capsule_caches_are_cleared_on_sync_invalidation() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "alpha overview\n").expect("alpha file should write");
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: usize,\n}\n",
        )
        .expect("rust file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        let cached_overview = assemble_overview(&root, 2).expect("cached overview should succeed");
        let cached_task = assemble_task_capsule(&root, "Widget", 2).expect("cached task capsule should succeed");

        fs::write(root.join("alpha.txt"), "fresh overview after sync\n").expect("alpha file should rewrite");
        fs::write(
            root.join("lib.rs"),
            "struct Widget {\n    id: u32,\n}\n",
        )
        .expect("rust file should rewrite");
        repo_index::sync_repo(&root).expect("resync should succeed");
        invalidate_exact_match_cache(&root).expect("cache invalidation should succeed");

        let refreshed_overview = assemble_overview(&root, 2).expect("refreshed overview should succeed");
        let refreshed_task = assemble_task_capsule(&root, "Widget", 2).expect("refreshed task capsule should succeed");

        assert_ne!(refreshed_overview.generated_at_epoch_ms, cached_overview.generated_at_epoch_ms);
        assert_ne!(refreshed_overview.documents[0].contents, cached_overview.documents[0].contents);
        assert_eq!(refreshed_overview.documents[0].contents, "fresh overview after sync\n");
        assert_ne!(refreshed_task.generated_at_epoch_ms, cached_task.generated_at_epoch_ms);
        assert_ne!(refreshed_task.documents[0].contents, cached_task.documents[0].contents);
        assert!(refreshed_task.documents[0].contents.contains("id: u32"));
    }

    #[test]
    fn register_and_list_repositories_are_persistent_and_deduplicated() {
        let state_root = temp_repo();
        let repo_one = temp_repo();
        let repo_two = temp_repo();

        let first = register_repository(&state_root, &repo_one).expect("first registration should succeed");
        let second = register_repository(&state_root, &repo_two).expect("second registration should succeed");
        let duplicate = register_repository(&state_root, &repo_one).expect("duplicate registration should succeed");
        let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");

        assert_eq!(listed.len(), 2);
        assert_eq!(duplicate.repository, first.repository);
        assert_eq!(listed[0].root, first.repository.root);
        assert_eq!(listed[1].root, second.repository.root);
        assert!(state_root.join(".quotarelay").join("registered_repositories.json").exists());
    }

    #[test]
    fn remove_registered_repository_updates_persistent_state() {
        let state_root = temp_repo();
        let repo_root = temp_repo();

        let registered = register_repository(&state_root, &repo_root)
            .expect("registration should succeed");
        let removed = remove_registered_repository(&state_root, &repo_root)
            .expect("removal should succeed");
        let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");

        assert_eq!(removed.repository, Some(registered.repository));
        assert!(listed.is_empty());
    }

    #[test]
    fn update_registered_repository_metadata_changes_display_name_only() {
        let state_root = temp_repo();
        let repo_root = temp_repo();

        let registered = register_repository(&state_root, &repo_root)
            .expect("registration should succeed");
        let updated = update_registered_repository_metadata(
            &state_root,
            &repo_root,
            Some("Workspace API"),
        )
        .expect("metadata update should succeed")
        .repository
        .expect("registered repository should be updated");

        assert_eq!(updated.id, registered.repository.id);
        assert_eq!(updated.root, registered.repository.root);
        assert_eq!(updated.name, "Workspace API");

        let listed = list_registered_repositories(&state_root, 10).expect("listing should succeed");
        assert_eq!(listed[0].name, "Workspace API");
        assert_eq!(listed[0].root, registered.repository.root);

        let missing_root = temp_repo();
        assert!(update_registered_repository_metadata(&state_root, &missing_root, Some("Missing"))
            .expect("missing metadata update should succeed")
            .repository
            .is_none());
    }

    #[test]
    fn registered_repository_state_reports_sync_and_recent_run_truth() {
        let state_root = temp_repo();
        let repo_root = temp_repo();
        fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

        register_repository(&state_root, &repo_root).expect("registration should succeed");
        repo_index::sync_repo(&repo_root).expect("sync should succeed");
        assemble_context(&repo_root, "needle", 2).expect("assembly should succeed");

        let state = registered_repository_state(&state_root, 5).expect("repository state should succeed");

        assert_eq!(state.len(), 1);
        assert_eq!(state[0].sync.status, RepositorySyncStatus::Indexed);
        assert_eq!(state[0].sync.indexed_files, 1);
        assert!(state[0].sync.indexed_at_epoch_ms.is_some());
        assert_eq!(state[0].recent_context_run.as_ref().map(|run| run.query.as_str()), Some("needle"));
    }

    #[test]
    fn registered_repository_detail_reports_one_repo_truth() {
        let state_root = temp_repo();
        let repo_root = temp_repo();
        let missing_root = temp_repo();
        fs::write(repo_root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");

        let registered = register_repository(&state_root, &repo_root)
            .expect("registration should succeed");
        repo_index::sync_repo(&repo_root).expect("sync should succeed");
        assemble_context(&repo_root, "needle", 2).expect("assembly should succeed");

        let detail = registered_repository_detail(&state_root, &repo_root)
            .expect("repository detail should succeed")
            .expect("registered repository detail should exist");

        assert_eq!(detail.repository, registered.repository);
        assert_eq!(detail.sync.status, RepositorySyncStatus::Indexed);
        assert_eq!(detail.sync.indexed_files, 1);
        assert_eq!(detail.recent_context_run.as_ref().map(|run| run.query.as_str()), Some("needle"));
        assert!(registered_repository_detail(&state_root, &missing_root)
            .expect("missing repository detail should succeed")
            .is_none());
    }

    #[test]
    fn assemble_context_for_registered_repositories_is_bounded_per_repo() {
        let state_root = temp_repo();
        let repo_one = temp_repo();
        let repo_two = temp_repo();
        fs::write(repo_one.join("alpha.txt"), "needle one\nneedle two\n").expect("repo one file should write");
        fs::write(repo_two.join("beta.txt"), "needle three\nneedle four\n").expect("repo two file should write");

        register_repository(&state_root, &repo_one).expect("repo one registration should succeed");
        register_repository(&state_root, &repo_two).expect("repo two registration should succeed");
        repo_index::sync_repo(&repo_one).expect("repo one sync should succeed");
        repo_index::sync_repo(&repo_two).expect("repo two sync should succeed");

        let assembly = assemble_context_for_registered_repositories(
            &state_root,
            &[repo_one.clone(), repo_two.clone()],
            "needle",
            1,
        )
        .expect("multi repo assembly should succeed");

        assert_eq!(assembly.query, "needle");
        assert_eq!(assembly.repositories.len(), 2);
        assert_eq!(assembly.repositories[0].assembly.snippets.len(), 1);
        assert_eq!(assembly.repositories[1].assembly.snippets.len(), 1);
        assert!(assembly.repositories[0]
            .assembly
            .omissions
            .iter()
            .any(|omission| omission.kind == OmissionReasonKind::ItemLimitReached));
    }

    #[test]
    fn workspace_profiles_persist_repo_groups_and_default_limits() {
        let state_root = temp_repo();
        let repo_one = temp_repo();
        let repo_two = temp_repo();

        let saved = save_workspace_profile(
            &state_root,
            "local-api",
            &[repo_one.clone(), repo_two.clone()],
            RetrievalMode::TaskCapsule,
            9,
            8,
        )
        .expect("workspace profile should save");

        assert_eq!(saved.profile.name, "local-api");
        assert_eq!(saved.profile.repo_roots.len(), 2);
        assert_eq!(saved.profile.default_mode, RetrievalMode::TaskCapsule);
        assert_eq!(saved.profile.default_limit, 5);
        assert_eq!(saved.profile.per_repo_limit, 5);

        let replacement = save_workspace_profile(
            &state_root,
            "local-api",
            &[repo_one],
            RetrievalMode::ExactSearch,
            2,
            1,
        )
        .expect("workspace profile replacement should save");
        let profiles = list_workspace_profiles(&state_root, 10).expect("workspace profiles should list");

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0], replacement.profile);
        assert_eq!(profiles[0].default_mode, RetrievalMode::ExactSearch);
        assert_eq!(profiles[0].default_limit, 2);
        assert_eq!(profiles[0].per_repo_limit, 1);
        assert!(state_root.join(".quotarelay").join("workspace_profiles.json").exists());
    }

    #[test]
    fn corrupt_local_json_state_returns_recoverable_errors() {
        let root = temp_repo();
        let state_dir = root.join(".quotarelay");
        fs::create_dir_all(&state_dir).expect("state dir should create");

        for file_name in [
            "memory_notes.json",
            "registered_repositories.json",
            "context_runs.json",
            "exact_match_cache.json",
        ] {
            fs::write(state_dir.join(file_name), "{ not json").expect("corrupt state should write");
        }

        for error in [
            memory_search(&root, "needle", 3).expect_err("corrupt memory should fail"),
            list_registered_repositories(&root, 10).expect_err("corrupt registration should fail"),
            context_run_history(&root, 3).expect_err("corrupt history should fail"),
            retrieve_context(&root, RetrievalMode::ExactSearch, Some("needle"), 3)
                .expect_err("corrupt exact cache should fail"),
        ] {
            assert_eq!(error.kind(), ErrorKind::InvalidData);
            let message = error.to_string();
            assert!(message.contains("corrupt local state file"));
            assert!(message.contains("repair or remove the file to recover"));
        }
    }

    #[test]
    fn retrieval_cache_inspection_and_clear_are_explicit() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");

        assemble_context(&root, "needle", 2).expect("exact search should populate cache");
        assemble_overview(&root, 2).expect("overview should populate cache");

        let before = inspect_retrieval_caches(&root).expect("cache inspection should succeed");
        assert_eq!(before.exact_search_cache.present, true);
        assert_eq!(before.exact_search_cache.item_count, 1);
        assert_eq!(before.retrieval_capsule_cache.present, true);
        assert_eq!(before.retrieval_capsule_cache.item_count, 1);

        let cleared = clear_retrieval_caches(&root).expect("cache clear should succeed");
        assert_eq!(cleared.exact_search_cache_cleared, true);
        assert_eq!(cleared.retrieval_capsule_cache_cleared, true);

        let after = inspect_retrieval_caches(&root).expect("cache inspection should succeed");
        assert_eq!(after.exact_search_cache.present, false);
        assert_eq!(after.exact_search_cache.item_count, 0);
        assert_eq!(after.retrieval_capsule_cache.present, false);
        assert_eq!(after.retrieval_capsule_cache.item_count, 0);
    }

    #[test]
    fn inspect_local_state_reports_bounded_presence_and_counts() {
        let root = temp_repo();
        fs::write(root.join("alpha.txt"), "needle in repo\n").expect("repo file should write");
        repo_index::sync_repo(&root).expect("sync should succeed");
        memory_write(&root, "Needle note", "needle memory", &["memory".to_string()])
            .expect("memory write should succeed");
        register_repository(&root, &root).expect("repository registration should succeed");
        assemble_context(&root, "needle", 2).expect("exact assembly should succeed");
        assemble_overview(&root, 2).expect("overview assembly should succeed");

        let state = inspect_local_state(&root).expect("local state inspection should succeed");

        assert_eq!(state.index.present, true);
        assert_eq!(state.index.item_count, 1);
        assert_eq!(state.memory_notes.present, true);
        assert_eq!(state.memory_notes.item_count, 1);
        assert_eq!(state.context_run_history.present, true);
        assert_eq!(state.context_run_history.item_count, 1);
        assert_eq!(state.registered_repositories.present, true);
        assert_eq!(state.registered_repositories.item_count, 1);
        assert_eq!(state.exact_search_cache.present, true);
        assert_eq!(state.exact_search_cache.item_count, 1);
        assert_eq!(state.retrieval_capsule_cache.present, true);
        assert_eq!(state.retrieval_capsule_cache.item_count, 1);
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
