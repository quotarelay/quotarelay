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

## Next Candidate Slices

| Candidate | Boundary |
|---|---|
| Private deployment decisions | Planning only until threat model, signing, provenance, auth, storage, backup, rollout, and rollback decisions close. |

## Closed Until Explicitly Opened

- Provider forwarding or provider key storage.
- Hosted multi-user tenancy.
- Cloud sync or hosted `.quotarelay` state.
- Auth, operator identity, audit logs, enterprise SSO, or BYOK.
- Deployment automation, publishing, release tagging, or production container shipping.
