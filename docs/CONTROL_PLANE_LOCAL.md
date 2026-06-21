# Running The Control Plane Locally

The control plane is a local frontend for the shipped backend truth surface. It renders backend truth and tool-backed results only; it does not invent health, readiness, savings, auth, or provider state.

## Normal Local Dashboard

Run the dashboard from the repository root:

```powershell
npm run start
```

This command builds the TeraJS dashboard, starts the loopback backend if it is not already running, then serves the built dashboard at `127.0.0.1:4174`. Stopping the command also stops the backend it started.

## Headless Backend

Run backend truth only, without the dashboard UI:

```powershell
npm run headless
```

Use this for terminal-only or MCP-adjacent workflows that only need the local HTTP truth routes.

## Console Usage Snapshot

Show the dashboard-style usage snapshot in the terminal:

```powershell
npm run usage -- [repo-root] [memory-query]
```

The reply includes a status line, MCP tool usage distribution, provider-call sparkline values, system health, local usage counts, and collapsed section headers.

## Backend HTTP Truth Surface

Run the backend HTTP server from the repository root:

```powershell
cargo run -p mcp-server -- --http 127.0.0.1:3030
```

The local HTTP backend refuses non-loopback bind addresses. Do not expose these endpoints on a public or LAN interface.

Then verify the truth endpoint in another terminal:

```powershell
Invoke-RestMethod http://127.0.0.1:3030/truth
```

The shipped HTTP routes are:

- `GET /truth`
- `GET /repositories?root=<state_root>&limit=20`
- `GET /memory?root=<repo_root>&query=<query>&limit=3`
- `GET /context-runs?root=<repo_root>&limit=5`

## Frontend

Build the control plane:

```powershell
npm run ui:build
```

Run the dashboard dev loop when editing the UI:

```powershell
npm run dev
```

Run only the frontend dev server when the backend is already managed separately:

```powershell
npm run dev:frontend
```

The dashboard server binds to `127.0.0.1:4174` and proxies the shipped backend JSON routes to `127.0.0.1:3030` for local preview only.

Open:

- `http://127.0.0.1:4174/` for the public product homepage.
- `http://127.0.0.1:4174/control-plane?root=<repo_root>&memory_query=<query>` for the local operator control plane.

The frontend reads the backend truth surface. If it cannot reach the backend URL configured by the local environment, it must show unavailable/not-configured state rather than fake readiness.

## Customization

The dashboard is source-customizable. Edit files under `web/controlplane/src`, then rerun `npm run start` to rebuild and serve the local dashboard.

## Boundaries

- No deployment is configured by this command.
- No auth is implemented for local HTTP endpoints.
- No provider calls are made.
- No cloud sync or multi-user tenancy is implied.
