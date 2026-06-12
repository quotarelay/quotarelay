# MCP Tool And CLI Reference

This page holds the detailed shipped tool surface so `README.md` can stay focused on first-run adoption.

## MCP Tools

The stdio MCP server currently exposes:

| Tool | Purpose |
|---|---|
| `bootstrap_status` | Returns the Quotarelay bootstrap banner. |
| `sync_repo` | Scans a local repository and writes `.quotarelay/index.json`. |
| `repo_inventory` | Returns bounded local index inventory. |
| `repo_map` | Returns bounded top-level directories and detected Rust symbols. |
| `cache_inspect` | Reports retrieval cache presence and item counts. |
| `cache_clear` | Explicitly clears local retrieval cache files. |
| `register_repository` | Registers a repository root in local state. |
| `list_repositories` | Lists registered repositories from local state. |
| `repository_state` | Returns registered repository sync and recent-run truth. |
| `repository_detail` | Returns one registered repository detail. |
| `repository_update_metadata` | Updates display metadata for one registered repository. |
| `remove_repository` | Removes a registered repository from local state. |
| `workspace_profile_save` | Saves a local workspace profile with repo roots and defaults. |
| `workspace_profile_list` | Lists local workspace profiles. |
| `team_policy_profile_save` | Saves local guardrails, validation recipes, MCP presets, and source-upload preference. |
| `team_policy_profile_list` | Lists local team policy profiles. |
| `search_code` | Searches the persisted local repository index. |
| `memory_write` | Writes a durable local memory note. |
| `memory_read` | Reads one durable memory note by id. |
| `memory_update` | Updates a durable memory note. |
| `memory_delete` | Deletes a durable memory note. |
| `memory_export` | Exports bounded durable memory notes as local JSON. |
| `memory_import` | Imports bounded durable memory JSON. |
| `memory_search` | Searches durable memory notes. |
| `context_run_detail` | Reads one recent context run. |
| `context_run_history` | Lists recent context runs with reasons. |
| `context_feedback_write` | Records bounded local feedback for a generated context pack. |
| `context_feedback_list` | Lists bounded local context feedback. |
| `validation_recommend` | Returns exact local validation commands with reasons. |
| `savings_report` | Aggregates local context-run byte/token/cache metadata. |
| `onboarding_pack` | Returns repo-shape metadata, validation commands, templates, and privacy notes. |
| `multi_repo_assemble_context` | Builds bounded context across explicitly selected registered repositories. |
| `assemble_context` | Builds a bounded context pack from the local index. |
| `handoff_packet` | Builds a bounded agent handoff packet. |

## Retrieval Contract

Retrieval is explicit and bounded:

- Modes: `exact_search`, `overview`, `task_capsule`, `diff_aware`
- Hard limits: 5 context items, 160 snippet bytes, 640 document bytes, 320 memory-note bytes, 10 history runs
- Inclusion and omission reasons are typed and preserved across engine, MCP, CLI, and context history boundaries
- Context packets include local raw bytes considered, included bytes, approximate tokens, and reduction ratio
- Stale status reports changed, missing, or new files relative to the last explicit sync
- Cache status reports hit, miss, or not-applicable
- Underspecified exact/task requests return bounded clarification questions instead of broad dumps

## CLI Examples

From the checkout, use the root aliases first. The lower-level source command is `cargo run -p mcp-server --`. After local install smoke, the branded command is `quotarelay-mcp`.

```powershell
npm run truth
npm run usage -- [repo_root] [memory_query]
npm run register -- <state_root> <repo_root>
npm run sync -- <repo_root>
npm run state -- <repo_root>
npm run search -- <repo_root> <query> [limit]
npm run context -- <repo_root> exact_search <query> [limit]
npm run context -- <repo_root> overview [limit]
npm run context -- <repo_root> task_capsule <query> [limit]
npm run context -- <repo_root> diff_aware [query] [limit]
npm run handoff -- <repo_root> <active_task> bug_fix|feature_slice|review|refactor|release exact_search <query> [limit]
```

The underlying CLI remains available for less common commands:

```powershell
cargo run -p mcp-server -- --cli map <repo_root>
cargo run -p mcp-server -- --cli handoff <repo_root> <active_task> exact_search <query> [limit]
cargo run -p mcp-server -- --cli savings-report <repo_root> [limit]
cargo run -p mcp-server -- --cli validate <path> [path...]
quotarelay-mcp --cli truth
```

CLI responses are newline-terminated JSON objects with `ok`, `command`, and either `result` or `error` plus `error_category`.

## Local State

Quotarelay writes local state under `.quotarelay/` in roots passed to tools. State can include repository index metadata, memory notes, cache entries, context history, registered repositories, workspace profiles, and team policy profiles.

Do not commit `.quotarelay/` without reviewing it. See `docs/LOCAL_STATE_PRIVACY.md`.
