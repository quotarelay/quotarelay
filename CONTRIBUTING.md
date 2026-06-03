# Contributing

Thanks for helping improve Quotarelay. This project is in a local-first MVP stage, so contributions should preserve a conservative public surface.

## Development Setup

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bootstrap.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\demo-local.ps1
```

## Validation

Run focused checks for your change, then run the release check before opening a public PR:

```powershell
cargo fmt --all --check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-line-counts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release-check.ps1
```

## Pull Request Expectations

- Keep changes small and scoped to one behavior or documentation concern.
- Include exact validation commands and results.
- Update `README.md`, `SECURITY.md`, `docs/`, and `openapi/controlplane.yaml` when public behavior or contracts change.
- Do not commit `.quotarelay/`, `target/`, `node_modules/`, `web/controlplane/dist/`, `.env`, temp files, local package archives, or machine-specific editor settings.
- Do not add external network behavior, provider forwarding, auth, deployment automation, telemetry, or multi-user features without an explicit decision and implementation plan.

## Security

Use `SECURITY.md` for vulnerability reporting. Do not open public issues containing secrets, private repository content, exploit payloads, or local `.quotarelay` state.
