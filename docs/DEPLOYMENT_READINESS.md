# Deployment Readiness

Quotarelay is not currently a hosted service. The shipped surface is local-first:

- stdio MCP server,
- local CLI,
- loopback-only HTTP control-plane backend,
- static control-plane build,
- local filesystem state under `.quotarelay`.

## What Works Today

Local operators can run the MCP server over stdio, run CLI workflows, run the HTTP truth surface on `127.0.0.1`, build a local zip artifact with `scripts/local-package.ps1`, and run `scripts/release-check.ps1` to validate the local release surface.

See `docs/PLATFORM_SURFACES.md` for the current split between the public DOM site, local control plane, local MCP engine, planned hosted team console, private deployment, and deferred Android/iOS companion previews. See `docs/HOSTED_THREAT_MODEL.md` before any hosted login, tenant, audit, or deployment-management implementation starts. See `docs/PRIVATE_DEPLOYMENT_PLAN.md` before adding container, customer-managed, or production deployment behavior.

## Not Production-Deployment Ready

Before any hosted, LAN, container, package-manager, or enterprise deployment, Quotarelay needs explicit decisions and implementation for:

- authentication and authorization,
- organization login and user identity,
- team and repository membership,
- TLS and network exposure policy,
- operator identity,
- audit logging,
- secrets and BYOK key storage,
- provider capability truth and provider routing boundaries,
- state storage, backup, and migration,
- multi-user tenancy,
- update and rollback strategy,
- supply-chain signing and artifact provenance,
- deployment-specific hardening and monitoring.

## Future Hosted Management Surface

A hosted Quotarelay surface should start with coordination and management data:

- organizations,
- users and teams,
- repository metadata,
- policy profiles,
- validation recipes,
- shared decision memory,
- MCP client presets,
- aggregate savings metrics,
- audit events.

It should not upload source code, raw context packets, local caches, or `.quotarelay` state by default.

## Safe Future Deployment Path

1. Keep the local engine and adapter boundaries intact.
2. Close the deployment decisions in `docs/PRIVATE_DEPLOYMENT_PLAN.md`.
3. Add a threat model and controls before adding network exposure.
4. Add tests that prove unauthorized local state cannot be read.
5. Add packaging and signing only after release validation is stable.
6. Keep public docs explicit about what is deployed, what is local-only, and what remains unsupported.
