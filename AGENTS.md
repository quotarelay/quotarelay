# AI Agent Guide

This repository is public, but the shipped surface is intentionally local-first and bounded. AI agents working here should optimize for honesty, small changes, and public-surface safety.

## Start Here

- Read `README.md`, `SECURITY.md`, `docs/TRUTH_MATRIX.md`, and `docs/GUARDRAILS.md` before changing behavior.
- Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1` before claiming release readiness.
- Keep generated output, local state, dependency folders, and `.quotarelay/` out of commits.

## Safe Change Rules

- Keep adapter code thin. MCP, HTTP, and CLI layers should parse input, call engine crates, serialize output, and map errors.
- Keep persistence, retrieval, cache, memory, repository registration, and validation rules in owning crates.
- Do not add provider forwarding, provider keys, telemetry, cloud sync, auth, multi-user behavior, deployment automation, or billing claims unless decision docs and tracker truth explicitly open that work.
- Do not expose local HTTP endpoints on non-loopback interfaces.
- Do not weaken bounded context limits or typed inclusion/omission reasons.
- Keep source files at 500 lines or fewer unless generated or explicitly exempted.

## Required Checks

Use focused checks while developing, then run the full release check before public-facing changes:

```powershell
cargo fmt --all --check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

## Public-Surface Review

Before committing public-facing changes, scan for secrets, tokens, API keys, private paths, local state, `.env` content, stale shipped/deferred claims, and machine-specific examples.
