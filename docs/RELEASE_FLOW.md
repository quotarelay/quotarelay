# Release Flow

This flow automates local release readiness without publishing, deploying, or exposing the local HTTP backend.

## Release Manifest

`release.json` is the release manifest for the current public candidate:

- version: `0.1.0`
- tag: `v0.1.0`
- release name: `Quotarelay 0.1.0`
- draft: `true`
- prerelease: `false`
- local-only: `true`
- publish: `false`

The manifest also records the GitHub About topics from `docs/GITHUB_ABOUT.md`.

## Maintainer Commands

Pull requests and pushes to `main` run the release-candidate gate in CI:

- `scripts\release-check.ps1`
- `scripts\local-package.ps1`
- local package artifact upload
- dependency audit

The post-merge maintainer step is only the irreversible release action: create and push the version tag after the release-candidate checks pass.

Check that all version surfaces agree:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\assert-release-version.ps1
```

Run full release prep:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\prepare-release.ps1
```

Run the dependency audit when registry access is available:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\prepare-release.ps1 -Audit
```

Build the local package during prep:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\prepare-release.ps1 -BuildLocalPackage
```

Create the local release tag after review:

```powershell
npm run release:tag -- -ConfirmTag
```

Push the tag only after owner approval:

```powershell
git push origin v0.1.0
```

## Automation Boundary

The release scripts validate, package, and optionally create a local tag. They do not publish a GitHub Release, push tags, deploy services, upload repository contents, add provider calls, or change the local-first product boundary.
