use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::*;
use repo_index::repo_inventory;

pub(crate) fn append_history(root: &Path, assembly: &ContextAssembly) -> io::Result<()> {
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
    fs::write(
        path,
        serde_json::to_vec_pretty(&history).map_err(io::Error::other)?,
    )
}

pub(crate) fn load_history(root: &Path) -> io::Result<StoredRunHistory> {
    let path = history_path(root);
    if !path.exists() {
        return Ok(StoredRunHistory::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
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
    let feedback = load_context_feedback(root)?;
    let registered_repositories = load_registered_repositories(root)?;
    let team_policy_profiles = load_team_policy_profiles(root)?;
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
        context_feedback: LocalStateArtifact {
            present: context_feedback_path(root).exists(),
            item_count: feedback.feedback.len(),
        },
        registered_repositories: LocalStateArtifact {
            present: registered_repositories_path(root).exists(),
            item_count: registered_repositories.repositories.len(),
        },
        team_policy_profiles: LocalStateArtifact {
            present: team_policy_profiles_path(root).exists(),
            item_count: team_policy_profiles.profiles.len(),
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
    let status = if exact_search_cache_cleared || retrieval_capsule_cache_cleared {
        CacheStatus {
            kind: CacheStatusKind::Cleared,
            detail: "One or more local retrieval cache files were cleared.".to_string(),
        }
    } else {
        CacheStatus {
            kind: CacheStatusKind::Empty,
            detail: "No local retrieval cache files existed to clear.".to_string(),
        }
    };

    clear_exact_match_cache(root)?;
    clear_capsule_cache(root)?;

    Ok(CacheClearResult {
        exact_search_cache_cleared,
        retrieval_capsule_cache_cleared,
        status,
    })
}

pub(crate) fn history_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("context_runs.json")
}

pub(crate) fn context_feedback_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("context_feedback.json")
}

pub(crate) fn exact_match_cache_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("exact_match_cache.json")
}

pub(crate) fn capsule_cache_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("retrieval_capsules.json")
}

pub(crate) fn memory_notes_path(root: &Path) -> PathBuf {
    root.join(".quotarelay").join("memory_notes.json")
}

pub(crate) fn registered_repositories_path(state_root: &Path) -> PathBuf {
    state_root
        .join(".quotarelay")
        .join("registered_repositories.json")
}

pub(crate) fn workspace_profiles_path(state_root: &Path) -> PathBuf {
    state_root
        .join(".quotarelay")
        .join("workspace_profiles.json")
}

pub(crate) fn team_policy_profiles_path(state_root: &Path) -> PathBuf {
    state_root
        .join(".quotarelay")
        .join("team_policy_profiles.json")
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

pub(crate) fn load_memory_notes(root: &Path) -> io::Result<StoredMemoryNotes> {
    let path = memory_notes_path(root);
    if !path.exists() {
        return Ok(StoredMemoryNotes::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

pub(crate) fn persist_memory_notes(root: &Path, notes: &StoredMemoryNotes) -> io::Result<()> {
    let path = memory_notes_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(notes).map_err(io::Error::other)?,
    )
}

pub(crate) fn load_context_feedback(root: &Path) -> io::Result<StoredContextFeedback> {
    let path = context_feedback_path(root);
    if !path.exists() {
        return Ok(StoredContextFeedback::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

pub(crate) fn persist_context_feedback(
    root: &Path,
    feedback: &StoredContextFeedback,
) -> io::Result<()> {
    let path = context_feedback_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(feedback).map_err(io::Error::other)?,
    )
}

pub(crate) fn load_workspace_profiles(state_root: &Path) -> io::Result<StoredWorkspaceProfiles> {
    let path = workspace_profiles_path(state_root);
    if !path.exists() {
        return Ok(StoredWorkspaceProfiles::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

pub(crate) fn persist_workspace_profiles(
    state_root: &Path,
    profiles: &StoredWorkspaceProfiles,
) -> io::Result<()> {
    let path = workspace_profiles_path(state_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(profiles).map_err(io::Error::other)?,
    )
}

pub(crate) fn load_team_policy_profiles(state_root: &Path) -> io::Result<StoredTeamPolicyProfiles> {
    let path = team_policy_profiles_path(state_root);
    if !path.exists() {
        return Ok(StoredTeamPolicyProfiles::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

pub(crate) fn persist_team_policy_profiles(
    state_root: &Path,
    profiles: &StoredTeamPolicyProfiles,
) -> io::Result<()> {
    let path = team_policy_profiles_path(state_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(profiles).map_err(io::Error::other)?,
    )
}

pub(crate) fn load_exact_match_cache(root: &Path) -> io::Result<StoredExactMatchCache> {
    let path = exact_match_cache_path(root);
    if !path.exists() {
        return Ok(StoredExactMatchCache::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

pub(crate) fn load_capsule_cache(root: &Path) -> io::Result<StoredCapsuleCache> {
    let path = capsule_cache_path(root);
    if !path.exists() {
        return Ok(StoredCapsuleCache::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

pub(crate) fn read_exact_match_cache(
    root: &Path,
    query: &str,
) -> io::Result<Option<ContextAssembly>> {
    let query = normalize_query(query);
    let stored = load_exact_match_cache(root)?;
    Ok(stored
        .entries
        .into_iter()
        .find(|entry| entry.query == query)
        .map(|entry| entry.assembly))
}

pub(crate) fn persist_exact_match_cache(
    root: &Path,
    query: &str,
    assembly: &ContextAssembly,
) -> io::Result<()> {
    let query = normalize_query(query);
    let mut stored = load_exact_match_cache(root)?;
    if let Some(entry) = stored.entries.iter_mut().find(|entry| entry.query == query) {
        entry.assembly = assembly.clone();
    } else {
        stored.entries.push(StoredExactMatchCacheEntry {
            query: query.to_string(),
            assembly: assembly.clone(),
        });
        stored
            .entries
            .sort_by(|left, right| left.query.cmp(&right.query));
    }

    let path = exact_match_cache_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(&stored).map_err(io::Error::other)?,
    )
}

pub(crate) fn read_capsule_cache(
    root: &Path,
    mode: RetrievalMode,
    query: Option<&str>,
) -> io::Result<Option<ContextCapsule>> {
    let query = query.map(normalize_query);
    let stored = load_capsule_cache(root)?;
    Ok(stored
        .entries
        .into_iter()
        .find(|entry| entry.mode == mode && entry.query.as_deref() == query.as_deref())
        .map(|entry| entry.capsule))
}

pub(crate) fn persist_capsule_cache(
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
    fs::write(
        path,
        serde_json::to_vec_pretty(&stored).map_err(io::Error::other)?,
    )
}

pub(crate) fn clear_exact_match_cache(root: &Path) -> io::Result<()> {
    let path = exact_match_cache_path(root);
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
}

pub(crate) fn clear_capsule_cache(root: &Path) -> io::Result<()> {
    let path = capsule_cache_path(root);
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
}

pub(crate) fn load_registered_repositories(
    state_root: &Path,
) -> io::Result<StoredRegisteredRepositories> {
    let path = registered_repositories_path(state_root);
    if !path.exists() {
        return Ok(StoredRegisteredRepositories::default());
    }

    let bytes = fs::read(&path)?;
    parse_state_json(&path, &bytes)
}

pub(crate) fn persist_registered_repositories(
    state_root: &Path,
    repositories: &StoredRegisteredRepositories,
) -> io::Result<()> {
    let path = registered_repositories_path(state_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(repositories).map_err(io::Error::other)?,
    )
}
