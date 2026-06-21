# Vision

Quotarelay aims to be the best local-first context compression and optimization layer for coding agents.

It should greatly reduce unnecessary model context by helping agents retrieve, compress, cache, and explain only the repository information they need.

The product is for developers and teams using AI coding agents on real codebases, especially large or important repos where context bloat, repeated file scans, and agent drift waste time and model tokens.

The free Apache-2.0 local core is the adoption product, not a trial stub. Hosted, team, and enterprise ideas stay sidelined until the local MVP proves regular developer adoption.

Long-range direction:

- Keep context assembly MCP-first instead of turning the product into a generic model gateway.
- Prefer local, incremental, inspectable state before heavier distributed infrastructure.
- Make context inclusion and omission explainable at the boundary that agents and control-plane operators can inspect.
- Support a control plane that mirrors backend truth instead of inventing unproven product claims.
- Optimize for fewer tokens sent to agents by default, not cheaper provider routing.
- Make ease of use a product requirement: a regular developer should be able to install, sync, and get a useful context packet without understanding the internals.
- Make savings clarity a product requirement: operators should be able to see context size, approximate token estimate, cache hit/miss state, stale state, and why content was included or omitted.
- When a request is ambiguous, prefer asking for the smallest clarifying detail needed to build the right context packet instead of dumping broad context.
- Make local cache hits, bounded handoff packets, repo maps, decision memory, stale-context detection, and validation recommendations core product capabilities.
- Let teams share policy, guardrails, repo profiles, validation recipes, and durable decisions while each developer can keep source code and local state on their own machine.
- Keep community trust high by making the solo-developer/local workflow useful without payment or cloud dependency.

## Product path

1. Local MVP: a self-hosted MCP/CLI context engine for repo sync, bounded retrieval, durable memory, cache/state inspection, and honest control-plane truth.
2. Token-saving workflow: context budget estimates, agent handoff packets, decision memory, stale context detection, diff-aware retrieval, repo map summaries, validation recommendations, and local savings benchmarks.
3. Deferred team layer: checked-in or shared team profiles for guardrails, approved validation commands, decision memory, MCP client presets, and onboarding templates after local adoption proof.
4. Deferred enterprise layer: policy controls, audit logs, admin-managed profiles, support, private deployment guidance, and security/compliance documentation after explicit future planning.

## Product locks

- Quotarelay is not a provider gateway.
- Quotarelay is not a cloud-first agent platform.
- Quotarelay is not a generic admin console.
- Quotarelay is not a fake token-savings dashboard.
- Quotarelay should prove local context reduction with bounded byte/token estimates and local benchmarks, without claiming exact provider billing savings.

This document is directional only. Shipped behavior is documented in `README.md` and the current execution mirror in `docs/EXECUTION_TRACKER.md`.
