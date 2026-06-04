# MVP Scope

## In scope

- Local repository sync into a persisted index under `.quotarelay/`
- Bounded code search, repository inventory, and repo map summaries
- Bounded context assembly across `exact_search`, `overview`, `task_capsule`, and `diff_aware`
- Durable local memory write, read, and search
- Repository registration with derived sync-state and recent-run truth
- A thin backend `/truth` endpoint for the control plane
- A control-plane page that renders the live backend truth payload
- Local CLI workflows for non-interactive operator and agent use
- Local cache inspection and explicit cache clear
- Local workspace profiles and repo groups
- Local guardrails for source-size and separation-of-concerns maintainability
- Release-ready docs for local install, quickstart, troubleshooting, privacy, and known limitations
- Token-saving proof work that is local and approximate: context budget estimate, handoff packets, decision memory, stale-context detection, diff-aware context, repo maps, validation recommender, context feedback, and local benchmark fixtures

## Explicit non-goals

- Cloud sync
- Multi-user tenancy
- Background schedulers or workers
- Fake live progress, fake savings metrics, or invented readiness states
- Broad multi-language parsing beyond the current local index behavior
- Release or deployment automation from the agent
- Provider request forwarding or model gateway behavior
- Provider capability detection or model recommendations
- Provider pricing recommendations or exact billing claims
- Hosted SaaS control plane for the local MVP
- BYOK key storage
- Enterprise SSO, admin tenancy, or shared cloud policy until explicit decision slices open them
- Compliance audit logs
- Telemetry, remote feedback collection, or uploading repository contents

## Release bar

The local MVP is releasable when a new operator can clone the repo, run the documented local bootstrap, register and sync a repository, assemble bounded context, manage durable memory, inspect/clear caches, view backend truth/control-plane truth, and run release validation without external services.

The token-saving release path is releasable when Quotarelay can prove, locally and honestly, that its bounded context packs or handoff packets are smaller than broad raw repository context for representative workflows.

## Release gap matrix

| Category | Status | Evidence or follow-up |
|---|---|---|
| Repo sync, search, inventory, map | Shipped local MVP behavior | Covered by `repo-index` tests and MCP `sync_repo`, `repo_inventory`, `repo_map`, and `search_code` tools. |
| Context assembly | Shipped local MVP behavior | `exact_search`, `overview`, `task_capsule`, and `diff_aware` are bounded and explain inclusion/omission reasons. |
| Durable memory | Shipped local MVP behavior | Write/read/update/delete/search/export/import are local and bounded; empty memory search is rejected. |
| Cache inspection and clear | Shipped local MVP behavior | Cache inspect/clear, corrupt cache errors, canonicalization, separation, and sync invalidation are tested. |
| Cache hit visibility | Shipped local token-saver behavior | Context packs expose hit, miss, or not-applicable cache status; cache clear reports cleared or empty state. |
| Savings clarity surface | Shipped local token-saver behavior | Context and handoff packets expose raw bytes considered, included bytes, approximate tokens, local reduction ratio, cache status, stale status, and omissions. |
| Handoff templates | Shipped local team-layer behavior | Handoff packets can carry local bug fix, feature slice, review, refactor, and release focus guidance without hosted sync or source upload. |
| Savings reports | Shipped local team-layer behavior | Recent context-run history can be aggregated into bounded local savings metadata without snippets, telemetry, or provider billing claims. |
| Onboarding packs | Shipped local team-layer behavior | MCP onboarding packs summarize bounded repo-shape metadata, validation commands, handoff templates, and privacy notes without source snippets or upload. |
| Ambiguous request clarification | Shipped local token-saver behavior | Underspecified exact/task context and handoff requests return bounded clarification questions instead of broad repo dumps. |
| Repository registration and profiles | Shipped local MVP behavior | Registration, dedupe, removal, metadata rename, detail lookup, bounded listing, and workspace profiles are tested. |
| Truth surfaces | Shipped local MVP behavior | `/truth`, CLI truth, OpenAPI mirror, and control-plane rendering are constrained to shipped backend behavior. |
| Validation recommender | Shipped local token-saver behavior | Returns exact local commands with reasons for repo-owned path patterns; it does not run tests or call external services. |
| Context feedback | Shipped local token-saver behavior | Records bounded useful/not-useful feedback locally for context packs; it is inspectable and does not change ranking. |
| Token-saver acceptance benchmark | Shipped local token-saver behavior | `scripts/token-saver-benchmark.ps1` compares raw fixture bytes against exact, overview, handoff, and diff-aware packets with cache assertions and no provider calls. |
| One-command clean check | Shipped local validation behavior | `scripts/clean-check.ps1` runs fmt check, line-count guardrail, Rust package tests, and control-plane build. |
| Fresh checkout bootstrap | Shipped local release behavior | T82 added a local bootstrap check that does not mutate global state or require external services. |
| MCP client configuration | Shipped local release behavior | T83 documents verified local client setup, state-root guidance, Windows path examples, and troubleshooting. |
| Example workspace and demo | Shipped local release behavior | T84 added a tiny local example plus a no-service demo script. |
| Operator quickstart | Shipped local release behavior | T85 made README usable from clone to first context pack using shipped commands only. |
| Troubleshooting and recovery | Shipped local release behavior | T86 documents corrupt JSON recovery, cache clear, missing index, missing repo, and Windows path handling. |
| Local privacy and security note | Shipped local release behavior | T87 states what is stored under `.quotarelay`, what is not sent, and current limitations without encryption/auth claims. |
| OpenAPI release contract freeze | Shipped local release behavior | T88 froze the shipped HTTP schema without adding auth, deployment, or fake readiness fields. |
| Release validation suite | Shipped local release behavior | T89 provides a release validation command that builds on local clean-check and demo proof. |
| Control-plane local run instructions | Shipped local release behavior | T90 documents running backend HTTP truth and the control plane locally. |
| Version metadata and notes | Shipped local release behavior | T91-T93 align version metadata, changelog/release notes, and known limitations. |
| Release blocker triage and dry run | Shipped local release behavior | T94-T100 completed blocker triage, artifact smoke, release candidate dry run, final review, commit plan, and go/no-go. |
| Auth, multi-user tenancy, BYOK, provider routing, cloud sync | Deferred platform capability | Closed by decisions until explicit decision slices open them; not required for local MVP releasability. |
| Token-saving clarity and benchmarks | Shipped local token-saver behavior | T101-T114 added local estimates, handoff packets, stale/diff context, repo maps, validation recommendations, cache visibility, savings clarity, and demos without provider billing claims. |
