# MVP Scope

## In scope

- Local repository sync into a persisted index under `.quotarelay/`
- Bounded code search and repository inventory
- Bounded context assembly across `exact_search`, `overview`, and `task_capsule`
- Durable local memory write, read, and search
- Repository registration with derived sync-state and recent-run truth
- A thin backend `/truth` endpoint for the control plane
- A control-plane page that renders the live backend truth payload

## Explicit non-goals

- Cloud sync
- Multi-user tenancy
- Background schedulers or workers
- Fake live progress, fake savings metrics, or invented readiness states
- Broad multi-language parsing beyond the current local index behavior
- Release or deployment automation from the agent