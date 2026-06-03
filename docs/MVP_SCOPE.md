# MVP Scope

## In scope

- Local repository sync into a persisted index under `.quotarelay/`
- Bounded code search and repository inventory
- Bounded context assembly across `exact_search`, `overview`, and `task_capsule`
- Durable local memory write, read, and search
- Repository registration with derived sync-state and recent-run truth
- A thin backend `/truth` endpoint for the control plane
- A control-plane page that renders the live backend truth payload
- Local CLI workflows for non-interactive operator and agent use
- Local cache inspection and explicit cache clear
- Local workspace profiles and repo groups
- Local guardrails for source-size and separation-of-concerns maintainability
- Release-ready docs for local install, quickstart, troubleshooting, privacy, and known limitations
- Token-saving proof work that is local and approximate: context budget estimate, handoff packets, decision memory, stale-context detection, diff-aware context, validation recommender, and local benchmark fixtures

## Explicit non-goals

- Cloud sync
- Multi-user tenancy
- Background schedulers or workers
- Fake live progress, fake savings metrics, or invented readiness states
- Broad multi-language parsing beyond the current local index behavior
- Release or deployment automation from the agent
- Provider request forwarding or model gateway behavior
- Provider pricing recommendations or exact billing claims
- Hosted SaaS control plane for the local MVP
- Enterprise SSO, admin tenancy, or shared cloud policy until explicit decision slices open them
- Telemetry, remote feedback collection, or uploading repository contents

## Release bar

The local MVP is releasable when a new operator can clone the repo, run the documented local bootstrap, register and sync a repository, assemble bounded context, manage durable memory, inspect/clear caches, view backend truth/control-plane truth, and run release validation without external services.

The token-saving release path is releasable when Quotarelay can prove, locally and honestly, that its bounded context packs or handoff packets are smaller than broad raw repository context for representative workflows.
