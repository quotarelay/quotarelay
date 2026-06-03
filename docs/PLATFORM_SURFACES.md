# Platform Surfaces

This document separates Quotarelay surfaces so public site, local tooling, hosted product, native previews, and deployments can evolve without blurring security boundaries.

## Current Local Surfaces

| Surface | Status | Local preview | Boundary |
|---|---|---|---|
| Public site | Available | `npm --prefix web/controlplane run dev`, then `/` | Static Terajs DOM site; no backend state required. |
| Local control plane | Available | `/control-plane?root=<repo_root>&memory_query=<query>` | Reads local loopback backend truth; do not expose the backend publicly. |
| Local MCP engine | Available | `cargo run -p mcp-server -- --cli truth` | Local stdio/CLI binary; no hosted tenancy. |

## Planned Product Surfaces

| Surface | Status | Before implementation |
|---|---|---|
| Hosted team console | Planned | Auth, authorization, tenant isolation, audit, retention, rate limits, data export/deletion, and privacy controls. |
| Private deployment | Planned | Container/package decision, signing, rollout, rollback, monitoring, backup, support, and incident-response model. |
| Native companion previews | Deferred | Android/iOS value proposition, threat model, local-state policy, mobile secret policy, and Terajs native target proof. |

## Local Preview Goals

The ideal local development workspace should eventually show:

- DOM public site preview,
- DOM local control-plane preview,
- hosted-console mock or local-only preview after the threat model exists,
- deployment status and artifact readiness,
- Android/iOS preview only if a native companion becomes an approved product surface.

Do not present Android, iOS, container, hosted, or deployment management as shipped until tests and docs prove the surface exists.

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
