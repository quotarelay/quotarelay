use super::*;

pub fn register_repository(
    state_root: &Path,
    repo_root: &Path,
) -> io::Result<RepositoryRegistrationResult> {
    let normalized_root = normalized_repo_root(repo_root)?;
    let mut stored = load_registered_repositories(state_root)?;

    if let Some(existing) = stored
        .repositories
        .iter()
        .find(|repository| repository.root == normalized_root)
        .cloned()
    {
        return Ok(RepositoryRegistrationResult {
            repository: existing,
        });
    }

    let repository = RegisteredRepository {
        id: normalized_root.clone(),
        name: repo_name(repo_root),
        root: normalized_root,
        registered_at_epoch_ms: now_epoch_ms()?,
    };

    stored.repositories.push(repository.clone());
    stored
        .repositories
        .sort_by(|left, right| left.root.cmp(&right.root));
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

    Ok(RepositoryRemovalResult {
        repository: removed,
    })
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

    Ok(RepositoryMetadataUpdateResult {
        repository: updated,
    })
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
        repositories.push(RepositoryContextAssembly {
            repository,
            assembly,
        });
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

    if let Some(existing) = stored
        .profiles
        .iter_mut()
        .find(|existing| existing.name == profile.name)
    {
        *existing = profile.clone();
    } else {
        stored.profiles.push(profile.clone());
        stored
            .profiles
            .sort_by(|left, right| left.name.cmp(&right.name));
        if stored.profiles.len() > MAX_WORKSPACE_PROFILES {
            stored.profiles.truncate(MAX_WORKSPACE_PROFILES);
        }
    }

    persist_workspace_profiles(state_root, &stored)?;

    Ok(WorkspaceProfileSaveResult { profile })
}

pub fn list_workspace_profiles(
    state_root: &Path,
    limit: usize,
) -> io::Result<Vec<WorkspaceProfile>> {
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
