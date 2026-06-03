# Quotarelay Truth Matrix

This matrix separates what is shipped from what is queued or deferred. It is a planning guardrail, not a marketing page.

## Shipped Local Behavior

| Area | Current truth |
|---|---|
| MCP server | Stdio MCP server exposes shipped tools for repo sync/search, context assembly, memory, cache, repository registration, workspace profiles, run history, and local state inspection. |
| CLI | Non-interactive local CLI returns stable JSON for `truth`, `register`, `sync`, `state`, `search`, and `assemble` modes. |
| Retrieval | `exact_search`, `overview`, and `task_capsule` return bounded context with inclusion and omission reasons. |
| Repository index | Local sync writes a bounded `.quotarelay/index.json`, respects explicit ignore config, and supports inventory/search/document lookup. |
| Memory | Durable local memory supports write/read/update/delete/search/export/import under `.quotarelay`. Empty memory search is rejected instead of dumping notes. |
| Cache | Local retrieval caches can be inspected, cleared, reused, invalidated by explicit sync, and recovered from corrupt JSON by operator action. |
| Truth surfaces | HTTP `/truth`, CLI `truth`, OpenAPI, and control-plane truth rendering mirror shipped backend capability only. |
| Guardrails | Source files are expected to stay under 500 lines unless generated or explicitly exempted. |

## Queued Local Work

| Area | Next useful work |
|---|---|
| Docs and release readiness | Bootstrap docs, MCP client setup, local quickstart, troubleshooting, known limitations, changelog, and release rehearsal. |
| Token-saving clarity | Add cache hit/miss visibility, stale state, and local reduction benchmarks without provider billing claims; local byte counts and approximate token estimates now ship for context packets. |
| Agent handoff | Produce bounded handoff packets with active task, relevant context, decision memory, validation commands, blockers, and omission reasons. |
| Repo intelligence | Add bounded repo maps, diff-aware context, and validation recommendations using local repo state only. |
| Demo and validation | Provide a local example workspace, demo script, and one-command validation suite with no external services. |

## Deferred Platform Decisions

| Area | Current decision |
|---|---|
| Auth | Closed until an explicit decision slice opens local control-plane protection. |
| Multi-user tenancy | Closed for MVP; future work must split local operator identity from hosted tenancy. |
| Provider routing | Closed; Quotarelay is not a provider gateway. |
| BYOK | Decision only until storage rules are approved; no provider calls or forwarding. |
| Cloud sync | Closed for MVP; local-first state remains the default. |
| Enterprise packaging | Future work may add private deployment guidance, auditability, and support paths after local MVP release readiness. |

## Explicit Non-Goals

| Non-goal | Boundary |
|---|---|
| Fake savings | Do not claim exact provider billing savings from local byte or approximate token estimates. |
| Fake readiness | Do not render health, progress, or readiness state that the backend does not expose. |
| Unbounded context | Do not dump full repos, memory stores, cache files, or histories into agents. |
| Provider forwarding | Do not route model calls or store provider keys unless decision docs and tracker slices explicitly open that work. |
| Hidden automation | Do not silently refresh, clear, sync, or repair local state without operator action. |
| Monolith drift | Keep adapters thin and business rules in engine/service modules under the line-count guardrail. |
