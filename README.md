# Quotarelay

Quotarelay is an MCP-first context engine for local agent workflows. The shipped workspace has three Rust crates and one Terajs control-plane scaffold:

- `apps/mcp-server`: stdio MCP server plus a thin HTTP `/truth` endpoint.
- `crates/repo-index`: local repository indexing, inventory, and search.
- `crates/context-engine`: bounded context assembly, durable memory, repository registration, and derived repository state truth.
- `web/controlplane`: a Terajs page that reads the live backend `/truth` payload and renders the current backend contract.

## Shipped behavior

Today the backend exposes these MCP tools:

- `bootstrap_status`
- `sync_repo`
- `repo_inventory`
- `register_repository`
- `list_repositories`
- `repository_state`
- `remove_repository`
- `search_code`
- `memory_write`
- `memory_read`
- `memory_update`
- `memory_delete`
- `memory_export`
- `memory_import`
- `memory_search`
- `context_run_detail`
- `context_run_history`
- `assemble_context`

The retrieval contract is explicit and bounded:

- Modes: `exact_search`, `overview`, `task_capsule`
- Hard limits: 5 context items, 160 snippet bytes, 640 document bytes, 320 memory-note bytes, 10 history runs
- Explainability: typed inclusion and omission reasons are preserved across engine and MCP boundaries
- Durable memory: stored locally under `.quotarelay/`
- Registered repository state: stored locally and exposed with sync status, indexed counts, and recent run metadata

The control plane does not invent readiness, savings, or live health state. It renders the current backend truth contract from `/truth`.

## Local validation

- `cargo test -p mcp-server backend_truth_endpoint_exposes_current_contract`
- `cargo test -p mcp-server repository_state_tool_reports_sync_and_recent_run_truth_over_stdio`
- `cargo test -p mcp-server repository_registration_tools_work_over_stdio`
- `cargo test -p context-engine registered_repository_state_reports_sync_and_recent_run_truth`
- `npm --prefix web/controlplane run build`

## Local CLI

The MCP server binary also exposes a non-interactive local CLI for operator workflows. CLI responses are newline-terminated JSON objects with `ok`, `command`, and either `result` or `error`.

- `cargo run -p mcp-server -- --cli truth`
- `cargo run -p mcp-server -- --cli register <state_root> <repo_root>`
- `cargo run -p mcp-server -- --cli sync <repo_root>`
- `cargo run -p mcp-server -- --cli state <repo_root>`
- `cargo run -p mcp-server -- --cli search <repo_root> <query> [limit]`
- `cargo run -p mcp-server -- --cli assemble <repo_root> exact_search <query> [limit]`
- `cargo run -p mcp-server -- --cli assemble <repo_root> overview [limit]`
- `cargo run -p mcp-server -- --cli assemble <repo_root> task_capsule <query> [limit]`
