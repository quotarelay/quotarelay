# Hosted Threat Model

This threat model gates any future hosted login, organization management, shared policy, shared memory, savings reporting, or deployment-management implementation.

It is not shipped behavior.

## Assets

Protect these assets before any hosted beta:

- organization identity,
- user identity and sessions,
- team and repository metadata,
- policy profiles,
- validation recipes,
- MCP client presets,
- shared decision-memory references,
- aggregate savings reports,
- audit events,
- billing and plan metadata.

Repository contents, raw context packets, local caches, and `.quotarelay` state are out of scope for default hosted sync and must remain local unless an explicit enterprise opt-in is designed and approved.

## Trust Boundaries

| Boundary | Default |
|---|---|
| Browser to hosted API | Authenticated HTTPS only. |
| Hosted API to database | Tenant-scoped queries and audited writes. |
| Local engine to hosted service | Metadata-first sync only; no source upload by default. |
| Admin actions | Role-checked and audit-logged. |
| Integrations | Least privilege tokens with rotation and revocation. |

## Primary Threats

| Threat | Required controls |
|---|---|
| Cross-tenant data read | Organization-scoped authorization on every read and write, tenant-isolation tests, query helpers that require tenant id. |
| Session theft | Secure cookies, short-lived sessions, rotation on privilege changes, logout invalidation, CSRF protection. |
| Unauthorized admin action | Role-based access control, step-up checks for destructive actions, audit events. |
| Source-content leakage | Source upload disabled by default, explicit opt-in gate, content classification, redaction, separate retention policy. |
| Sensitive audit logging | Structured audit fields, no raw source/context packets in logs, log tests with sensitive fixtures. |
| Integration token exposure | Encrypted storage, scoped tokens, rotation, revocation, secret scans. |
| Abuse and scraping | Rate limits, invite controls, account lockout/backoff, anomaly monitoring. |
| Data deletion failure | Tested export/delete workflows and retention windows. |
| Deployment compromise | signed artifacts, provenance, rollback, environment separation, monitored releases. |

## Required Tests Before Hosted Beta

- unauthenticated request rejection,
- unauthorized cross-org read/write rejection,
- role permission matrix,
- CSRF protection for session-backed writes,
- audit event creation for policy/admin changes,
- audit log sensitive-content exclusion,
- source upload disabled by default,
- aggregate savings report contains no raw source,
- data export and deletion workflows,
- integration token rotation and revocation,
- deployment rollback smoke.

## Implementation Rules

- Start with organization, user, team, and policy metadata only.
- Do not add provider forwarding while building hosted coordination.
- Do not store provider keys until a separate BYOK decision defines encryption, rotation, and access rules.
- Do not claim compliance certifications before evidence exists.
- Keep the local Apache-2.0 core fully useful without hosted login.
- Keep all hosted sync opt-in, explainable, and reversible.

## Open Decisions

- identity provider strategy,
- session store and duration,
- database and migration tooling,
- audit retention period,
- billing provider,
- support access controls,
- private deployment topology,
- whether any enterprise source-content opt-in is allowed at all.
