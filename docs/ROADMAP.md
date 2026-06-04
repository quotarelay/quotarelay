# Roadmap

This roadmap locks the product path. It is not a shipped-behavior document; `README.md` remains the source for what works today, and `EXECUTION_TRACKER.md` remains the active execution queue.

For a compact shipped/queued/deferred boundary, see `docs/TRUTH_MATRIX.md`.

## North star

Quotarelay is a local-first context compression and optimization engine for coding agents. It helps agents use less context by syncing repositories locally, assembling bounded context packs, preserving team decisions, caching repeated work, and recommending the smallest useful validation path.

The product should make a large repository feel cheaper and safer to work with by preventing agents from repeatedly rereading broad codebase context.

It must also be easy to use. Operators should not need to understand retrieval internals to see:

- what context was sent,
- what was omitted,
- why the packet was assembled,
- whether a cache hit occurred,
- whether the packet is stale,
- approximate token size,
- approximate context reduction versus broad/raw context,
- and what question Quotarelay needs answered if the request is ambiguous.

## Audience

- Solo developers: free local MCP/CLI context engine that is useful without cloud services.
- Small teams: shared local-first repo profiles, guardrails, decision memory, validation recipes, and handoff packets.
- Enterprise teams: policy controls, auditability, support, private deployment guidance, and savings evidence.

## Phase 1: Local MVP

Goal: ship a trustworthy local tool regular developers can use for real repositories.

Core outcomes:

- Register and sync local repositories.
- Search and assemble bounded context.
- Store durable local memory.
- Inspect local state and caches.
- Expose honest MCP, CLI, HTTP truth, and control-plane truth.
- Document quickstart, troubleshooting, local privacy, known limitations, and validation.

Must not include:

- Provider forwarding.
- Cloud sync.
- Multi-user tenancy.
- Fake savings/readiness claims.
- Release or deployment automation from agents.

## Phase 2: Token-Saving Workflow

Goal: make token savings visible and useful during agent work.

Core outcomes:

- Context budget estimate for context packs and handoff packets.
- Cache hit/miss visibility for local retrieval/cache surfaces.
- Clear stale/fresh state for cached packets.
- Agent handoff packet with task, relevant context, decision memory, validation commands, and blockers.
- Decision memory profile for repo/team guardrails.
- Stale-context detection after repo changes.
- Diff-aware context assembly for local changes.
- Repo map summaries for fast orientation.
- Validation recommender based on touched/query-relevant files.
- Clarifying-question support when a request is too broad or ambiguous to retrieve efficiently.
- Local benchmark fixture comparing raw context size to Quotarelay context size.

Success measure:

- A large repo workflow can use a small bounded packet instead of dumping broad repo files into the agent context.
- Savings are described as local byte/approximate-token reduction, not provider billing guarantees.
- Cache hits and stale states are visible enough that operators can trust when local work was reused.

## Phase 3: Team Layer

Goal: let teams share the brain of the repo while each developer can keep code local.

Core outcomes:

- Team profile config for repo groups, default retrieval limits, validation recipes, and guardrails.
- Local team policy profiles for guardrails, validation recipes, MCP client presets, and explicit source-upload preference.
- Shared decision memory that agents load before edits.
- Handoff packet templates for bug fix, feature slice, review, refactor, and release work.
- MCP client presets for verified local setups.
- Team-visible savings reports generated from local runs.

Boundary:

- Team sharing should prefer checked-in or explicit local configuration.
- Do not upload source code or local state by default.

## Phase 4: Enterprise Layer

Goal: monetize governance, scale, support, and auditability without weakening the free core.

Potential outcomes after decision slices approve them:

- Admin-managed policy profiles.
- Append-only audit event logs.
- Organization validation policy.
- Private deployment guidance.
- Security/compliance documentation.
- Priority support and custom integration assistance.
- Optional hosted coordination for policies and reports, not repository contents by default.

Still gated:

- SSO.
- Multi-user cloud tenancy.
- BYOK storage.
- Provider capability truth beyond local configuration.
- Any provider request routing.

## Monetization Lock

The free core must remain valuable for regular developers.

Paid value should come from:

- Collaboration.
- Governance.
- Team memory.
- Policy.
- Reporting.
- Onboarding.
- Support.
- Enterprise assurance.

Do not make the free version a crippled demo. Community trust and solo-developer adoption are part of the business strategy.

See `docs/COMMERCIAL_STRATEGY.md` for the open-core gating model, hosted management surface, and savings metrics boundaries.

See `docs/HOSTED_CONTROL_PLANE_PLAN.md` for the privacy and security gates before public login or hosted organization management ships.

See `docs/PLATFORM_SURFACES.md` for the local DOM surfaces, planned deployment management path, and deferred Android/iOS companion preview boundary.

See `docs/HOSTED_THREAT_MODEL.md` for the threat model required before hosted auth, tenant isolation, audit logging, integrations, or deployment management begin.

See `docs/PRIVATE_DEPLOYMENT_PLAN.md` for the planned private deployment shapes and tests required before container or customer-managed deployment work ships.
