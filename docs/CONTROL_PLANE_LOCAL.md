# Running The Control Plane Locally

The control plane is a local frontend for the shipped backend truth surface. It renders backend truth and tool-backed results only; it does not invent health, readiness, savings, auth, or provider state.

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
npm --prefix web/controlplane run build
```

Run the local dev server:

```powershell
npm --prefix web/controlplane run dev
```

The dev server binds to `127.0.0.1:4174` and proxies the shipped backend JSON routes to `127.0.0.1:3030` for local preview only.

Open:

- `http://127.0.0.1:4174/` for the public product homepage.
- `http://127.0.0.1:4174/control-plane?root=<repo_root>&memory_query=<query>` for the local operator control plane.

The frontend reads the backend truth surface. If it cannot reach the backend URL configured by the local environment, it must show unavailable/not-configured state rather than fake readiness.

## Boundaries

- No deployment is configured by this command.
- No auth is implemented for local HTTP endpoints.
- No provider calls are made.
- No cloud sync or multi-user tenancy is implied.
