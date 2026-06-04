use context_engine::{savings_report, SavingsReport};
use serde_json::Value;

use crate::args::parse_root;

pub(crate) fn savings_report_from_args(arguments: &Value) -> Result<SavingsReport, String> {
    let root = parse_root(arguments)?;
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(10);

    savings_report(&root, limit).map_err(|error| format!("savings_report failed: {error}"))
}
