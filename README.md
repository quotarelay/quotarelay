# Quotarelay

Quotarelay is an MCP-first context engine for local agent workflows. The shipped workspace has three Rust crates and one Terajs control-plane scaffold:

Current local MVP version: `0.1.0`.

- `apps/mcp-server`: stdio MCP server plus a thin HTTP `/truth` endpoint.
- `crates/repo-index`: local repository indexing, inventory, and search.
- `crates/context-engine`: bounded context assembly, durable memory, repository registration, and derived repository state truth.
- `web/controlplane`: a Terajs page that reads the live backend `/truth` payload and renders the current backend contract.

## Shipped behavior

Today the backend exposes these MCP tools:

- `bootstrap_status`
- `sync_repo`
- `repo_inventory`
- `repo_map`
- `cache_inspect`
- `cache_clear`
- `register_repository`
- `list_repositories`
- `repository_state`
- `repository_detail`
- `repository_update_metadata`
- `remove_repository`
- `workspace_profile_save`
- `workspace_profile_list`
- `team_policy_profile_save`
- `team_policy_profile_list`
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
- `context_feedback_write`
- `context_feedback_list`
- `validation_recommend`
- `multi_repo_assemble_context`
- `assemble_context`
- `handoff_packet`

The retrieval contract is explicit and bounded:

- Modes: `exact_search`, `overview`, `task_capsule`, `diff_aware`
- Local-change mode: `diff_aware` assembles bounded context around changed/new files and optional related indexed matches
- Hard limits: 5 context items, 160 snippet bytes, 640 document bytes, 320 memory-note bytes, 10 history runs
- Explainability: typed inclusion and omission reasons are preserved across engine and MCP boundaries
- Budget estimate: context packs include local raw bytes considered, included-byte counts, approximate tokens, and a derived reduction ratio
- Stale status: context packs report whether indexed files appear changed, missing, or new since the last explicit sync
- Cache status: context packs report local cache hit, miss, or not-applicable state; cache clear reports cleared or empty state
- Clarification: underspecified exact/task requests return 1-3 focused questions instead of broad context dumps
- Durable memory: stored locally under `.quotarelay/`
- Memory profiles: notes can be marked as `normal`, `decision`, or `guardrail`; decision and guardrail matches are prioritized in bounded context and handoff packets
- Registered repository state: stored locally and exposed with sync status, indexed counts, bounded repo maps, and recent run metadata
- Team policy profiles: stored locally with guardrails, validation recipes, MCP client presets, and explicit source-upload preference
- Validation recommendations: exact local commands with reasons for touched or queried paths; commands are returned, not run
- Context feedback: local bounded records for useful/not-useful context packs; feedback is inspectable and does not change ranking
- Repo index ignore config: optional `.quotarelay/ignore.json` with `paths` for exact relative paths and `prefixes` for relative directory/file prefixes; changes apply on the next explicit sync

The control plane does not invent readiness, savings, or live health state. It renders the current backend truth contract from `/truth`.

## Quickstart

From a fresh checkout on this machine:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1
cargo run -p mcp-server -- --cli truth
```

To run against one of your own repos:

```powershell
cargo run -p mcp-server -- --cli register <state_root> <repo_root>
cargo run -p mcp-server -- --cli sync <repo_root>
cargo run -p mcp-server -- --cli map <repo_root>
cargo run -p mcp-server -- --cli validate crates/context-engine/src/lib.rs README.md
cargo run -p mcp-server -- --cli feedback-write <repo_root> <generated_at_epoch_ms> useful "kept the packet focused"
cargo run -p mcp-server -- --cli feedback-list <repo_root> 5
cargo run -p mcp-server -- --cli team-policy-save <state_root> backend "keep source local" "cargo test -p mcp-server" "examples/mcp-client-presets/generic-stdio.json"
cargo run -p mcp-server -- --cli team-policy-list <state_root> 5
cargo run -p mcp-server -- --cli search <repo_root> <query> 5
cargo run -p mcp-server -- --cli assemble <repo_root> exact_search <query> 3
cargo run -p mcp-server -- --cli state <repo_root>
```

Use a dedicated local `state_root` when you want to register multiple repositories as a workspace. Quotarelay writes local state under `.quotarelay` inside the roots you pass to tools.

## Local validation

- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\clean-check.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\token-saver-benchmark.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\local-package.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1`
- `cargo test -p mcp-server backend_truth_endpoint_exposes_current_contract`
- `cargo test -p mcp-server repository_state_tool_reports_sync_and_recent_run_truth_over_stdio`
- `cargo test -p mcp-server repository_registration_tools_work_over_stdio`
- `cargo test -p context-engine registered_repository_state_reports_sync_and_recent_run_truth`
- `npm --prefix web/controlplane run build`

## Local CLI

The MCP server binary also exposes a non-interactive local CLI for operator workflows. CLI responses are newline-terminated JSON objects with `ok`, `command`, and either `result` or `error` plus `error_category`.

- `cargo run -p mcp-server -- --cli truth`
- `cargo run -p mcp-server -- --cli register <state_root> <repo_root>`
- `cargo run -p mcp-server -- --cli sync <repo_root>`
- `cargo run -p mcp-server -- --cli state <repo_root>`
- `cargo run -p mcp-server -- --cli map <repo_root>`
- `cargo run -p mcp-server -- --cli validate <path> [path...]`
- `cargo run -p mcp-server -- --cli feedback-write <repo_root> <generated_at_epoch_ms> useful|not_useful <reason>`
- `cargo run -p mcp-server -- --cli feedback-list <repo_root> [limit]`
- `cargo run -p mcp-server -- --cli team-policy-save <state_root> <name> <guardrails_semicolon_list> <validation_recipes_semicolon_list> <mcp_presets_semicolon_list> [true|false]`
- `cargo run -p mcp-server -- --cli team-policy-list <state_root> [limit]`
- `cargo run -p mcp-server -- --cli search <repo_root> <query> [limit]`
- `cargo run -p mcp-server -- --cli assemble <repo_root> exact_search <query> [limit]`
- `cargo run -p mcp-server -- --cli assemble <repo_root> overview [limit]`
- `cargo run -p mcp-server -- --cli assemble <repo_root> task_capsule <query> [limit]`
- `cargo run -p mcp-server -- --cli assemble <repo_root> diff_aware [query] [limit]`
- `cargo run -p mcp-server -- --cli handoff <repo_root> <active_task> exact_search <query> [limit]`
- `cargo run -p mcp-server -- --cli handoff-template <repo_root> <active_task> bug_fix|feature_slice|review|refactor|release exact_search <query> [limit]`

## MCP client setup

See `docs/MCP_CLIENT_CONFIG.md` and `examples/mcp-client-presets/` for the local stdio command, generic client JSON shape, Windows path examples, state-root guidance, and startup troubleshooting.

## Troubleshooting

See `docs/TROUBLESHOOTING.md` for corrupt local JSON recovery, cache inspect/clear guidance, missing index/repository checks, Windows path notes, and deferred platform boundaries.

## Changelog

See `CHANGELOG.md` for local MVP release notes, validation commands, known limitations, and deferred platform boundaries.

## Control plane

See `docs/CONTROL_PLANE_LOCAL.md` for local backend HTTP and frontend commands. The Terajs frontend serves the public product homepage at `/` and the local operator control plane at `/control-plane`.

## Token-saver benchmark

Run the one-command local demo with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\token-saver-demo.ps1
```

The demo creates a temporary fixture and reports broad raw repo size versus Quotarelay packet size, cache miss then cache hit, stale status after a local file change, and the exact context and handoff packets. It does not call providers or claim billing savings.

Run the local acceptance benchmark with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\token-saver-benchmark.ps1
```

The benchmark creates a temporary fixture, syncs it locally, records decision memory, assembles exact/overview/handoff/diff-aware packets, checks exact-search cache entries across repeated assembly, and reports raw bytes versus packet bytes with approximate tokens. It does not call providers or claim billing savings.

## Local package smoke

Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\local-package.ps1` to build a local-only zip artifact under `target\local-package\`. The script smoke-runs the packaged backend binary with `--cli truth`; it does not publish, sign, install globally, deploy, or contact external services.

After extracting the zip, run the packaged backend locally with:

```powershell
.\bin\mcp-server.exe --cli truth
.\bin\mcp-server.exe --http 127.0.0.1:3030
```

## Local state privacy

See `docs/LOCAL_STATE_PRIVACY.md` for what `.quotarelay` stores locally, what the local MVP does not send, what not to commit, and current limitations.

## Security

See `SECURITY.md` for supported security boundaries, vulnerability reporting guidance, local data handling, and the loopback-only HTTP policy.

## Public repo guidance

- `AGENTS.md`: instructions for AI agents and bots working in this repo.
- `CONTRIBUTING.md`: contribution workflow and validation expectations.
- `SUPPORT.md`: support boundaries and issue guidance.
- `docs/VERSIONING.md`: versioning and release checklist.
- `docs/DEPLOYMENT_READINESS.md`: what is local-only today and what is required before hosted deployment.
- `docs/PLATFORM_SURFACES.md`: current DOM/local surfaces, planned deployment management, and deferred Android/iOS preview boundaries.
- `docs/HOSTED_THREAT_MODEL.md`: hosted auth, tenancy, audit, privacy, integration, and deployment security gate.
- `docs/PRIVATE_DEPLOYMENT_PLAN.md`: planned private deployment shapes, artifact boundaries, and container readiness tests.
- `docs/PUBLICATION_CHECKLIST.md`: public push/release checklist.
- `docs/COMMERCIAL_STRATEGY.md`: open-core monetization path that keeps the local core free.

## License

Quotarelay is licensed under Apache-2.0. See `LICENSE` and `NOTICE`.

## Known limitations

See `docs/KNOWN_LIMITATIONS.md` for local-only boundaries, indexing limits, provider/network non-goals, control-plane limits, and deferred platform work.
