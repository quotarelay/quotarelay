# MCP Client Configuration

Quotarelay runs as a local stdio MCP server. Any client that supports a command plus argument list for stdio MCP servers can launch it from this checkout.

Quotarelay is an agent-facing MCP tool, not a desktop or mobile app. Native companion apps are deferred unless users later need a wrapper for local service management or one-click client setup.

## Server Command

From the repository root:

```powershell
cargo run -p mcp-server
```

For clients that need command and args split:

```json
{
  "command": "cargo",
  "args": ["run", "-p", "mcp-server"]
}
```

The same verified local command/args payload is checked in at `examples/mcp-client-presets/generic-stdio.json`.

This configuration starts the MCP server only. It does not start the HTTP control plane, install packages, publish artifacts, or contact model providers.

## Installed Command

For a source-first local install smoke, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-smoke.ps1
```

That script runs `cargo install --path apps/mcp-server --root target\install-smoke --debug --force --locked --offline` and proves the installed `quotarelay-mcp --cli truth` command works. Run `scripts\bootstrap.ps1` first on a fresh checkout so Cargo dependencies are available locally.

For clients that launch an installed command:

```json
{
  "command": "quotarelay-mcp",
  "args": []
}
```

The installed-command preset is checked in at `examples/mcp-client-presets/installed-stdio.json`.

## Windows Path Examples

If your client supports a working directory field, point it at the checkout:

```json
{
  "command": "cargo",
  "args": ["run", "-p", "mcp-server"],
  "cwd": "C:\\path\\to\\quotarelay"
}
```

When passing repository or state roots to Quotarelay tools, use normal Windows paths. The backend canonicalizes registered repository roots:

```json
{
  "root": "C:\\path\\to\\quotarelay",
  "repo_root": "C:\\path\\to\\quotarelay"
}
```

## State Root Guidance

Quotarelay stores local state under `.quotarelay` inside the root you pass to a tool.

- Use a repo root when you want index, context history, memory, and cache to live beside that repo.
- Use a separate local state root when registering multiple repos as a workspace.
- Do not commit `.quotarelay` unless you have intentionally reviewed the stored local state.

## Local Validation

Before configuring a client, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1
cargo run -p mcp-server -- --cli truth
```

To prove the installed command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-smoke.ps1
```

For a fuller repo check:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\clean-check.ps1
```

## Stdio Troubleshooting

| Symptom | Check |
|---|---|
| Client cannot start server | Run `cargo run -p mcp-server` from the checkout and confirm Cargo can build locally. |
| Installed command is missing | Run `scripts\install-smoke.ps1` and point the client at the installed `quotarelay-mcp` command or use the source-run preset. |
| Client starts in the wrong folder | Add `cwd` if the client supports it, or use an absolute path to the checkout before launching. |
| Tool calls cannot find repo state | Confirm the `root` or `repo_root` argument points at the same local path used for sync/register. |
| Search returns no results | Run `sync_repo` first; the index is explicit and local. |
| Corrupt local state error | Inspect `.quotarelay` and repair or remove the named corrupt JSON file by operator choice. |
| Unexpected future/platform feature missing | Check `docs/TRUTH_MATRIX.md`; auth, provider routing, cloud sync, and multi-user tenancy are deferred. |
