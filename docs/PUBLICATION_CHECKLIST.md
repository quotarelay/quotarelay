# Public Repository Checklist

Use this before pushing public changes or cutting a public release.

## Required

- `git status --short --branch` is clean.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1` passes.
- `npm.cmd --prefix web/controlplane audit --audit-level=moderate` reports no moderate-or-higher vulnerabilities when registry access is available.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1` finds no secrets, private paths, `.quotarelay` state, `.env` files, temp planning files, local package archives, or generated dependency/build directories.
- `SECURITY.md`, `SUPPORT.md`, `CONTRIBUTING.md`, `AGENTS.md`, `README.md`, `CHANGELOG.md`, `LICENSE`, and `NOTICE` are current.
- `docs/TRUTH_MATRIX.md`, `docs/KNOWN_LIMITATIONS.md`, `docs/DEPLOYMENT_READINESS.md`, and `docs/COMMERCIAL_STRATEGY.md` separate shipped local behavior from deferred platform work and commercial plans.

## Monetization Note

Apache-2.0 permits commercial use by Quotarelay and by third parties. Monetization should come from hosted product, support, enterprise controls, integrations, policy, deployment, branding, and execution rather than source-code exclusivity.
