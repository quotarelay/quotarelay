Here is your updated planning document, clean and tightly integrated. The AST skeletonization architecture has been added to the Current Product Thesis, the Practical Architecture Shape, and the Immediate Questions sections to ensure it serves as a core optimization goal for the next phase.
Planning Session Context - 2026-05-31
Purpose

This file captures the current planning context so a new chat can resume without rehydrating the full prior conversation.
Repo In Scope

    Workspace repo: c:\Users\brogrammer\source\quotarelay

    Current repo shape: optimization-first AI gateway/control-plane scaffold

    Current implementation focus in repo: exact cache, deterministic dedupe, routing, budgets, replay/observability

What Was Verified In Code
QuotaRelay today

QuotaRelay is currently an optimization/control-plane gateway, not the full token-burn reduction product the planning discussion converged on.

Code-grounded findings:

    services/gateway/internal/service.go

        Main chat completion orchestration path.

        Handles routing, token caps, forwarding, caching decisions, dedupe, and events.

    services/gateway/internal/http.go

        HTTP adapter for chat completions.

        No real prompt-compaction or memory-assembly layer found.

    internal/cache/cache.go

        Exact cache eligibility and whole-request keying.

    internal/dedupe/dedupe.go

        Deterministic duplicate suppression for identical requests.

    internal/routing/routing.go

        Rule-based routing only.

    internal/providers/provider.go

        Provider-neutral request contract.

    internal/providers/openai/adapter.go

        OpenAI-compatible forwarding.

    bench/replay/analysis.go and bench/replay/runner.go

        Replay and token/cost analysis, not context compaction.

Conclusion: the repo currently reduces spend via exact cache/dedupe/routing/budgets, but it does not yet solve the deeper repeated-context problem.
Headroom (chopratejas/headroom)

The relevant competitor was the GitHub project, not the unrelated company site.

Verified findings from its code:

    crates/headroom-core/src/transforms/live_zone.rs

        Performs live-zone compression/rewrite of request bodies.

        Real proxy-side prompt compression exists there.

    headroom/graph/watcher.py

        References incremental reindex via codebase-memory-mcp / cbm.

    headroom/cli/wrap.py

        Adds CBM MCP server and invokes repository indexing.

    tests/test_graph.py

        Verifies cbm cli index_repository usage in fast mode.

Conclusion: Headroom is materially closer to prompt compaction than QuotaRelay, but it still appears to compose multiple pieces rather than fully owning the entire memory + codebase-understanding + minimal-context product wedge.
The Real Problem Identified

The user clarified the real goal is not generic cost control or dashboards.

The real product target is:

    drastically reduce token burn, especially input tokens

    preserve continuity and memory across long-running agent workflows

    avoid re-sending giant historical context every turn

    know the codebase once, then update incrementally instead of rescanning everything

    assemble only the minimum useful context for each request

Short version:

    durable out-of-band memory

    durable codebase understanding/index

    incremental updates

    selective retrieval

    minimal prompt/context compiler

Current Product Thesis

The stronger wedge appears to be:

    persistent memory layer for agent/user/project state

    codebase indexing once plus incremental maintenance

    Structural Code Skeletonization (AST): Parsing files into structural signatures (imports, exports, types, function headers) via tools like tree-sitter to strip inner function bodies by default.

    exact evidence retrieval for the current task, rehydrating full file content only within the narrow, active editing seam.

    context compiler that emits only the smallest required task capsule

    measurable token savings and continuity as the product proof

This is meaningfully different from:

    dashboard-first spend tools

    exact-cache-only gateways

    pure agent-memory products without codebase grounding

    code search/index products without prompt assembly and savings proof

Market Scan Outcome

Research during the session pointed to a fragmented market:

    Mem0 and Zep: memory-centric systems

    Cursor, Sourcegraph, Continue: codebase understanding/index/search surfaces

    Headroom/CBM-related work: prompt compression and incremental code indexing pieces

Working conclusion:

    parts of the solution exist

    there does not appear to be one clearly dominant, fully integrated product whose core wedge is durable memory + codebase understanding + incremental updates + minimal prompt assembly + direct token reduction proof

Stack Direction Agreed During Planning
Core recommendation

    Keep a strong systems/backend core for the real context engine

    Terajs is acceptable and likely desirable for the frontend/dashboard/web surface if it is already chosen and owned

Technical direction discussed

For the core product engine:

    Rust for indexing, parsing/AST-skeletonization, retrieval, memory/context assembly, and high-performance backend services

For web UI / SSR / metadata / dashboard surface:

    Terajs

For operational state:

    Postgres

For analytics / realtime savings reporting at scale:

    ClickHouse

For artifacts / raw traces / replay blobs:

    object storage (S3-compatible)

Optional:

    Redis for caches/counters only if needed

    vector retrieval later, only if it proves necessary

Why Dashboard Is Secondary

The dashboard is useful, but only as a proof layer.
It should answer:

    how many tokens were avoided

    what memory/index hits occurred

    what context was included vs omitted (and what was hollowed out via skeletonization)

    latency tradeoffs

    continuity quality over time

But the product wedge is not the dashboard itself.
The wedge is the context engine.
Practical Architecture Shape Under Discussion

Probable high-level split:

    Rust ingest/index/skeletonization service

    Rust retrieval/context-assembly service

    Rust memory/state service or shared backend layer

    Postgres for operational product state

    ClickHouse for usage/savings analytics

    Terajs app for dashboard/control-plane/user-facing proof and management

Important Constraints From User Direction

    Do not optimize around generic budget/spend-control positioning if the real pain is repeated giant prompt context.

    Keep the solution grounded in actual code and evidence, not speculation.

    Terajs is already chosen and should be treated as a serious frontend option, not dismissed casually.

Open Work / Next Chat Starting Point

The next intended step was:

    inspect the sibling terajs repo directly

    validate what Terajs already provides for SSR, SEO, metadata, routing, and app-shell concerns

    decide precisely what Terajs should own vs what should remain in Rust/backend services

    turn the product thesis into a concrete MVP architecture and implementation plan