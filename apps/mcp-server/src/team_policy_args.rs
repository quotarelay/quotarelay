use context_engine::{
    list_team_policy_profiles, save_team_policy_profile, TeamPolicyProfile,
    TeamPolicyProfileSaveResult,
};
use serde_json::Value;

use crate::args::parse_root;

pub(crate) fn team_policy_profile_save_from_args(
    arguments: &Value,
) -> Result<TeamPolicyProfileSaveResult, String> {
    let root = parse_root(arguments)?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "team_policy_profile_save requires a string name".to_string())?;
    let guardrails = parse_string_array(arguments, "guardrails")?;
    let validation_recipes = parse_string_array(arguments, "validation_recipes")?;
    let mcp_client_presets = parse_string_array(arguments, "mcp_client_presets")?;
    let allow_source_upload = arguments
        .get("allow_source_upload")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    save_team_policy_profile(
        &root,
        name,
        &guardrails,
        &validation_recipes,
        &mcp_client_presets,
        allow_source_upload,
    )
    .map_err(|error| format!("team_policy_profile_save failed: {error}"))
}

pub(crate) fn team_policy_profile_list_from_args(
    arguments: &Value,
) -> Result<Vec<TeamPolicyProfile>, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(20);

    list_team_policy_profiles(&root, limit)
        .map_err(|error| format!("team_policy_profile_list failed: {error}"))
}

fn parse_string_array(arguments: &Value, field: &str) -> Result<Vec<String>, String> {
    arguments
        .get(field)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| format!("{field} must be an array of strings"))
                })
                .collect()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}
