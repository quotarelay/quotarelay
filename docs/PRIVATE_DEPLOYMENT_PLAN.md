# Private Deployment Plan

Quotarelay does not currently ship a production container, hosted API, deployment controller, package-manager release, or managed service. This plan defines the boundary for future private deployment work without changing the local-first product surface.

## Current Artifacts

| Artifact | Status | Purpose |
|---|---|---|
| MCP backend binary | Available | Local stdio, CLI, and loopback HTTP truth workflows. |
| Control-plane static build | Available | Public site and local operator UI assets. |
| Local package zip | Available | Smoke artifact created by `scripts\local-package.ps1`; not signed, published, installed globally, or deployed. |
| Production container image | Not shipped | Requires the decisions and tests below. |
| Hosted team service | Not shipped | Requires hosted auth, tenancy, audit, privacy, and operations controls. |

## Deployment Principles

- Keep the local engine useful without hosted accounts.
- Keep source code, context packets, local caches, and `.quotarelay` state local by default.
- Treat static site hosting separately from any API or tenant data service.
- Treat a container as an operating model decision, not a shortcut around auth, TLS, secrets, storage, backup, or audit work.
- Prefer metadata-first team coordination before any source upload path.

## Candidate Deployment Shapes

| Shape | Fit | Required before shipping |
|---|---|---|
| Static public site hosting | Good first hosted surface | Route smoke, metadata smoke, cache headers, accessibility pass, and no secret-bearing configuration. |
| Local package zip | Good local operator artifact | License and notice files, local smoke, checksum/signing decision, and release checklist pass. |
| Private static control-plane hosting | Limited | Clear warning that local backend remains loopback-only and unmanaged. |
| MCP server container | Usually poor fit | Explicit volume policy, local path mapping, user identity, non-root runtime, and no accidental LAN exposure. |
| Hosted team API container | Future paid surface | Auth, tenant isolation, audit, rate limits, retention, backups, monitoring, signing, and incident response. |

## Deployment Management Data

Future team and enterprise surfaces may track:

- environment name,
- deployment type,
- release version,
- artifact digest,
- build provenance,
- health and readiness state,
- applied policy profile,
- audit event status,
- backup and retention status,
- rollback target,
- aggregate savings report status.

This metadata must not imply that Quotarelay has uploaded source code, raw prompts, context packets, caches, memories, or repository state.

## Required Tests Before Container Work Ships

- Container image builds reproducibly from a clean checkout.
- Runtime process runs as non-root.
- Image contains no `.quotarelay`, `.env`, local state, secrets, private paths, or dependency cache.
- HTTP surfaces bind only to approved interfaces.
- Health and readiness checks reflect real dependencies.
- Static routes and metadata pass smoke tests.
- License, notice, and release docs are present in artifacts.
- SBOM, signing, provenance, and rollback checks are documented.
- Backup, restore, migration, and retention behavior are tested for any persisted server state.
- Unauthorized tenant and repository access is denied by proof tests.

## Open Decisions

- Whether private deployment means a static site, a hosted team service, a customer-managed service, or all three.
- Whether the first paid surface should be hosted by Quotarelay or self-hosted by teams.
- Which signing and provenance system to use.
- Which database, backup, migration, and retention model applies to hosted/team state.
- Which support tier owns incident response and rollback execution.

Until these decisions are closed, public docs should describe private deployment as planned work and the local package as a smoke artifact.
