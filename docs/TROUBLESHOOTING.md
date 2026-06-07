# Troubleshooting And Recovery

Quotarelay is local-first. Most issues are caused by missing explicit setup steps or local JSON state under `.quotarelay`.

## Quick Checks

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1
cargo run -p mcp-server -- --cli truth
cargo run -p mcp-server -- --cli state <repo_root>
```

Use `state` to inspect bounded local presence and counts. It does not dump file contents, memory contents, or cache payloads.

## Fresh Machine Setup

Run bootstrap before deeper checks:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1
```

If `rustc` or `cargo` is not found, install or repair the Rust toolchain and restart the terminal so `cargo` is on `PATH`. If Rust is installed but only this shell is stale, start a new shell and rerun bootstrap.

If `npm` is not found, install or repair Node.js and restart the terminal. For a backend-only check, run bootstrap with `-SkipFrontend`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1 -SkipFrontend
```

If frontend commands fail after a clean checkout, run `npm --prefix web/controlplane ci` when registry access is available, then rerun bootstrap or `scripts\release-check.ps1`.

## Wrong Working Directory

Most source-run commands expect the Quotarelay checkout as the current directory. From elsewhere, either change into the checkout first or configure the MCP client `cwd` field.

```powershell
cargo run -p mcp-server -- --cli truth
```

If a client cannot start the source-run preset, confirm the same command works in a normal terminal from the checkout.

## Installed Command Not On PATH

`scripts\install-smoke.ps1` installs into `target\install-smoke\bin` and proves the command there. That directory is intentionally local to the repo and is not a global installer.

Use the full local path when testing directly:

```powershell
target\install-smoke\bin\quotarelay-mcp.exe --cli truth
```

For an MCP client, use `examples/mcp-client-presets/installed-stdio.json` only after the client can resolve `quotarelay-mcp` from its environment. Otherwise use the source-run preset with `cargo`.

## Missing Index

Search and context assembly require an explicit sync first.

```powershell
cargo run -p mcp-server -- --cli sync <repo_root>
cargo run -p mcp-server -- --cli search <repo_root> <query> 5
```

If search still returns no hits, confirm the file is not excluded by normal index exclusions or `.quotarelay/ignore.json`.

## Missing Registered Repository

Repository state and multi-repo workflows use a state root plus registered repo roots.

```powershell
cargo run -p mcp-server -- --cli register <state_root> <repo_root>
```

Use the same `state_root` when listing or querying registered repositories. Use the same canonical repo path when updating metadata, removing a repo, or requesting detail.

## Cache Inspect And Clear

Use local state inspection to confirm whether retrieval caches exist:

```powershell
cargo run -p mcp-server -- --cli state <repo_root>
```

The MCP tools `cache_inspect` and `cache_clear` expose cache presence/counts and explicit clearing. Clearing cache files is an operator action; it does not delete the repository index, memory notes, registered repository state, or context run history.

## Corrupt Local JSON

Corrupt local state errors include the file path and tell the operator to repair or remove the named file. Review the named file before changing it.

Common local state files:

- `.quotarelay/index.json`
- `.quotarelay/memory_notes.json`
- `.quotarelay/context_runs.json`
- `.quotarelay/exact_match_cache.json`
- `.quotarelay/retrieval_capsules.json`
- `.quotarelay/registered_repositories.json`
- `.quotarelay/workspace_profiles.json`

Recovery options are operator choices:

- Repair the JSON if the intended contents are clear.
- Move the corrupt file aside and rerun the explicit command that owns it.
- Clear only retrieval cache files when the cache is the named corrupt artifact.

Do not remove all `.quotarelay` state unless you have reviewed what is stored there and intentionally want to reset local Quotarelay state.

## Windows Paths

Quotarelay canonicalizes registered repo roots. These path forms should resolve consistently when they point at the same local repo:

```text
C:\path\to\quotarelay
C:/path/to/quotarelay
C:\path\to\quotarelay\.
```

If a client starts in the wrong directory, configure its `cwd` or use an absolute checkout path in the MCP client configuration.

## Deferred Features

If a tool or UI surface appears to be missing auth, provider routing, cloud sync, multi-user tenancy, or exact provider billing savings, check `docs/TRUTH_MATRIX.md`. Those platform features are deferred and are not required for the local MVP.
