use std::io::{self, Write};
use std::path::PathBuf;

use context_engine::{
    assemble_handoff_packet, assemble_handoff_packet_with_template, context_feedback_list,
    context_feedback_write, inspect_local_state, invalidate_exact_match_cache,
    list_team_policy_profiles, parse_handoff_template, recommend_validation, register_repository,
    retrieve_context, save_team_policy_profile, savings_report, ContextFeedbackRating,
    RetrievalMode,
};
use repo_index::{repo_map, search_code, sync_repo};
use serde_json::{json, Value};

use crate::backend_truth_payload;

pub fn run_cli<I, S, W>(args: I, mut stdout: W) -> io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    W: Write,
{
    let mut args_iter = args.into_iter();
    let command = match args_iter.next() {
        Some(arg) => arg.as_ref().to_string(),
        None => {
            write_cli_error(&mut stdout, None, "missing command")?;
            return Ok(());
        }
    };

    let result = match command.as_str() {
        "register" => run_cli_register(&mut args_iter),
        "sync" => run_cli_sync(&mut args_iter),
        "state" => run_cli_state(&mut args_iter),
        "map" => run_cli_map(&mut args_iter),
        "validate" => run_cli_validate(&mut args_iter),
        "feedback-write" => run_cli_feedback_write(&mut args_iter),
        "feedback-list" => run_cli_feedback_list(&mut args_iter),
        "savings-report" => run_cli_savings_report(&mut args_iter),
        "team-policy-save" => run_cli_team_policy_save(&mut args_iter),
        "team-policy-list" => run_cli_team_policy_list(&mut args_iter),
        "search" => run_cli_search(&mut args_iter),
        "assemble" => run_cli_assemble(&mut args_iter),
        "handoff" => run_cli_handoff(&mut args_iter),
        "handoff-template" => run_cli_handoff_template(&mut args_iter),
        "truth" => Ok(json!({"truth": backend_truth_payload()})),
        _ => {
            return write_cli_error(
                &mut stdout,
                Some(&command),
                &format!("unknown command: {command}"),
            )
        }
    };

    match result {
        Ok(result) => {
            serde_json::to_writer(
                &mut stdout,
                &json!({
                    "ok": true,
                    "command": command,
                    "result": result,
                }),
            )?;
            stdout.write_all(b"\n")?;
        }
        Err(error) => {
            write_cli_error(&mut stdout, Some(&command), &error)?;
        }
    }

    Ok(())
}

fn run_cli_register<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let state_root = parse_cli_arg(args, "state_root")?;
    let repo_root = parse_cli_arg(args, "repo_root")?;
    let registration = register_repository(&PathBuf::from(state_root), &PathBuf::from(repo_root))
        .map_err(|error| format!("register failed: {error}"))?;
    Ok(json!({"repository": registration.repository}))
}

fn run_cli_sync<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let result =
        sync_repo(&PathBuf::from(&root)).map_err(|error| format!("sync failed: {error}"))?;
    invalidate_exact_match_cache(&PathBuf::from(&root))
        .map_err(|error| format!("sync failed to invalidate cache: {error}"))?;
    Ok(json!({"sync": result}))
}

fn run_cli_state<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let state = inspect_local_state(&PathBuf::from(&root))
        .map_err(|error| format!("state failed: {error}"))?;
    Ok(json!({"state": state}))
}

fn run_cli_map<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let map = repo_map(&PathBuf::from(&root)).map_err(|error| format!("map failed: {error}"))?;
    Ok(json!({"repo_map": map}))
}

fn run_cli_validate<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let paths = args.map(|arg| arg.as_ref().to_string()).collect::<Vec<_>>();
    if paths.is_empty() {
        return Err("validate requires at least one path".to_string());
    }

    Ok(json!({"validation": recommend_validation(&paths)}))
}

fn run_cli_feedback_write<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let generated_at_epoch_ms = parse_cli_arg(args, "generated_at_epoch_ms")?
        .parse::<u128>()
        .map_err(|error| format!("generated_at_epoch_ms must be a number: {error}"))?;
    let rating = parse_cli_feedback_rating(&parse_cli_arg(args, "rating")?)?;
    let reason = parse_cli_arg(args, "reason")?;
    let result = context_feedback_write(
        &PathBuf::from(&root),
        generated_at_epoch_ms,
        rating,
        &reason,
    )
    .map_err(|error| format!("feedback-write failed: {error}"))?;

    Ok(json!({"feedback": result}))
}

fn run_cli_feedback_list<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let limit = parse_cli_limit(args, 5)?;
    let result = context_feedback_list(&PathBuf::from(&root), limit)
        .map_err(|error| format!("feedback-list failed: {error}"))?;

    Ok(json!({"feedback": result}))
}

fn run_cli_savings_report<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let limit = parse_cli_limit(args, 10)?;
    let report = savings_report(&PathBuf::from(&root), limit)
        .map_err(|error| format!("savings-report failed: {error}"))?;

    Ok(json!({"savings_report": report}))
}

fn run_cli_team_policy_save<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let name = parse_cli_arg(args, "name")?;
    let guardrails = parse_cli_list_arg(args, "guardrails")?;
    let validation_recipes = parse_cli_list_arg(args, "validation_recipes")?;
    let mcp_client_presets = parse_cli_list_arg(args, "mcp_client_presets")?;
    let allow_source_upload = parse_cli_bool_arg(args, false)?;
    let result = save_team_policy_profile(
        &PathBuf::from(&root),
        &name,
        &guardrails,
        &validation_recipes,
        &mcp_client_presets,
        allow_source_upload,
    )
    .map_err(|error| format!("team-policy-save failed: {error}"))?;

    Ok(json!({"team_policy": result}))
}

fn run_cli_team_policy_list<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let limit = parse_cli_limit(args, 20)?;
    let profiles = list_team_policy_profiles(&PathBuf::from(&root), limit)
        .map_err(|error| format!("team-policy-list failed: {error}"))?;

    Ok(json!({"team_policies": profiles}))
}

fn run_cli_search<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let query = parse_cli_arg(args, "query")?;
    let limit = parse_cli_limit(args, 5)?;
    let hits = search_code(&PathBuf::from(&root), &query, limit)
        .map_err(|error| format!("search failed: {error}"))?;
    Ok(json!({"hits": hits}))
}

fn run_cli_assemble<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let mode = parse_cli_arg(args, "mode")?;
    let (query, limit) = match mode.as_str() {
        "overview" => (None, parse_cli_limit(args, 3)?),
        "diff_aware" => parse_cli_optional_query_and_limit(args, 3)?,
        "exact_search" | "task_capsule" => {
            let query = parse_cli_arg(args, "query")?;
            (Some(query), parse_cli_limit(args, 3)?)
        }
        other => {
            return Err(format!(
                "assemble mode must be exact_search, overview, task_capsule, or diff_aware; got {other}"
            ))
        }
    };
    let retrieval = retrieve_context(
        &PathBuf::from(&root),
        parse_retrieval_mode_from_str(&mode)?,
        query.as_deref(),
        limit,
    )
    .map_err(|error| format!("assemble failed: {error}"))?;
    Ok(json!({"context": retrieval}))
}

fn run_cli_handoff<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let active_task = parse_cli_arg(args, "active_task")?;
    let mode = parse_cli_arg(args, "mode")?;
    let (query, limit) = match mode.as_str() {
        "overview" => (None, parse_cli_limit(args, 3)?),
        "diff_aware" => parse_cli_optional_query_and_limit(args, 3)?,
        "exact_search" | "task_capsule" => {
            let query = parse_cli_arg(args, "query")?;
            (Some(query), parse_cli_limit(args, 3)?)
        }
        other => {
            return Err(format!(
            "handoff mode must be exact_search, overview, task_capsule, or diff_aware; got {other}"
        ))
        }
    };
    let packet = assemble_handoff_packet(
        &PathBuf::from(&root),
        &active_task,
        parse_retrieval_mode_from_str(&mode)?,
        query.as_deref(),
        limit,
    )
    .map_err(|error| format!("handoff failed: {error}"))?;
    Ok(json!({"handoff": packet}))
}

fn run_cli_handoff_template<I, S>(args: &mut I) -> Result<Value, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let root = parse_cli_arg(args, "root")?;
    let active_task = parse_cli_arg(args, "active_task")?;
    let template_name = parse_cli_arg(args, "template")?;
    let template = parse_handoff_template(&template_name)
        .ok_or_else(|| format!("handoff template is not supported: {template_name}"))?;
    let mode = parse_cli_arg(args, "mode")?;
    let (query, limit) = match mode.as_str() {
        "overview" => (None, parse_cli_limit(args, 3)?),
        "diff_aware" => parse_cli_optional_query_and_limit(args, 3)?,
        "exact_search" | "task_capsule" => {
            let query = parse_cli_arg(args, "query")?;
            (Some(query), parse_cli_limit(args, 3)?)
        }
        other => {
            return Err(format!(
            "handoff mode must be exact_search, overview, task_capsule, or diff_aware; got {other}"
        ))
        }
    };
    let packet = assemble_handoff_packet_with_template(
        &PathBuf::from(&root),
        &active_task,
        template,
        parse_retrieval_mode_from_str(&mode)?,
        query.as_deref(),
        limit,
    )
    .map_err(|error| format!("handoff failed: {error}"))?;
    Ok(json!({"handoff": packet}))
}

fn parse_cli_arg<I, S>(args: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    args.next()
        .map(|value| value.as_ref().to_string())
        .ok_or_else(|| format!("{name} is required"))
}

fn parse_cli_arg_opt<I, S>(args: &mut I) -> Option<String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    args.next().map(|value| value.as_ref().to_string())
}

fn parse_cli_limit<I, S>(args: &mut I, default: usize) -> Result<usize, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    match parse_cli_arg_opt(args) {
        Some(value) => value
            .parse::<usize>()
            .map_err(|error| format!("limit must be a positive integer: {error}")),
        None => Ok(default),
    }
}

fn parse_cli_list_arg<I, S>(args: &mut I, name: &str) -> Result<Vec<String>, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    Ok(parse_cli_arg(args, name)?
        .split(';')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn parse_cli_bool_arg<I, S>(args: &mut I, default: bool) -> Result<bool, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    match parse_cli_arg_opt(args).as_deref() {
        None => Ok(default),
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(other) => Err(format!(
            "boolean argument must be true or false; got {other}"
        )),
    }
}

fn parse_cli_optional_query_and_limit<I, S>(
    args: &mut I,
    default_limit: usize,
) -> Result<(Option<String>, usize), String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let Some(first) = parse_cli_arg_opt(args) else {
        return Ok((None, default_limit));
    };
    if let Ok(limit) = first.parse::<usize>() {
        return Ok((None, limit));
    }
    let limit = parse_cli_limit(args, default_limit)?;
    Ok((Some(first), limit))
}

fn parse_retrieval_mode_from_str(mode: &str) -> Result<RetrievalMode, String> {
    match mode {
        "exact_search" => Ok(RetrievalMode::ExactSearch),
        "overview" => Ok(RetrievalMode::Overview),
        "task_capsule" => Ok(RetrievalMode::TaskCapsule),
        "diff_aware" => Ok(RetrievalMode::DiffAware),
        other => Err(format!(
            "assemble mode must be exact_search, overview, task_capsule, or diff_aware; got {other}"
        )),
    }
}

fn parse_cli_feedback_rating(rating: &str) -> Result<ContextFeedbackRating, String> {
    match rating {
        "useful" => Ok(ContextFeedbackRating::Useful),
        "not_useful" => Ok(ContextFeedbackRating::NotUseful),
        other => Err(format!(
            "feedback rating must be useful or not_useful; got {other}"
        )),
    }
}

fn write_cli_error<W: Write>(
    stdout: &mut W,
    command: Option<&str>,
    message: &str,
) -> io::Result<()> {
    serde_json::to_writer(
        &mut *stdout,
        &json!({
            "ok": false,
            "command": command,
            "error_category": classify_error(message),
            "error": message,
        }),
    )?;
    stdout.write_all(b"\n")
}

pub(crate) fn classify_error(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("requires")
        || lower.contains("missing")
        || lower.contains("invalid")
        || lower.contains("must be")
        || lower.contains("unknown command")
        || lower.contains("unsupported")
        || lower.contains("got ")
    {
        "invalid_args"
    } else if lower.contains("not found") || lower.contains("not registered") {
        "missing_state"
    } else if lower.contains("corrupt") || lower.contains("expected") || lower.contains("eof") {
        "corrupt_state"
    } else {
        "backend_error"
    }
}
