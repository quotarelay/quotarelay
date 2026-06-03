# MCP Client Presets

These presets are local templates for clients that support stdio MCP servers with a command and argument list.

## Verified Locally

- `generic-stdio.json`: starts Quotarelay from a shell whose current working directory is the Quotarelay checkout.

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

## Boundaries

- These presets do not install, publish, push, deploy, or contact model providers.
- Client-specific config keys vary by client; use `generic-stdio.json` as the verified local command/args payload.
- Run `cargo run -p mcp-server -- --cli truth` from the checkout before adding the preset to a client.
