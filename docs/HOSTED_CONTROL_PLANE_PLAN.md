# Hosted Control Plane Plan

This is the planning contract for a future public login and management surface. It is not shipped behavior.

## Current Slice

The first implementation slice is local team policy profiles:

- guardrails,
- validation recipes,
- MCP client presets,
- explicit source-upload preference,
- local persistence under `.quotarelay/team_policy_profiles.json`.

This gives teams a useful shared configuration shape through `team_policy_profile_save` and `team_policy_profile_list` without requiring hosted auth, cloud sync, telemetry, or repository uploads.

## Hosted Objects

The hosted product should start with coordination metadata:

- organization,
- user,
- team,
- repository registration metadata,
- policy profile,
- validation recipe,
- MCP client preset,
- shared decision-memory reference,
- aggregate savings report,
- audit event.

Repository contents, raw context packets, local caches, and `.quotarelay` state remain local by default.

## Security Gates

Hosted implementation must not begin until these are designed and tested:

- tenant isolation,
- authentication and session lifecycle,
- authorization for each organization object,
- SSO/OIDC integration boundary,
- audit-event schema and retention,
- rate limits and abuse controls,
- secure cookie and CSRF strategy,
- secrets handling and rotation,
- data deletion and export,
- deployment rollback and incident response.

## Privacy Gates

Default behavior:

- sync metadata, policy, configuration, hashes, and aggregate metrics first,
- require explicit opt-in before source-content upload,
- never upload raw local caches or raw context packets by default,
- keep savings claims to local estimates and aggregate reductions,
- expose what data is stored, exported, retained, and deleted.

## Test Strategy

Before a hosted beta:

- unit tests for policy validation and privacy defaults,
- MCP stdio tests for policy save/list and local inspection counts,
- integration tests for auth and authorization boundaries,
- tenant-isolation tests proving one organization cannot read another,
- audit-log tests proving sensitive contents are not logged,
- e2e tests for login, team creation, policy management, and report views,
- negative tests for unauthenticated and unauthorized access,
- dependency and secret scans in release checks,
- local regression tests proving the free core still works without hosted services.
