# Architecture

## Runtime shape

- `apps/mcp-server` is an adapter layer.
  It owns stdio MCP transport, a thin HTTP `/truth` route, tool registration, and request argument parsing.
- `crates/context-engine` owns orchestration and durable local state.
  It assembles context, persists memory, persists repository registration, and derives repository-state truth.
- `crates/repo-index` owns repository indexing, inventory, and search over the local persisted index.
- `web/controlplane` is a read-only mirror of backend truth.
  It fetches `/truth` at runtime and renders the current backend contract.

## State boundaries

- Repository index artifacts live under each repository root in `.quotarelay/`.
- Durable memory and context run history live under the target repository root in `.quotarelay/`.
- Registered repository state lives under a caller-provided local state root in `.quotarelay/registered_repositories.json`.
- Derived repository sync state is computed from existing repo inventory and recent context history; it is not maintained as a separate mutable store.

## Transport boundaries

- MCP tools and HTTP routes do not own retrieval, persistence, or repository-state logic.
- The `/truth` route serializes the current tool registry, retrieval contract, and proof commands.
- The control plane must consume the backend payload as-is and stay within backend-proven fields.