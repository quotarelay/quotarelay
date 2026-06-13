# Execution Tracker

This is the active local execution mirror for Quotarelay. It tracks bounded slices only; shipped behavior remains documented in `README.md`, and long-range direction remains in `docs/VISION.md`.

## Current Status

Local MVP and token-saving workflow proof are shipped for `0.1.0` release-candidate purposes.

Recently completed slices:

| Slice | Status | Evidence |
|---|---|---|
| T101-T114 token-saving clarity | Shipped | Context estimates, handoff packets, stale/diff-aware context, repo maps, validation recommendations, cache visibility, savings clarity, and local demos are reflected in `docs/MVP_SCOPE.md`. |
| Local handoff templates | Shipped | Handoff packets support `bug_fix`, `feature_slice`, `review`, `refactor`, and `release` templates without hosted sync or source upload. |
| Local savings reports | Shipped | Recent context-run history aggregates byte, approximate-token, cache, stale, and omission metadata without snippets or provider billing claims. |
| Local onboarding packs | Shipped | Repo-shape metadata, validation commands, handoff template names, and privacy notes are exposed without source dumps. |
| MCP-first productization | Shipped | `quotarelay-mcp` is the branded local MCP command; desktop and mobile app surfaces remain deferred. |
| MCP client preset verification | Shipped | Source and installed stdio presets are smoke-tested against real MCP `initialize` responses. |
| Control-plane truth polish | Shipped | The local control plane summarizes existing `/truth` tool, retrieval, cache, memory, budget, and CLI entrypoint fields without inventing readiness or provider state. |
| Open-source MVP adoption proof | Shipped | Public docs and homepage present the Apache-2.0 local MCP tool as the full adoption path while keeping hosted/commercial work deferred. |
| Adoption proof pack | Shipped | The release proof path now includes the token-saver benchmark and public-site overclaim checks. |
| First-run polish | Shipped | README, troubleshooting, MCP client setup, and preset docs now guide a fresh checkout through proof, install, and client setup cleanly. |
| Contract regression tests | Shipped | MCP tools/list now locks public tool descriptions, closed input schemas, and required-field shapes. |
| Error-message edge polish | Shipped | MCP stdio malformed tool calls now have regression coverage for explicit text responses. |
| Example quality | Shipped | The local demo now shows sync, memory, exact search cache hit, handoff, truth count, and no provider calls. |
| Release presentation | Shipped | Changelog, publication checklist, limitations, versioning, and security docs now mirror the polished proof path. |
| README and repo traction polish | Shipped | README now leads with problem, value, proof metrics, quickstart, examples, and links detailed tool/GitHub About metadata docs. |
| Public homepage product polish | Shipped | Homepage now mirrors current benchmark metrics, shows the first useful local workflow, and keeps public-site proof checks locked in smoke validation. |
| One-page stats dashboard polish | Shipped | Public UI is now a single stats-first dashboard with benchmark proof, local usage panels, and a minimal control-plane redirect page. |
| Professional operator dashboard redesign | Shipped | Root UI now has a calm SaaS-style app header, backend status banner, KPI cards, usage panels, system-health boundaries, compact savings proof, and intentional empty states. |
| Modern command center theme redesign | Shipped | Root UI now uses a command-center layout, semantic light/dark/system theme tokens, a persisted theme toggle, a four-metric rail, and calmer table/list detail sections. |
| Focused workspace simplification | Shipped | Root UI now reduces first-load density to one local status panel, one operator snapshot table, and collapsed proof/setup disclosures while keeping light/dark/system themes. |
| Dashboard action and proof chart polish | Shipped | GitHub and Quickstart now sit in a prominent action strip with icons, separate from backend/theme settings, and the local status panel includes a compact savings proof chart. |
| Unified health panel dashboard | Shipped | Root UI now uses one unified panel with a compact Backend/Repo/Context/Memory status bar, MCP tool usage pie chart, provider-call sparkline, and collapsed secondary sections. |
| Minimal operator console refinement | Shipped | The dashboard now treats the pie chart as the primary visual, removes inner boxed chart/detail treatments, and shortens health copy inside a single console surface. |
| Mobile-first SVG operator console | Shipped | The dashboard now uses mobile-first flow, an inline SVG MCP usage pie chart, a provider-call sparkline, and collapsed single-line secondary headers without expanded helper text. |
| Smarter local dashboard startup | Shipped | The control-plane dev command now starts the loopback HTTP backend when needed before launching the dashboard. |
| Product-grade dashboard run modes | Shipped | The normal dashboard command now builds and serves production assets, with separate headless and UI-dev modes. |
| Console usage snapshot | Shipped | The CLI now exposes a compact dashboard-style `usage` snapshot with status, tool usage, provider calls, and local counts. |
| Friendly root commands | Shipped | Root `npm run` aliases now expose dashboard, headless, usage, truth, sync, context, and UI build commands without long prefixes. |
| Release automation flow | Shipped | Release metadata, version assertion, release prep, local package, explicit tag, and tag-triggered validation workflow are now wired without publishing or deployment. |

## Completed Slice

### T115: Public Planning Consistency

Status: done

Goal: keep the public planning surface internally consistent now that execution planning lives under `docs/`.

Allowed files:

- `docs/EXECUTION_TRACKER.md`
- `docs/GUARDRAILS.md`
- `docs/ROADMAP.md`
- `docs/VISION.md`
- `docs/KNOWN_LIMITATIONS.md`
- other docs-only references if they point at the old tracker location
- `.github/agents/*.agent.md` tracker path references only
- `.github/instructions/*.instructions.md` tracker path references only

Non-goals:

- Do not add shipped behavior.
- Do not change MCP, CLI, HTTP, OpenAPI, or persisted-state contracts.
- Do not open auth, hosted tenancy, provider routing, BYOK, cloud sync, telemetry, deployment automation, or release publishing.
- Do not claim provider billing savings or production readiness.

Validation:

```powershell
cargo fmt --all --check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
```

Completion bar:

- Docs and local agent guidance consistently reference `docs/EXECUTION_TRACKER.md`.
- The active slice remains docs-only and does not widen product scope.
- Public-surface wording still separates shipped local behavior from planned or deferred work.

Current proof:

- `rg -n "EXECUTION_TRACKER.md|tracker slices" .github docs README.md AGENTS.md CHANGELOG.md SECURITY.md SUPPORT.md CONTRIBUTING.md` shows all live tracker references point to `docs/EXECUTION_TRACKER.md`.
- `cargo fmt --all --check` passed after adding `C:\Users\theco\.cargo\bin` to this shell's `PATH`.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1` passed: `line-count guardrail passed (max 500 lines). Checked 91 source files.`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `git diff --check` passed with line-ending warnings only.
- Docs sanity search passed: `rg -n "local MVP|Deferred platform|provider billing|docs/TRUTH_MATRIX.md|docs/LOCAL_STATE_PRIVACY.md|docs/TROUBLESHOOTING.md" README.md docs`.

Extra checks not required for T115, later covered by T116:

- Control-plane tests, control-plane build, and public-site smoke were covered by the T116 release-readiness rerun.

## Active Slice

### T116: Public Release Readiness Rerun

Status: done

Goal: rerun the local release-readiness validation path after restoring the docs execution tracker, and fix only concrete docs/test issues surfaced by validation.

Allowed files:

- `docs/EXECUTION_TRACKER.md`
- files directly surfaced by `scripts\release-check.ps1` failures, only when the failure is a docs, test, or public-surface consistency issue

Non-goals:

- Do not add shipped behavior.
- Do not change MCP, CLI, HTTP, OpenAPI, or persisted-state contracts unless release validation proves an existing contract mirror is stale.
- Do not open auth, hosted tenancy, provider routing, BYOK, cloud sync, telemetry, deployment automation, publishing, or tagging.
- Do not claim production deployment readiness or provider billing savings.

Validation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Stop condition:

- If release validation is blocked by local machine setup rather than repo content, record the exact blocker and do not change product files to work around it.
- If release validation exposes a real repo issue, fix the smallest affected file set and rerun the failing proof.

Proof:

- `npm --prefix web/controlplane ci` passed: `added 58 packages`, `found 0 vulnerabilities`.
- `powershell -NoProfile -ExecutionPolicy Bypass -Command '$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"; & "scripts\release-check.ps1"'` passed.
- Release check covered clean check, Rust package tests, control-plane tests/build, local demo smoke, local install smoke, public site smoke, public surface scan, and docs sanity.

### T117: MCP Client Preset Verification

Status: done

Goal: prove checked-in source-run and installed-command MCP presets can launch Quotarelay and answer MCP `initialize` without adding publishing, marketplace, or native-app behavior.

Allowed files:

- `scripts/mcp-preset-smoke.ps1`
- `scripts/release-check.ps1`
- `README.md`
- `docs/MCP_CLIENT_CONFIG.md`
- `examples/mcp-client-presets/README.md`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not add global install, publishing, marketplace integration, native app behavior, or deployment automation.
- Do not change MCP tool contracts, persisted state, provider boundaries, auth, or hosted behavior.
- Do not claim client-specific support beyond the generic stdio command/args presets tested here.

Validation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\mcp-preset-smoke.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Proof:

- `scripts\mcp-preset-smoke.ps1` passed with `source preset initialize`, `cargo install quotarelay-mcp`, `installed command truth`, and `installed preset initialize`.
- `scripts\release-check.ps1` includes the MCP preset smoke after local install proof.

### T118: Control-plane Truth Summary

Status: done

Goal: make the existing backend truth payload easier to scan in the local control plane while preserving the rule that UI mirrors shipped backend truth only.

Allowed files:

- `web/controlplane/src/`
- `web/controlplane/vite.config.ts`
- `docs/CONTROL_PLANE_LOCAL.md`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not add health, readiness, provider, auth, savings, telemetry, hosted, or deployment state.
- Do not change MCP, CLI, HTTP, OpenAPI, persisted-state, retrieval, memory, cache, or repository contracts.
- Do not expose local HTTP endpoints beyond loopback.

Validation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
npm.cmd --prefix web/controlplane run test -- --run
npm.cmd --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Proof:

- `scripts\check-line-counts.ps1` passed: `line-count guardrail passed (max 500 lines). Checked 94 source files.`
- Control-plane tests passed: `2 passed (2)`, `12 passed (12)`.
- Control-plane build passed with Vite.
- A direct loopback preview proof passed by starting the real backend at `127.0.0.1:3030`, starting Vite dev at `127.0.0.1:4174`, and confirming `http://127.0.0.1:4174/truth` returns backend tools and retrieval modes.
- Playwright visual verification passed against real loopback backend truth: the control plane rendered 6 truth-summary cards, 35 backend tools, no warning notes, and no summary-card overlaps.
- `scripts\release-check.ps1` passed after the slice.

## Completed Slice

### T119: Open-source MVP Adoption Proof

Status: done

Goal: make the public surface present Quotarelay as a complete, useful Apache-2.0 local MVP for adoption now, with hosted/commercial ideas clearly sidelined as deferred planning rather than the main product story.

Allowed files:

- `README.md`
- `CHANGELOG.md`
- `docs/EXECUTION_TRACKER.md`
- `docs/TRUTH_MATRIX.md`
- `docs/ROADMAP.md`
- `docs/VISION.md`
- `docs/DECISIONS.md`
- `docs/GUARDRAILS.md`
- `docs/KNOWN_LIMITATIONS.md`
- `docs/COMMERCIAL_STRATEGY.md`
- `docs/PLATFORM_SURFACES.md`
- `docs/PRIVATE_DEPLOYMENT_PLAN.md`
- `docs/PUBLICATION_CHECKLIST.md`
- `scripts/public-site-smoke.ps1`
- `web/controlplane/index.html`
- `web/controlplane/src/pages/index.tera`

Non-goals:

- Do not add shipped behavior.
- Do not change MCP, CLI, HTTP, OpenAPI, persisted-state, retrieval, memory, cache, repository, or control-plane contracts.
- Public-site smoke text assertions may change only to mirror the adoption-first homepage copy.
- Do not remove local team policy/profile features that already ship free.
- Do not open hosted login, billing, auth, provider routing, BYOK, cloud sync, telemetry, deployment automation, publishing, or tagging.
- Do not claim exact provider billing savings, production hosted readiness, or compliance posture.

Validation:

```powershell
cargo fmt --all --check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
npm.cmd --prefix web/controlplane run test -- --run
npm.cmd --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
```

Completion bar:

- Homepage and public docs lead with the free local MCP product and adoption proof.
- Hosted/commercial text is reframed as deferred future planning and does not read like a near-term gate.
- Local shipped features remain complete and useful in the free Apache-2.0 core.
- Proof commands pass or any local-machine blocker is recorded exactly.

Proof:

- `cargo fmt --all --check` passed.
- `scripts\check-line-counts.ps1` passed: `line-count guardrail passed (max 500 lines). Checked 94 source files.`
- `npm.cmd --prefix web/controlplane run test -- --run` passed: `2 passed (2)`, `12 passed (12)`.
- `npm.cmd --prefix web/controlplane run build` passed with Vite.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\public-site-smoke.ps1` passed after updating the public metadata assertions to the adoption-first homepage copy.
- `scripts\release-check.ps1` passed after the slice.

## Completed Slice

### T120: Adoption Proof Pack

Status: done

Goal: make the polished public proof path show the local product's value end to end, including token-saving benchmark evidence, install and MCP preset proof, public-site positioning checks, and release validation.

Allowed files:

- `README.md`
- `docs/EXECUTION_TRACKER.md`
- `docs/PUBLICATION_CHECKLIST.md`
- `scripts/release-check.ps1`
- `scripts/public-site-smoke.ps1`
- focused public proof scripts only if a concrete regression gap remains

Non-goals:

- Do not change MCP, CLI, HTTP, OpenAPI, persisted-state, retrieval, memory, cache, repository, or control-plane contracts.
- Do not add hosted login, billing, auth, provider routing, BYOK, cloud sync, telemetry, deployment automation, publishing, tagging, or package-manager release behavior.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, or compliance status.
- Do not add broad docs rewrites beyond the proof path and public checklist.

Validation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- `scripts\release-check.ps1` includes the local token-saver benchmark.
- Public-site smoke fails on obvious paid-gate, exact-billing-savings, guaranteed-savings, production-hosted, or compliance overclaims.
- README and publication checklist identify release-check as the main public proof path.
- Full release-check passes after the slice.

Proof:

- `scripts\token-saver-benchmark.ps1` passed, reporting exact search, overview, handoff, and diff-aware local byte/approximate-token reduction plus cache-hit and stale-state assertions.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed with public route metadata and adoption-first positioning checks.
- `scripts\check-line-counts.ps1` passed: `line-count guardrail passed (max 500 lines). Checked 94 source files.`
- `scripts\release-check.ps1` passed after adding the benchmark to the release path.

## Completed Slice

### T121: First-run Polish

Status: done

Goal: make clone, build, run, MCP setup, proof output, and troubleshooting feel clean for a new user without adding new product surfaces or changing shipped behavior.

Allowed files:

- `README.md`
- `docs/EXECUTION_TRACKER.md`
- `docs/TROUBLESHOOTING.md`
- `docs/MCP_CLIENT_CONFIG.md`
- `examples/mcp-client-presets/README.md`

Non-goals:

- Do not change MCP, CLI, HTTP, OpenAPI, persisted-state, retrieval, memory, cache, repository, or control-plane contracts.
- Do not add install automation, publishing, marketplace, deployment, hosted, auth, provider routing, telemetry, billing, desktop, or mobile behavior.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, or compliance status.
- Do not add broad marketing copy; keep first-run guidance command-oriented and honest.

Validation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- README gives a clean first-run path from bootstrap to local demo, installed command proof, MCP preset proof, and release proof.
- Troubleshooting covers common fresh-machine blockers around Rust/Cargo, Node/npm, PATH, working directory, missing sync, and local state without hiding local-first boundaries.
- MCP client docs and presets tell users exactly when to use source-run versus installed-command configuration.
- Full release-check passes after the docs polish.

Proof:

- `scripts\check-line-counts.ps1` passed: `line-count guardrail passed (max 500 lines). Checked 94 source files.`
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `git diff --check` passed with line-ending warnings only.
- `scripts\release-check.ps1` passed after the docs polish.

## Completed Slice

### T122: Contract And Regression Tests

Status: done

Goal: tighten public MCP schema, CLI output, HTTP truth, frontend assumptions, and loopback-boundary regression tests where audits find under-proved contracts.

Allowed files:

- `apps/mcp-server/src/tests/`
- `web/controlplane/src/*.test.ts`
- `docs/EXECUTION_TRACKER.md`
- small test fixtures only if a specific contract gap requires them

Non-goals:

- Do not change shipped MCP, CLI, HTTP, OpenAPI, persisted-state, retrieval, memory, cache, repository, or control-plane behavior.
- Do not add hosted login, billing, auth, provider routing, BYOK, cloud sync, telemetry, deployment automation, publishing, tagging, or package-manager release behavior.
- Do not broaden tests just for count; add coverage only for concrete public-contract gaps.

Validation:

```powershell
cargo test -p mcp-server tools_list_schemas_are_closed_and_have_stable_required_fields
cargo test -p mcp-server
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- MCP `tools/list` tests prove every public tool has a useful description, object schema, closed additional properties, and locked required-field shape.
- Any additional contract gaps found during audit either receive focused tests or stay recorded as future candidates.
- Full release-check passes after the test hardening.

Proof:

- `cargo test -p mcp-server tools_list_schemas_are_closed_and_have_stable_required_fields` passed.
- `cargo test -p mcp-server` passed: 57 tests.
- `scripts\release-check.ps1` passed after the test hardening.

## Completed Slice

### T123: Error-message And Edge-case Polish

Status: done

Goal: audit missing paths, unsynced repos, ignored files, corrupt local state, empty queries, stale cache, binary/generated files, and invalid MCP arguments; add focused tests only for real gaps.

Allowed files:

- `apps/mcp-server/src/tests/`
- `crates/context-engine/src/tests/`
- `crates/repo-index/src/tests/`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change shipped behavior unless an existing test exposes an actual bug.
- Do not loosen bounded context limits, local-state privacy, typed reasons, or public contract checks.
- Do not add hosted, billing, auth, provider routing, cloud sync, telemetry, deployment, publishing, desktop, or mobile behavior.

Validation:

```powershell
cargo test -p mcp-server malformed_tool_calls_return_explicit_text_without_panic
cargo test -p mcp-server
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- Existing edge-case coverage is audited before adding tests.
- MCP stdio malformed tool calls return explicit text responses for missing tool name, unknown tool, missing required root, and invalid retrieval mode.
- Full release-check passes after the edge-case test hardening.

Proof:

- `cargo test -p mcp-server malformed_tool_calls_return_explicit_text_without_panic` passed.
- `cargo test -p mcp-server` passed: 58 tests.
- `scripts\release-check.ps1` passed after the edge-case test hardening.

## Completed Slice

### T124: Example Quality

Status: done

Goal: polish one small local workflow that shows sync, memory, exact search, handoff, cache hit, token-saver proof, and no provider call.

Allowed files:

- `scripts/demo-local.ps1`
- `examples/demo-repo/README.md`
- `README.md`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not add new product surfaces, installers, hosted behavior, provider calls, telemetry, auth, billing, deployment, publishing, desktop, or mobile behavior.
- Do not change MCP, CLI, HTTP, OpenAPI, persisted-state, retrieval, memory, cache, repository, or control-plane contracts.
- Do not duplicate the token-saver benchmark; the demo should stay small and local.

Validation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- The local demo summary proves sync, memory, exact search, repeated exact-search cache hit, handoff packet, cache inspection, backend truth count, and no provider call.
- The fixture README explains why the tiny demo repo exists.
- Full release-check passes after the example polish.

Proof:

- `scripts\demo-local.ps1` passed and reported `first_exact_cache_status: miss`, `repeated_exact_cache_status: hit`, `handoff_template: feature_slice`, `truth_tool_count: 35`, and `provider_calls: none`.
- `scripts\release-check.ps1` passed after the example polish.

## Completed Slice

### T125: Release Presentation

Status: done

Goal: polish changelog, publication checklist, limitations, security wording, and release narrative after proof and first-run gaps are closed.

Allowed files:

- `CHANGELOG.md`
- `docs/PUBLICATION_CHECKLIST.md`
- `docs/KNOWN_LIMITATIONS.md`
- `docs/VERSIONING.md`
- `SECURITY.md`
- `SUPPORT.md`
- `CONTRIBUTING.md`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change shipped behavior, tests, scripts, product surfaces, or release automation.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.
- Do not reopen private deployment, commercial packaging, hosted, desktop, or mobile work.

Validation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- Release-facing docs match the now-polished proof path and first-run story.
- Known limitations and security/support wording remain explicit about local-only boundaries.
- Full release-check passes after the release presentation polish.

Proof:

- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `git diff --check` passed with line-ending warnings only.
- `scripts\release-check.ps1` passed after the release presentation polish.

## Completed Slice

### T126: README And Repo Traction Polish

Status: done

Goal: make the repository front door easier to understand in the first minute by shortening the README, adding honest local proof metrics, moving long reference material into docs, and preserving local-first boundaries.

Allowed files:

- `README.md`
- `docs/MCP_TOOL_REFERENCE.md`
- `docs/GITHUB_ABOUT.md`
- `docs/PUBLICATION_CHECKLIST.md`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change shipped behavior, tests, scripts, product surfaces, proof commands, release automation, or GitHub repository settings.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.
- Do not add marketing claims that are not backed by local scripts or shipped docs.

Validation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- README leads with problem, value, proof metrics, quickstart, and examples instead of a long tool inventory.
- Detailed MCP tool and CLI reference lives in docs.
- GitHub About copy/topics are captured in docs for repository settings.
- Full release-check passes after the documentation polish.

Proof:

- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `git diff --check` passed with line-ending warnings only.
- `scripts\release-check.ps1` passed after the README and repo traction polish.

## Completed Slice

### T127: Public Homepage Product Polish

Status: done

Goal: make the public homepage feel more concrete and adoption-ready by aligning visible proof metrics with the current benchmark and showing a first useful local workflow without widening shipped scope.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/styles.css`
- `web/controlplane/src/public-site.css`
- `scripts/public-site-smoke.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, or repository settings.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.
- Do not add paid-version, hosted, desktop, mobile, telemetry, provider-call, or deployment positioning.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- Homepage proof metrics match the current token-saver benchmark snapshot.
- First useful local workflow is visible without implying hosted, paid, provider, or deployment behavior.
- Public homepage CSS stays under the line-count guardrail after splitting public-site styles.
- Full release-check passes after the homepage polish.

Proof:

- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed and now checks the proof/workflow copy.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the homepage polish.
- In-app browser visual QA was attempted, but the local Browser connection failed in this Windows sandbox with a setup refresh error; the local preview server responded with HTTP 200 before it was stopped.

## Completed Slice

### T128: One-page Stats Dashboard Polish

Status: done

Goal: replace the chatter-heavy public/control-plane split with one cleaner stats-first dashboard that shows shipped proof, local usage state when configured, benchmark reductions, and a short workflow.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/pages/control-plane.tera`
- `web/controlplane/src/public-site.css`
- `scripts/public-site-smoke.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, or repository settings.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.
- Do not add paid-version, hosted, desktop, mobile, telemetry, provider-call, or deployment positioning.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- `/` is a single stats-first dashboard, not a long marketing/planning page.
- Visible AI/planning-style page chatter is removed from the product UI.
- `/control-plane` no longer presents a second internal-agent dashboard as the main experience.
- Public-site smoke checks the new dashboard stats copy.
- Full release-check passes after the dashboard polish.

Proof:

- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed and checks the stats-dashboard copy.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the one-page dashboard polish.
- Local preview responded with HTTP 200; in-app browser visual QA was attempted but still failed in this Windows sandbox with the setup refresh error.

## Completed Slice

### T129: Professional Operator Dashboard Redesign

Status: done

Goal: turn the one-page UI into a professional calm SaaS-style local operator dashboard with system health, usage, and savings proof prioritized above setup/documentation content.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/pages/control-plane.tera`
- `web/controlplane/src/public-site.css`
- `scripts/public-site-smoke.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, or repository settings.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.
- Do not add paid-version, hosted, desktop, mobile, telemetry, provider-call, or deployment positioning.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- Above the fold reads as an operator dashboard with app header, status pill/banner, and KPI cards.
- Backend offline state is small and actionable rather than a giant error card.
- Empty states avoid escaped placeholders and read like intentional operator guidance.
- Dashboard sections are System health, Usage, and Savings proof, with setup reduced to a compact panel.
- Full release-check passes after the professional dashboard redesign.

Proof:

- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed and checks the new dashboard copy.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the professional operator dashboard redesign.
- Local preview responded with HTTP 200; in-app browser visual QA was attempted but still failed in this Windows sandbox with the setup refresh error.

## Completed Slice

### T130: Modern Command Center Theme Redesign

Status: done

Goal: replace the remaining card-heavy operator UI with a cleaner command-center dashboard, reduce first-screen clutter, and add persisted System/Light/Dark theme support through semantic CSS tokens.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/pages/control-plane.tera`
- `web/controlplane/src/public-site.css`
- `scripts/public-site-smoke.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, or repository settings.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.
- Do not add paid-version, hosted, desktop, mobile, telemetry, provider-call, or deployment positioning.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- First screen uses one dominant local context status panel plus a four-item metric rail, not a cluster of equal cards.
- Dashboard details use table/list rows and compact panels instead of repeated boxed cards.
- Theme control exposes System, Light, and Dark options and persists explicit choices in `localStorage`.
- CSS uses semantic light/dark tokens with system preference fallback.
- Full release-check passes after the command-center theme redesign.

Proof:

- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed and checks command-center/theme copy.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the command-center theme redesign.
- Local preview responded with HTTP 200; in-app browser visual QA was attempted but still failed in this Windows sandbox with the setup refresh error.

## Completed Slice

### T131: Focused Workspace Simplification

Status: done

Goal: respond to the card-heavy dashboard critique by using dashboard hierarchy and progressive disclosure patterns: make one status panel primary, reduce visible metrics, move proof/setup/boundary detail into disclosures, and keep light/dark/system support.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/pages/control-plane.tera`
- `web/controlplane/src/public-site.css`
- `scripts/public-site-smoke.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, or repository settings.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.
- Do not add paid-version, hosted, desktop, mobile, telemetry, provider-call, or deployment positioning.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- First screen shows one local context status panel plus a compact signal list, not a cluster of cards.
- Usage is a single operator snapshot table with short rows and subdued empty states.
- Savings proof, boundaries, and setup are collapsed details instead of always-visible panels.
- Light, Dark, and System theme behavior remains tokenized and persisted.
- Full release-check passes after the focused workspace simplification.

Proof:

- UX research reviewed Carbon dashboard hierarchy/clutter guidance, Apple dark-mode surface guidance, Material empty-state and navigation guidance, and Fluent modern app-shell/dashboard guidance.
- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed and checks the simplified workspace copy.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the focused workspace simplification.
- Local preview responded with HTTP 200; in-app browser visual QA was attempted but still failed in this Windows sandbox with the setup refresh error.

## Completed Slice

### T132: Dashboard Action and Proof Chart Polish

Status: done

Goal: separate product actions from settings, make GitHub and Quickstart more prominent with icons, and add restrained charting without returning to a dense card dashboard.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/public-site.css`
- `scripts/public-site-smoke.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, repository settings, or product scope.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- GitHub and Quickstart are no longer grouped with backend status and theme settings.
- Primary action buttons include icons and remain prominent in light and dark themes.
- The status panel includes a compact savings proof chart for the benchmark reductions.
- Smoke validation checks the action strip and chart copy.
- Full release-check passes after the action/chart polish.

Proof:

- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed and checks `Start here` plus `Savings proof chart`.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the action/chart polish.
- Local preview responded with HTTP 200; in-app browser visual QA was attempted but still failed in this Windows sandbox with the setup refresh error.

## Completed Slice

### T133: Unified Health Panel Dashboard

Status: done

Goal: replace the remaining multi-section dashboard feel with a single unified panel that communicates health at a glance through one compact status bar, a tool usage pie chart, a provider-call sparkline, and collapsed secondary sections.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/public-site.css`
- `scripts/public-site-smoke.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, repository settings, or product scope.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- Root UI is one unified dashboard panel rather than separate card clusters.
- Compact status bar reads `Backend: ... | Repo: ... | Context: ... | Memory: ...`.
- MCP tool distribution is visualized as a pie chart using live `/truth` tool names with release-proof fallback groups.
- Provider calls are visualized as a zero-call sparkline consistent with the local-only product boundary.
- Recent runs, memory matches, cache, and boundaries are collapsed by default.
- Full release-check passes after the unified health panel redesign.

Proof:

- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed and checks compact status tokens plus chart labels.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the unified health panel redesign.
- Local preview responded with HTTP 200; in-app browser visual QA was attempted but still failed in this Windows sandbox with the setup refresh error.

## Completed Slice

### T134: Minimal Operator Console Refinement

Status: done

Goal: tighten the unified dashboard into a stricter minimal operator console by making the pie chart the primary visual, removing inner card treatments, and shortening explanatory health text.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/public-site.css`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, repository settings, or product scope.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- Pie chart is the dominant first visual in the console.
- Chart and disclosure areas use internal separators rather than nested card boxes.
- Health copy is short enough to scan without explanation.
- Full release-check passes after the minimal operator console refinement.

Proof:

- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the minimal operator console refinement.
- Local preview responded with HTTP 200; in-app browser visual QA was attempted but still failed in this Windows sandbox with the setup refresh error.

## Completed Slice

### T135: Mobile-First SVG Operator Console

Status: done

Goal: implement the strict mobile-first operator console plan with one status line, a reliable inline SVG pie chart, provider-call sparkline, and collapsed single-line secondary sections.

Allowed files:

- `web/controlplane/src/pages/index.tera`
- `web/controlplane/src/public-site.css`
- `scripts/public-site-smoke.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend behavior, MCP/CLI/HTTP contracts, proof scripts, release automation, repository settings, or product scope.
- Do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, auth, cloud sync, deployment support, or provider integrations.

Validation:

```powershell
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- Mobile-first vertical flow shows header utilities, status line, SVG pie, sparkline, and collapsed headers.
- MCP usage pie is rendered as inline SVG slices rather than CSS conic background.
- No expanded recent-run, memory, cache, or boundary helper text appears by default.
- Smoke validation no longer requires removed setup/detail copy.
- Full release-check passes after the mobile-first SVG console redesign.

Proof:

- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed and checks compact status tokens plus chart labels.
- `scripts\check-line-counts.ps1` passed with 95 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the mobile-first SVG console redesign.
- Local preview responded with HTTP 200; in-app browser visual QA was attempted but still failed in this Windows sandbox with a permission error.

## Completed Slice

### T140: Release Automation Flow

Status: done

Goal: set the release version/tag flags and add a safe automated release-prep flow that validates versions, runs release checks, builds local artifacts, and supports explicit maintainer tag creation without publishing or deployment.

Allowed files:

- `release.json`
- `package.json`
- `.github/workflows/release.yml`
- `scripts/assert-release-version.ps1`
- `scripts/prepare-release.ps1`
- `scripts/tag-release.ps1`
- `scripts/release-check.ps1`
- `docs/RELEASE_FLOW.md`
- `docs/VERSIONING.md`
- `docs/PUBLICATION_CHECKLIST.md`
- `docs/KNOWN_LIMITATIONS.md`
- `README.md`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not publish a GitHub Release, push tags, deploy services, expose non-loopback HTTP, add hosted state, add telemetry, add provider calls, add package-manager publishing, or claim production hosted readiness.

Validation:

```powershell
npm run release:version
npm run release:prep -- -SkipReleaseCheck
npm run release:tag -- -Help
npm run ui:test
npm run ui:build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- `release.json` defines version `0.1.0`, tag `v0.1.0`, draft/prerelease/local-only/publish flags, and GitHub topics.
- `scripts\assert-release-version.ps1` verifies release metadata against Cargo, package, README, changelog, versioning docs, and local package metadata.
- `scripts\prepare-release.ps1` runs version assertion, release-check, optional audit, optional local package build, and prints the owner-approved tag/push commands.
- `scripts\tag-release.ps1` creates only a local annotated tag after explicit `-ConfirmTag`, clean worktree, version assertion, and release-check.
- `.github/workflows/release.yml` validates `v*` tags and workflow-dispatch release candidates, then uploads a local package artifact without publishing a GitHub Release.
- Full release-check passes after the release automation flow.

Proof:

- `npm run release:version` passed and confirmed version `0.1.0`, tag `v0.1.0`, draft `true`, prerelease `false`, local-only `true`, and publish `false`.
- `npm run release:prep -- -SkipReleaseCheck` passed and printed the owner-approved local tag and push commands.
- `npm run release:tag -- -Help` passed without creating a tag.
- `npm run ui:test` passed with 12 tests.
- `npm run ui:build` passed.
- `scripts\check-line-counts.ps1` passed with 101 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed with the new release version manifest step.
- `npm run release:package -- -SkipReleaseCheck` passed and created `target\local-package\quotarelay-0.1.0-local.zip`.

## Completed Slice

### T139: Friendly Root Commands

Status: done

Goal: replace long first-run command prefixes with short root `npm run` aliases while keeping the underlying Cargo and control-plane commands intact.

Allowed files:

- `package.json`
- `README.md`
- `docs/CONTROL_PLANE_LOCAL.md`
- `docs/MCP_CLIENT_CONFIG.md`
- `docs/MCP_TOOL_REFERENCE.md`
- `docs/PLATFORM_SURFACES.md`
- `docs/TROUBLESHOOTING.md`
- `scripts/bootstrap.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not change backend contracts, MCP schemas, HTTP routes, dashboard behavior, provider boundaries, hosted state, telemetry, auth, billing, cloud sync, or release automation.

Validation:

```powershell
npm run truth
npm run usage
npm run start -- --help
npm run ui:build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- `npm run start`, `npm run headless`, `npm run usage`, `npm run truth`, `npm run register`, `npm run sync`, `npm run search`, `npm run context`, `npm run handoff`, and `npm run state` are available from the repo root.
- README quickstart and repo usage examples use the friendly aliases.
- Control-plane docs use the friendly aliases for dashboard, headless, usage, build, and dev paths.
- Bootstrap next-command output points users at `npm run truth`, `npm run usage`, and `npm run start`.
- Full release-check passes after the alias polish.

Proof:

- `npm run truth` passed and returned backend truth.
- `npm run usage` passed and returned the console usage snapshot.
- `npm run start -- --help` passed and shows the dashboard/headless/dev modes.
- `npm run ui:build` passed.
- `npm run ui:test` passed with 12 tests.
- `scripts\check-line-counts.ps1` passed with 98 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the friendly root command aliases.

## Completed Slice

### T138: Console Usage Snapshot

Status: done

Goal: give command-line users a compact operator snapshot similar to the dashboard, not just a help menu.

Allowed files:

- `apps/mcp-server/src/cli.rs`
- `apps/mcp-server/src/cli_usage.rs`
- `apps/mcp-server/src/lib.rs`
- `apps/mcp-server/src/truth.rs`
- `apps/mcp-server/src/tests.rs`
- `apps/mcp-server/src/tests/cli_usage.rs`
- `README.md`
- `docs/CONTROL_PLANE_LOCAL.md`
- `scripts/local-package.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not add provider telemetry, remote analytics, hosted state, auth, billing, cloud sync, or non-loopback HTTP behavior.
- Do not turn the CLI output into an unstructured terminal UI that breaks scripts.

Validation:

```powershell
cargo fmt --all --check
cargo test -p mcp-server local_cli_usage_returns_console_snapshot
cargo test -p mcp-server
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- `--cli usage` returns status, MCP tool usage distribution, provider-call state/sparkline, system health, usage counts, and collapsed section summaries.
- `--cli usage [repo-root] [memory-query]` can include local run and memory counts when local state is present.
- Backend truth lists the console usage command.
- README and local control-plane docs expose the console snapshot path.
- Full release-check passes after the console usage snapshot.

Proof:

- `cargo fmt --all --check` passed.
- `cargo test -p mcp-server local_cli_usage_returns_console_snapshot` passed.
- `cargo test -p mcp-server` passed with 59 tests.
- `cargo run -p mcp-server -- --cli usage` returned status, tool usage, provider-call sparkline, system health, collapsed rows, and usage counts.
- `scripts\check-line-counts.ps1` passed with 98 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the console usage snapshot.

## Completed Slice

### T137: Product-Grade Dashboard Run Modes

Status: done

Goal: make the dashboard easy to run as a built local product, keep headless backend operation explicit, and reserve the dev server for UI customization work.

Allowed files:

- `web/controlplane/dev-with-backend.mjs`
- `web/controlplane/package.json`
- `docs/CONTROL_PLANE_LOCAL.md`
- `docs/PLATFORM_SURFACES.md`
- `scripts/local-package.ps1`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not add browser-started local processes.
- Do not expose HTTP endpoints outside loopback.
- Do not add hosted state, telemetry, auth, provider calls, deployment automation, plugin customization, or background services.

Validation:

```powershell
node --check web\controlplane\dev-with-backend.mjs
npm --prefix web/controlplane run start -- --help
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- `npm --prefix web/controlplane run start` builds the dashboard and serves the production assets through local preview.
- `npm --prefix web/controlplane run headless` starts the loopback HTTP backend without UI.
- `npm --prefix web/controlplane run dev` remains available for UI contributors.
- Local control-plane docs describe the normal, headless, dev, and source-customization paths.
- Full release-check passes after the run-mode polish.

Proof:

- `node --check web\controlplane\dev-with-backend.mjs` passed.
- `npm --prefix web/controlplane run start -- --help` passed and documents `--headless` plus `--dev`.
- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed.
- `scripts\check-line-counts.ps1` passed with 98 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the product-grade dashboard run modes.

## Completed Slice

### T136: Smarter Local Dashboard Startup

Status: done

Goal: make the local dashboard dev command start the loopback HTTP backend when it is missing, while keeping the browser read-only and preserving local-only boundaries.

Allowed files:

- `web/controlplane/dev-with-backend.mjs`
- `web/controlplane/package.json`
- `docs/CONTROL_PLANE_LOCAL.md`
- `docs/EXECUTION_TRACKER.md`

Non-goals:

- Do not let the browser start local processes.
- Do not expose HTTP endpoints outside loopback.
- Do not add hosted state, telemetry, auth, provider calls, deployment automation, or background services.

Validation:

```powershell
node --check web\controlplane\dev-with-backend.mjs
npm --prefix web/controlplane run test
npm --prefix web/controlplane run build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1 -SkipBuild
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

Completion bar:

- `npm --prefix web/controlplane run dev` checks `/truth`, starts `mcp-server --http 127.0.0.1:3030` if needed, then starts Vite on loopback.
- `npm --prefix web/controlplane run dev:frontend` remains available for frontend-only preview.
- Local control-plane docs explain the one-command preview path and the separate backend/frontend options.
- Full release-check passes after the dev-startup polish.

Proof:

- `node --check web\controlplane\dev-with-backend.mjs` passed.
- `npm --prefix web/controlplane run test` passed with 12 tests.
- `npm --prefix web/controlplane run build` passed.
- `scripts\public-site-smoke.ps1 -SkipBuild` passed.
- `scripts\check-line-counts.ps1` passed with 96 checked source files.
- `scripts\public-surface-scan.ps1` passed with `{ "ok": true }`.
- `scripts\release-check.ps1` passed after the smarter local dashboard startup change.

## Active Slice

No active slice. Console usage snapshot and product-grade dashboard run modes are complete.

## Polish Plan Extension

| Candidate | Boundary |
|---|---|
| T121 First-run polish | Make clone, build, run, MCP setup, proof output, and troubleshooting feel clean for a new user without adding new product surfaces. |
| T122 Contract and regression tests | Tighten MCP schema, CLI output, HTTP truth, frontend assumptions, and loopback-boundary tests where audits find under-proved contracts. |
| T123 Error-message and edge-case polish | Audit missing paths, unsynced repos, ignored files, corrupt local state, empty queries, stale cache, binary/generated files, and invalid MCP arguments; add focused tests only for real gaps. |
| T124 Example quality | Polish one small local workflow that shows sync, memory, exact search, handoff, cache hit, token-saver proof, and no provider call. |
| T125 Release presentation | Polish changelog, publication checklist, limitations, security wording, and product narrative after proof and first-run gaps are closed. |
| Private deployment decisions | Deferred planning only until the polished open-source local adoption proof is strong and the user explicitly reopens deployment planning. |

## Closed Until Explicitly Opened

- Provider forwarding or provider key storage.
- Hosted multi-user tenancy.
- Cloud sync or hosted `.quotarelay` state.
- Auth, operator identity, audit logs, enterprise SSO, or BYOK.
- Deployment automation, publishing, release tagging, or production container shipping.
