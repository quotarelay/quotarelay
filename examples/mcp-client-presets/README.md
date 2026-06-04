# MCP Client Presets

These presets are local templates for clients that support stdio MCP servers with a command and argument list.

## Verified Locally

- `generic-stdio.json`: starts Quotarelay from a shell whose current working directory is the Quotarelay checkout.
- `installed-stdio.json`: starts the installed `quotarelay-mcp` command after `scripts\install-smoke.ps1` or a local Cargo install.

The preset is intentionally small:

```json
{
  "command": "cargo",
  "args": ["run", "-p", "mcp-server"]
}
```

If your client supports a working-directory field, set it to your local checkout path, for example:

```text
C:\path\to\quotarelay
```

After local install smoke, clients can use the installed command directly:

```json
{
  "command": "quotarelay-mcp",
  "args": []
}
```

## Boundaries

- These presets do not install, publish, push, deploy, or contact model providers.
- Client-specific config keys vary by client; use `generic-stdio.json` for source runs and `installed-stdio.json` after local install proof.
- Run `cargo run -p mcp-server -- --cli truth` from the checkout before adding the preset to a client.
- Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-smoke.ps1` before using `installed-stdio.json`.
