use std::io::{self, Write};
use std::path::PathBuf;

use context_engine::{
    assemble_handoff_packet, inspect_local_state, invalidate_exact_match_cache,
    register_repository, retrieve_context, RetrievalMode,
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
        "search" => run_cli_search(&mut args_iter),
        "assemble" => run_cli_assemble(&mut args_iter),
        "handoff" => run_cli_handoff(&mut args_iter),
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
