use context_engine::{onboarding_pack, OnboardingPack};
use serde_json::Value;

use crate::args::parse_root;

pub(crate) fn onboarding_pack_from_args(arguments: &Value) -> Result<OnboardingPack, String> {
    let root = parse_root(arguments)?;
    let touched_paths = arguments
        .get("touched_paths")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| "onboarding_pack touched_paths must be strings".to_string())
                })
                .collect()
        })
        .unwrap_or_else(|| Ok(Vec::new()))?;

    onboarding_pack(&root, &touched_paths)
        .map_err(|error| format!("onboarding_pack failed: {error}"))
}
