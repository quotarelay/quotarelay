# Changelog

## 0.1.0 - Local MVP release candidate

This release candidate documents the current local MVP behavior. It is not a hosted service, deployment artifact, provider gateway, or billing-savings claim.

Public-readiness update: the local core is Apache-2.0, with public guidance for humans and AI agents plus an open-core commercial strategy that keeps local functionality free while reserving hosted/team/enterprise coordination, governance, support, and deployment as paid product surfaces.

Team-layer planning update: local team policy profiles now store guardrails, validation recipes, MCP presets, and explicit source-upload preference under `.quotarelay`, with MCP save/list tools and hosted control-plane security/privacy gates documented before public login work proceeds.

Team workflow update: handoff packets now support local templates for bug fixes, feature slices, reviews, refactors, and releases, adding bounded workflow focus without hosted sync or source upload.

Savings proof update: local savings reports now aggregate recent context-run history into bounded byte, approximate-token, cache, stale, and omission metadata without uploading snippets or claiming provider billing savings.

Onboarding update: local MCP onboarding packs now expose bounded repo-shape metadata, validation commands, handoff template names, and privacy notes without dumping source.

Productization update: Quotarelay now has an agent-tool-first install surface with a branded `quotarelay-mcp` local MCP command, a source/Cargo install smoke, and public wording that defers desktop and mobile apps.

### Shipped local behavior

- Stdio MCP server with tools for repository sync/search, bounded context assembly, durable memory, cache inspection/clear, repository registration, workspace profiles, team policy profiles, context run history, and local state inspection.
- Local CLI JSON workflows for `truth`, `register`, `sync`, `state`, `search`, `assemble`, and templated handoff packets.
- Bounded retrieval modes: `exact_search`, `overview`, and `task_capsule`.
- Explicit packing limits: 5 context items, 160 snippet bytes, 640 document bytes, 320 memory-note bytes, and 10 history runs.
- Typed inclusion and omission reasons across engine, MCP, CLI, and context history boundaries.
- Local repository index, memory, cache, context history, registered repositories, workspace profiles, and team policy profiles stored under `.quotarelay/`.
- Optional `.quotarelay/ignore.json` support for exact paths and prefixes applied on explicit sync.
- Thin HTTP routes for local control-plane use: `GET /truth`, `GET /repositories`, `GET /memory`, and `GET /context-runs`.
- Control plane that renders backend truth without inventing readiness, live health, provider state, or savings metrics.
- Local validation scripts for bootstrap, clean checks, release checks, demo smoke, and source line-count guardrails.
- Local savings reports generated from context-run history metadata only.
- Local onboarding packs generated from repository index metadata only.
- Branded local MCP command `quotarelay-mcp` for installed stdio agent workflows.

### Validation commands

Primary local release checks:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\clean-check.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-smoke.ps1
```

Focused checks used by the current release-readiness track:

```powershell
cargo test -p mcp-server backend_truth_endpoint_exposes_current_contract
cargo test -p mcp-server repository_state_tool_reports_sync_and_recent_run_truth_over_stdio
cargo test -p mcp-server repository_registration_tools_work_over_stdio
cargo test -p context-engine registered_repository_state_reports_sync_and_recent_run_truth
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
```

### Known limitations

- Local MVP only; no hosted control plane, cloud sync, or multi-user tenancy is implemented.
- No authentication or access control is implemented for local HTTP endpoints.
- No encryption-at-rest is claimed for `.quotarelay/` state.
- No provider forwarding, provider key handling, model gateway behavior, or model-provider calls are implemented.
- No telemetry, remote feedback collection, or upload of repository contents is implemented.
- Context reduction is bounded and explainable locally, but this release does not claim exact provider tokenization, billing savings, or pricing impact.
- Repository indexing is local and bounded; broad multi-language semantic analysis, vector search, filesystem watchers, and automatic background sync are not part of this release.
- The control plane renders existing backend truth only and must show unavailable or not-configured states when the backend cannot be reached.

### Deferred platform work

The following remain deferred until explicit future decision slices open them:

- Auth and local control-plane protection.
- Hosted multi-user tenancy.
- Cloud sync or hosted repository state.
- BYOK storage or provider key management.
- Provider request routing.
- Enterprise packaging, SSO, audit, and compliance surfaces.

### Local operator docs

- `README.md` has the quickstart and shipped command surface.
- `docs/MCP_CLIENT_CONFIG.md` has stdio MCP client setup guidance.
- `docs/CONTROL_PLANE_LOCAL.md` has local backend and frontend commands.
- `docs/TROUBLESHOOTING.md` has recovery guidance for local JSON state, cache issues, missing index state, and path problems.
- `docs/LOCAL_STATE_PRIVACY.md` describes what `.quotarelay/` stores and what the local MVP does not send.
- `docs/KNOWN_LIMITATIONS.md` lists current local MVP limitations.
- `docs/TRUTH_MATRIX.md` separates shipped behavior, queued local work, deferred platform decisions, and explicit non-goals.
