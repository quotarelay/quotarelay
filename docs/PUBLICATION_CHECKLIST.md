# Public Repository Checklist

Use this before pushing public changes or cutting a public release.

## Required

- `git status --short --branch` is clean.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\assert-release-version.ps1` confirms `release.json`, package files, docs, and local package metadata agree.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\prepare-release.ps1` passes before a tag is created.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\prepare-release.ps1 -Audit` passes when registry access is available.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1` passes, including frontend tests, token-saver benchmark proof, MCP preset smoke, public-site smoke, and public-surface scan.
- `scripts\demo-local.ps1` output still proves sync, memory, exact search, repeated exact-search cache hit, handoff, backend truth count, and no provider calls.
- `npm.cmd --prefix web/controlplane audit --audit-level=moderate` reports no moderate-or-higher vulnerabilities when registry access is available.
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-surface-scan.ps1` finds no secrets, private paths, `.quotarelay` state, `.env` files, temp planning files, local package archives, or generated dependency/build directories.
- `SECURITY.md`, `SUPPORT.md`, `CONTRIBUTING.md`, `AGENTS.md`, `README.md`, `CHANGELOG.md`, `LICENSE`, and `NOTICE` are current.
- `docs/TRUTH_MATRIX.md`, `docs/KNOWN_LIMITATIONS.md`, `docs/DEPLOYMENT_READINESS.md`, and `docs/COMMERCIAL_STRATEGY.md` present the free local MVP as the adoption surface and keep hosted/commercial plans deferred.
- GitHub About description, website, and topics match `docs/GITHUB_ABOUT.md`.
- Release tag is `v0.1.0`; create it only with `scripts\tag-release.ps1 -ConfirmTag` after owner approval.
- Public docs and homepage do not claim exact provider billing savings, guaranteed savings, hosted production readiness, compliance status, or paid access gates around the local core.

## Monetization Note

Apache-2.0 permits commercial use by Quotarelay and by third parties. Current public positioning should prioritize adoption of the complete local MVP; monetization remains deferred planning rather than a near-term gate.
