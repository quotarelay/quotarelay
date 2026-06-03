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

## Release gap matrix

| Category | Status | Evidence or follow-up |
|---|---|---|
| Repo sync, search, inventory | Shipped local MVP behavior | Covered by `repo-index` tests and MCP `sync_repo`, `repo_inventory`, and `search_code` tools. |
| Context assembly | Shipped local MVP behavior | `exact_search`, `overview`, and `task_capsule` are bounded and explain inclusion/omission reasons. |
| Durable memory | Shipped local MVP behavior | Write/read/update/delete/search/export/import are local and bounded; empty memory search is rejected. |
| Cache inspection and clear | Shipped local MVP behavior | Cache inspect/clear, corrupt cache errors, canonicalization, separation, and sync invalidation are tested. |
| Repository registration and profiles | Shipped local MVP behavior | Registration, dedupe, removal, metadata rename, detail lookup, bounded listing, and workspace profiles are tested. |
| Truth surfaces | Shipped local MVP behavior | `/truth`, CLI truth, OpenAPI mirror, and control-plane rendering are constrained to shipped backend behavior. |
| One-command clean check | Shipped local validation behavior | `scripts/clean-check.ps1` runs fmt check, line-count guardrail, Rust package tests, and control-plane build. |
| Fresh checkout bootstrap | Local release blocker | T82 must provide a local bootstrap check that does not mutate global state or require external services. |
| MCP client configuration | Local release blocker | T83 must document verified local client setup, state-root guidance, Windows path examples, and troubleshooting. |
| Example workspace and demo | Local release blocker | T84 must provide a tiny local example plus a no-service demo script. |
| Operator quickstart | Local release blocker | T85 must make README usable from clone to first context pack using shipped commands only. |
| Troubleshooting and recovery | Local release blocker | T86 must document corrupt JSON recovery, cache clear, missing index, missing repo, and Windows path handling. |
| Local privacy and security note | Local release blocker | T87 must state what is stored under `.quotarelay`, what is not sent, and current limitations without encryption/auth claims. |
| OpenAPI release contract freeze | Local release blocker | T88 must freeze shipped HTTP schema without adding auth, deployment, or fake readiness fields. |
| Release validation suite | Local release blocker | T89 must provide a release validation command that builds on local clean-check and demo proof. |
| Control-plane local run instructions | Local release blocker | T90 must document running backend HTTP truth and the control plane locally. |
| Version metadata and notes | Local release blocker | T91-T93 must align version metadata, changelog/release notes, and known limitations. |
| Release blocker triage and dry run | Local release blocker | T94-T100 must finish blocker triage, artifact smoke, release candidate dry run, final review, commit plan, and go/no-go. |
| Auth, multi-user tenancy, BYOK, provider routing, cloud sync | Deferred platform capability | Closed by decisions until explicit decision slices open them; not required for local MVP releasability. |
| Token-saving clarity and benchmarks | Post-core token-saver release path | T101-T114 add local estimates, handoff packets, stale/diff context, repo maps, validation recommendations, cache visibility, savings clarity, and demos without provider billing claims. |
