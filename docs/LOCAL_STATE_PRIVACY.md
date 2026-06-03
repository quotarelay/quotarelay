# Local State Privacy And Security

Quotarelay local MVP stores state on the operator machine. It does not provide authentication, encryption-at-rest, hosted tenancy, cloud sync, or provider routing.

## What Is Stored Locally

Quotarelay writes local state under `.quotarelay` inside roots passed to tools.

| File | Purpose |
|---|---|
| `.quotarelay/index.json` | Bounded repository index used by local search and context assembly. |
| `.quotarelay/memory_notes.json` | Durable memory notes written by the operator or agent. |
| `.quotarelay/context_runs.json` | Recent bounded context assembly history with inclusion and omission reasons. |
| `.quotarelay/exact_match_cache.json` | Local exact-search retrieval cache. |
| `.quotarelay/retrieval_capsules.json` | Local overview and task-capsule retrieval cache. |
| `.quotarelay/registered_repositories.json` | Registered repo roots and display metadata for a chosen state root. |
| `.quotarelay/workspace_profiles.json` | Local workspace profile names, repo roots, and default retrieval limits. |
| `.quotarelay/team_policy_profiles.json` | Local team guardrails, validation recipes, MCP presets, and source-upload preference. |
| `.quotarelay/ignore.json` | Optional local index ignore config when the operator creates it. |

State files can contain source snippets, memory text, repo paths, queries, context history, guardrails, and validation recipes. Treat them as project-local working data.

## What The Local MVP Does Not Send

The shipped local MVP does not:

- Call model providers.
- Forward prompts or repository contents to providers.
- Upload repository contents to a Quotarelay service.
- Sync `.quotarelay` state to cloud storage.
- Emit telemetry or remote feedback.
- Provide hosted multi-user tenancy.

## What Not To Commit

Do not commit `.quotarelay` state unless you have intentionally reviewed it and want it in the repository. In normal use, keep `.quotarelay` local.

Review before committing:

- Memory notes.
- Context run history.
- Cache files.
- Registered repository paths.
- Workspace profiles that include local machine paths.

## Current Limitations

- No authentication or access control is implemented for local HTTP control-plane endpoints.
- No encryption-at-rest is claimed for `.quotarelay` files.
- No BYOK storage or provider key handling is implemented.
- No cloud security, compliance, or hosted tenancy posture is claimed.
- Any sharing of local state is an operator choice outside the shipped local MVP.

For recovery guidance, see `docs/TROUBLESHOOTING.md`. For deferred platform boundaries, see `docs/TRUTH_MATRIX.md`.
