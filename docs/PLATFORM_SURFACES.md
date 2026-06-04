# Platform Surfaces

This document separates Quotarelay surfaces so public site, local MCP tooling, hosted product, native companion ideas, and deployments can evolve without blurring security boundaries.

## Current Local Surfaces

| Surface | Status | Local preview | Boundary |
|---|---|---|---|
| Public site | Available | `npm --prefix web/controlplane run dev`, then `/` | Static Terajs DOM site; no backend state required. |
| Local control plane | Available | `/control-plane?root=<repo_root>&memory_query=<query>` | Reads local loopback backend truth; do not expose the backend publicly. |
| Local MCP engine | Available | `cargo run -p mcp-server -- --cli truth` or installed `quotarelay-mcp --cli truth` | Agent-facing stdio MCP tool; no native desktop/mobile app or hosted tenancy. |

## Planned Product Surfaces

| Surface | Status | Before implementation |
|---|---|---|
| Hosted team console | Planned | Auth, authorization, tenant isolation, audit, retention, rate limits, data export/deletion, and privacy controls. |
| Private deployment | Planned | See `docs/PRIVATE_DEPLOYMENT_PLAN.md`; requires container/package decision, signing, rollout, rollback, monitoring, backup, support, and incident-response model. |
| Native desktop/mobile companion | Deferred | User evidence that a wrapper is needed, plus desktop/mobile value proposition, threat model, local-state policy, secret policy, and Terajs native target proof. |

## Local Preview Goals

The ideal local development workspace should eventually show:

- DOM public site preview,
- DOM local control-plane preview,
- hosted-console mock or local-only preview after the threat model exists,
- deployment status and artifact readiness,
- desktop/mobile preview only if a native companion becomes an approved wrapper around the MCP engine.

Do not present desktop apps, Android, iOS, container, hosted, or deployment management as shipped until tests and docs prove the surface exists.

Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\public-site-smoke.ps1` to build the Terajs frontend and verify that the public `/` route, `/control-plane` route, and public metadata are present in the generated artifacts.

## Deployment Management Expectations

Typical platform management for later paid/team work should include:

- environment list,
- release version,
- artifact provenance,
- health/readiness checks,
- rollback target,
- audit events,
- policy profile applied,
- aggregate savings report status,
- privacy/data-retention status.

These should be metadata-first. Do not upload source code, raw context packets, local caches, or `.quotarelay` state by default.
