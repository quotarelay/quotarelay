# Quotarelay Truth Matrix

This matrix separates what is shipped, useful next work, and deferred platform decisions. It is a planning guardrail, not a marketing page.

## Shipped Local Behavior

| Area | Current truth |
|---|---|
| MCP server | Stdio MCP server exposes shipped tools for repo sync/search/map, context assembly, memory, cache, repository registration, workspace profiles, run history, and local state inspection. |
| CLI | Non-interactive local CLI returns stable JSON for `truth`, `register`, `sync`, `state`, `search`, and `assemble` modes. |
| Retrieval | `exact_search`, `overview`, `task_capsule`, and `diff_aware` return bounded context with inclusion and omission reasons. |
| Repository index | Local sync writes a bounded `.quotarelay/index.json`, respects explicit ignore config, and supports inventory/search/document lookup. |
| Memory | Durable local memory supports write/read/update/delete/search/export/import under `.quotarelay`. Empty memory search is rejected instead of dumping notes. |
| Cache | Local retrieval caches can be inspected, cleared, reused, invalidated by explicit sync, and recovered from corrupt JSON by operator action. |
| Token-saving clarity | Context and handoff packets expose local raw bytes considered, included bytes, approximate tokens, reduction ratio, cache status, stale status, omissions, and one-command demo output without provider billing claims. |
| Handoff templates | Local handoff packets can include bug fix, feature slice, review, refactor, or release focus guidance without hosted sync or source upload. |
| Savings reports | Local context-run history can be aggregated into byte, approximate-token, cache, stale, and omission metadata without snippets, telemetry, or provider billing claims. |
| Onboarding packs | Local MCP onboarding packs summarize repo-shape metadata, validation commands, handoff templates, and privacy notes without source snippets or upload. |
| Team policy profiles | Local `.quotarelay` policy profiles can store guardrails, validation recipes, MCP client presets, and explicit source-upload preference without hosted sync. |
| Truth surfaces | HTTP `/truth`, CLI `truth`, OpenAPI, and control-plane truth rendering mirror shipped backend capability only. |
| Guardrails | Source files are expected to stay under 500 lines unless generated or explicitly exempted. |
| Public guidance | `AGENTS.md`, `CONTRIBUTING.md`, `SUPPORT.md`, `SECURITY.md`, versioning docs, deployment-readiness docs, `LICENSE`, and `NOTICE` define the public operating surface. |
| Commercial posture | Apache-2.0 local core stays useful; paid value is planned around hosted/team/enterprise coordination, governance, integrations, support, and deployment. |

## Next Useful Work

| Area | Next useful work |
|---|---|
| Public launch | Keep README, SECURITY, support, agent guidance, public homepage, release notes, and known limitations aligned with shipped local behavior. |
| Proof suite | Keep one local proof path covering register, sync, memory, assemble, cache hit visibility, savings estimates, CLI, HTTP truth, and control-plane rendering. |
| Hosted/team planning | Keep paid surfaces focused on coordination, shared policy, aggregate savings reporting, auditability, integrations, support, and deployment management. |
| Deployment readiness | Close private deployment decisions, signing/provenance choices, rollout/rollback expectations, and container proof tests before shipping customer-managed deployment behavior. |

## Deferred Platform Decisions

| Area | Current decision |
|---|---|
| Auth | Closed until an explicit decision slice opens local control-plane protection. |
| Multi-user tenancy | Closed for MVP; future work must split local operator identity from hosted tenancy. |
| Provider routing | Closed; Quotarelay is not a provider gateway. |
| BYOK | Decision only until storage rules are approved; no provider calls or forwarding. |
| Cloud sync | Closed for MVP; local-first state remains the default. |
| Enterprise packaging | Future work may add private deployment guidance, auditability, and support paths after local MVP release readiness. |
| Hosted deployment | Deferred until auth, TLS, identity, audit, state, rollout, rollback, and monitoring controls are designed and implemented. |
| Hosted team control plane | Deferred until organization login, roles, policy storage, shared memory workflow, audit events, and privacy boundaries are implemented. |

## Explicit Non-Goals

| Non-goal | Boundary |
|---|---|
| Fake savings | Do not claim exact provider billing savings from local byte or approximate token estimates. |
| Fake readiness | Do not render health, progress, or readiness state that the backend does not expose. |
| Unbounded context | Do not dump full repos, memory stores, cache files, or histories into agents. |
| Provider forwarding | Do not route model calls or store provider keys unless decision docs and tracker slices explicitly open that work. |
| Hidden automation | Do not silently refresh, clear, sync, or repair local state without operator action. |
| Monolith drift | Keep adapters thin and business rules in engine/service modules under the line-count guardrail. |
