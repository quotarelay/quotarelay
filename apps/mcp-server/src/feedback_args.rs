use std::path::PathBuf;

use context_engine::{
    context_feedback_list, context_feedback_write, ContextFeedbackListResult,
    ContextFeedbackRating, ContextFeedbackWriteResult,
};
use serde_json::Value;

pub(crate) fn context_feedback_write_from_args(
    arguments: &Value,
) -> Result<ContextFeedbackWriteResult, String> {
    let root = parse_root(arguments)?;
    let generated_at_epoch_ms = arguments
        .get("generated_at_epoch_ms")
        .and_then(Value::as_u64)
        .map(u128::from)
        .ok_or_else(|| {
            "context_feedback_write requires numeric generated_at_epoch_ms".to_string()
        })?;
    let rating = parse_feedback_rating(arguments)?;
    let reason = arguments
        .get("reason")
        .and_then(Value::as_str)
        .ok_or_else(|| "context_feedback_write requires string reason".to_string())?;

    context_feedback_write(&root, generated_at_epoch_ms, rating, reason)
        .map_err(|error| format!("context_feedback_write failed: {error}"))
}

pub(crate) fn context_feedback_list_from_args(
    arguments: &Value,
) -> Result<ContextFeedbackListResult, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(5);

    context_feedback_list(&root, limit)
        .map_err(|error| format!("context_feedback_list failed: {error}"))
}

fn parse_feedback_rating(arguments: &Value) -> Result<ContextFeedbackRating, String> {
    match arguments.get("rating").and_then(Value::as_str) {
        Some("useful") => Ok(ContextFeedbackRating::Useful),
        Some("not_useful") => Ok(ContextFeedbackRating::NotUseful),
        Some(other) => Err(format!(
            "context feedback rating must be useful or not_useful; got {other}"
        )),
        None => Err("context feedback requires string rating".to_string()),
    }
}

fn parse_root(arguments: &Value) -> Result<PathBuf, String> {
    let root = arguments
        .get("root")
        .and_then(Value::as_str)
        .ok_or_else(|| "tool requires a string root".to_string())?;
    Ok(PathBuf::from(root))
}
