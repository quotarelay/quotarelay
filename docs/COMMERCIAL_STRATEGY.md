# Commercial Strategy

Quotarelay uses an open-core strategy:

- the local developer core is Apache-2.0 and should remain genuinely useful,
- paid products should add collaboration, governance, managed operations, reporting, and enterprise controls,
- paid products must not require uploading repository contents by default.

## What Ships Free

The free local core includes:

- local MCP server,
- local CLI,
- local repository sync and search,
- local context assembly,
- local handoff packets,
- local memory,
- local retrieval cache,
- local validation recommendations,
- local control plane,
- local savings demos and benchmarks,
- local `.quotarelay` state.

Do not remove or cripple these capabilities to force upgrades. Adoption and trust come from the local core being useful on real work.

## What Teams Pay For

Teams and companies usually need capabilities that solo local users do not:

- hosted team control plane,
- organization login,
- team and repo membership,
- shared policy and guardrail packs,
- approved validation recipes,
- shared decision memory with review workflow,
- team-visible savings and cache metrics,
- audit logs for changes to shared policies and memory,
- GitHub, GitLab, Jira, Linear, Slack, and Teams integrations,
- private deployment support,
- SSO/SAML/OIDC,
- compliance documentation,
- priority support and onboarding.

## Public Login and Management Surface

The future hosted surface should manage coordination data, not source code by default.

Minimum hosted objects:

- organization,
- user,
- team,
- repository registration metadata,
- policy profile,
- validation recipe,
- shared memory note,
- MCP client preset,
- savings report metadata,
- audit event.

Default privacy boundary:

- keep source code, raw context packets, local caches, and `.quotarelay` state local unless an operator explicitly opts in,
- sync metadata, hashes, policy, configuration, and aggregate metrics first,
- make any source-content upload a separate enterprise decision and control.

## Gating Model

Recommended gates:

- Free: local-only core under Apache-2.0.
- Team: hosted coordination, shared policies, shared memory, integrations, and aggregate metrics.
- Business: admin roles, private repos, team dashboards, policy workflows, and support.
- Enterprise: SSO, audit logs, private deployment, compliance docs, custom integrations, and SLA.

Avoid gates that block local productivity. Gate team coordination and operational trust instead.

## Savings Metrics

Free local metrics:

- raw bytes considered,
- included bytes,
- approximate tokens,
- local reduction ratio,
- cache hit/miss,
- stale status,
- omission reasons,
- one-command demo and benchmark output.

Paid team metrics:

- aggregate context reduction by repo and team,
- cache reuse trends,
- stale-context incidents,
- broad-dump avoidance count,
- validation recommendation adoption,
- policy compliance,
- shared-memory usage and review age.

Boundaries:

- do not claim exact provider tokenization without provider-specific proof,
- do not claim guaranteed billing savings,
- do not collect raw source or context packets by default.

## Why Apache-2.0 Still Fits

Apache-2.0 lowers adoption friction for developers, AI-tooling ecosystems, and companies that need legal clarity before trying the project. Monetization should come from hosted/team/enterprise value, not from making the local core artificially weak.

The brand, hosted service, enterprise controls, managed deployment, integrations, and support remain Quotarelay commercial assets.
