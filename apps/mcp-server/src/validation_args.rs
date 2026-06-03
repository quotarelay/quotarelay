use context_engine::{recommend_validation, ValidationRecommendationResult};
use serde_json::Value;

pub(crate) fn validation_recommend_from_args(
    arguments: &Value,
) -> Result<ValidationRecommendationResult, String> {
    let paths = arguments
        .get("paths")
        .and_then(Value::as_array)
        .ok_or_else(|| "validation_recommend requires paths array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| "validation_recommend paths must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(recommend_validation(&paths))
}
