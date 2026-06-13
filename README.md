# Quotarelay

Quotarelay is a local-first MCP context engine for coding agents. It helps agents ask for smaller, fresher, better-explained repo context instead of repeatedly pulling broad source dumps into a chat.

It runs locally, stores state under `.quotarelay/`, and does not call model providers.

Current local MVP version: `0.1.0`.

## Why It Exists

AI coding agents are useful, but large repos make context messy:

- agents reread the same files over and over,
- handoffs lose decisions and validation steps,
- stale file state is easy to miss,
- broad context dumps waste tokens,
- teams need guardrails without sending source to a hosted service.

Quotarelay gives agents a local MCP tool for bounded repo search, durable memory, cache visibility, handoff packets, validation recommendations, and honest context-size proof.

## What Ships

- Local stdio MCP server with the installed `quotarelay-mcp` command.
- Local CLI for `truth`, `sync`, `search`, `assemble`, `handoff`, `state`, and related workflows.
- Bounded retrieval modes: `exact_search`, `overview`, `task_capsule`, and `diff_aware`.
- Durable local memory with normal, decision, and guardrail profiles.
- Local cache inspection/clear plus cache-hit visibility in context packets.
- Handoff packets with `general`, `bug_fix`, `feature_slice`, `review`, `refactor`, and `release` templates.
- Local savings reports, onboarding packs, team policy profiles, and MCP client presets.
- Thin local HTTP truth routes for the control plane, bound to loopback only.

No hosted login, provider gateway, telemetry, cloud sync, auth, billing, or exact provider billing-savings claim ships in this release.

## Proof Snapshot

These numbers come from `scripts\token-saver-benchmark.ps1`, which builds a temporary local fixture, syncs it, writes decision memory, assembles context packets, checks cache behavior, and reports local byte plus approximate-token estimates. They are not provider tokenizer output or billing guarantees.

| Packet | Raw bytes | Packet bytes | Approx raw tokens | Approx packet tokens | Local reduction |
|---|---:|---:|---:|---:|---:|
| `exact_search` | 24,132 | 1,361 | 6,033 | 341 | 94.36% |
| `overview` | 24,132 | 1,435 | 6,033 | 359 | 94.05% |
| `handoff_packet` | 24,132 | 2,676 | 6,033 | 669 | 88.91% |
| `diff_aware` | 24,274 | 1,663 | 6,069 | 416 | 93.15% |

The local demo also proves the first exact search is a cache miss, the repeated exact search is a cache hit, a handoff packet is generated, backend truth exposes 35 tools, and `provider_calls` is `none`.

## Quickstart

From a fresh checkout:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1
npm run truth
npm run usage
```

Expected proof signals:

- `bootstrap passed`
- `repeated_exact_cache_status: "hit"`
- `handoff_template: "feature_slice"`
- `truth_tool_count: 35`
- `provider_calls: "none"`
- `usage` returns a compact console snapshot with status, MCP tool usage, provider calls, and collapsed usage sections

Prove the installed local MCP command and checked-in client presets:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-smoke.ps1
target\install-smoke\bin\quotarelay-mcp.exe --cli truth
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\mcp-preset-smoke.ps1 -SkipInstall
```

## Use It On A Repo

```powershell
npm run register -- <state_root> <repo_root>
npm run sync -- <repo_root>
npm run usage -- <repo_root> <memory_query>
npm run search -- <repo_root> <query> 5
npm run context -- <repo_root> exact_search <query> 3
npm run handoff -- <repo_root> "Continue the feature" feature_slice exact_search <query> 3
npm run state -- <repo_root>
```

Use a separate local `state_root` when registering multiple repos as a workspace. Quotarelay writes local state under `.quotarelay` inside the roots you pass to tools.

## MCP Client Setup

Use the source-run preset first:

```json
{
  "command": "cargo",
  "args": ["run", "-p", "mcp-server"]
}
```

Use the installed-command preset only after `scripts\install-smoke.ps1` passes and your MCP client can resolve `quotarelay-mcp`:

```json
{
  "command": "quotarelay-mcp",
  "args": []
}
```

See `docs/MCP_CLIENT_CONFIG.md` and `examples/mcp-client-presets/` for working-directory guidance and troubleshooting.

## Validation

The main public proof path is:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

It runs clean checks, Rust tests, control-plane tests/build, local demo proof, token-saver benchmark proof, install smoke, MCP preset smoke, public-site smoke, public-surface scan, and docs sanity.

Focused checks:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\token-saver-benchmark.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\mcp-preset-smoke.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
cargo test -p mcp-server
npm run ui:build
```

## Project Layout

| Path | Purpose |
|---|---|
| `apps/mcp-server` | stdio MCP server, local CLI, and loopback HTTP truth backend |
| `crates/repo-index` | local repository sync, inventory, map, document lookup, and search |
| `crates/context-engine` | bounded retrieval, memory, cache, handoff, repository state, reports |
| `web/controlplane` | Terajs public homepage and local control-plane truth view |
| `examples/demo-repo` | tiny local fixture used by `scripts\demo-local.ps1` |
| `examples/mcp-client-presets` | source-run and installed-command stdio presets |

## Docs

- `docs/MCP_TOOL_REFERENCE.md`: shipped MCP tools, retrieval contract, and CLI examples.
- `docs/MCP_CLIENT_CONFIG.md`: stdio client setup, presets, Windows path examples, and startup troubleshooting.
- `docs/TROUBLESHOOTING.md`: corrupt local JSON recovery, missing sync/index checks, cache guidance, and path notes.
- `docs/TRUTH_MATRIX.md`: shipped behavior, useful next work, deferred decisions, and explicit non-goals.
- `docs/LOCAL_STATE_PRIVACY.md`: what `.quotarelay` stores locally and what not to commit.
- `docs/KNOWN_LIMITATIONS.md`: local-only boundaries, indexing limits, provider/network non-goals, and control-plane limits.
- `docs/PUBLICATION_CHECKLIST.md`: public push/release checklist.
- `docs/RELEASE_FLOW.md`: version, release-prep, package, and local tag flow.
- `docs/GITHUB_ABOUT.md`: suggested GitHub About description and topics.

## Boundaries

Quotarelay is local-first:

- no provider calls or provider keys,
- no telemetry or remote feedback collection,
- no hosted tenancy or cloud sync,
- no auth or access control on local HTTP routes,
- no exact provider billing savings claim,
- no background sync, cache warming, or silent local-state repair.

See `SECURITY.md`, `SUPPORT.md`, `CONTRIBUTING.md`, and `AGENTS.md` for public operating guidance.

## License

Quotarelay is licensed under Apache-2.0. See `LICENSE` and `NOTICE`.
