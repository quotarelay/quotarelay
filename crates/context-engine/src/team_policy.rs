use super::*;

pub fn save_team_policy_profile(
    state_root: &Path,
    name: &str,
    guardrails: &[String],
    validation_recipes: &[String],
    mcp_client_presets: &[String],
    allow_source_upload: bool,
) -> io::Result<TeamPolicyProfileSaveResult> {
    let name = name.trim();
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "team policy profile name cannot be empty",
        ));
    }

    let profile = TeamPolicyProfile {
        name: name.to_string(),
        guardrails: bounded_non_empty_items(guardrails),
        validation_recipes: bounded_non_empty_items(validation_recipes),
        mcp_client_presets: bounded_non_empty_items(mcp_client_presets),
        allow_source_upload,
    };
    let mut stored = load_team_policy_profiles(state_root)?;

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
        if stored.profiles.len() > MAX_TEAM_POLICY_PROFILES {
            stored.profiles.truncate(MAX_TEAM_POLICY_PROFILES);
        }
    }

    persist_team_policy_profiles(state_root, &stored)?;

    Ok(TeamPolicyProfileSaveResult { profile })
}

pub fn list_team_policy_profiles(
    state_root: &Path,
    limit: usize,
) -> io::Result<Vec<TeamPolicyProfile>> {
    let capped_limit = limit.clamp(1, MAX_TEAM_POLICY_PROFILES);
    let stored = load_team_policy_profiles(state_root)?;

    Ok(stored.profiles.into_iter().take(capped_limit).collect())
}

fn bounded_non_empty_items(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .take(MAX_TEAM_POLICY_ITEMS)
        .map(ToOwned::to_owned)
        .collect()
}
