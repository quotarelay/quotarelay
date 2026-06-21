# Versioning and Release Policy

Quotarelay currently uses `0.x` versioning while the public API and operational shape are still stabilizing.

## Current Version

- Workspace crates: `0.1.0`
- Control plane package: `0.1.0`
- Release tag: `v0.1.0`
- Release state: local MVP release-ready
- License: Apache-2.0

## Compatibility Expectations

Until `1.0.0`:

- minor versions may change local CLI, MCP tool, HTTP, or persisted-state contracts when release notes call it out,
- patch versions should be bug fixes, documentation corrections, or validation improvements,
- migrations must be documented before persisted `.quotarelay` formats change incompatibly.

## Release Checklist

Before a public version tag:

1. Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\assert-release-version.ps1`.
2. Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\prepare-release.ps1`.
3. Confirm release-check covers the local demo, token-saver benchmark, install smoke, MCP preset smoke, public-site smoke, public-surface scan, and docs sanity.
3. Run public-surface scans for secrets, private paths, local state, and stale shipped/deferred claims.
4. Confirm `SECURITY.md`, `README.md`, `CHANGELOG.md`, `docs/TRUTH_MATRIX.md`, and `docs/KNOWN_LIMITATIONS.md` match shipped behavior.
5. Confirm OpenAPI mirrors shipped HTTP routes.
6. Confirm `release.json`, `Cargo.toml`, `package.json`, `web/controlplane/package.json`, and release notes use the same version.
7. Confirm release artifacts include `LICENSE`, `NOTICE`, and deployment-readiness docs.

## Tagging

Release validation is automated through `scripts\prepare-release.ps1` and `.github/workflows/release.yml`. Local tags are created only by an explicit maintainer command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\tag-release.ps1 -ConfirmTag
```

The agent must not run tag creation, tag push, GitHub Release publishing, package-manager publishing, or deployment commands. Push `v0.1.0` only after the repository owner confirms the release.
