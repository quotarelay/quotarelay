# Known Limitations

This page describes current local MVP boundaries. It is not a roadmap promise and does not weaken the shipped local workflows in `README.md`.

## Local-only operation

- Quotarelay runs from this checkout as a local MCP server, local CLI, and optional local HTTP backend for the control plane.
- No hosted SaaS control plane, cloud sync, multi-user tenancy, or remote repository storage is implemented.
- Repository index, memory, cache, context history, registered repositories, and workspace profiles are stored under `.quotarelay/` in local roots passed to tools.
- Sharing any `.quotarelay/` state is an operator choice outside the shipped local MVP.

## Repository indexing

- Repository sync is explicit; search and context assembly require a prior `sync_repo` or CLI `sync` run.
- Indexing is local and bounded. It is intended for context retrieval, not full semantic code intelligence.
- Rust structural capsules are supported, but broad multi-language AST parsing is not implemented.
- No vector database, embedding index, language server integration, dependency graph engine, or full code graph is implemented.
- Optional `.quotarelay/ignore.json` rules apply on the next explicit sync; no background file watcher refreshes the index automatically.

## Context and savings

- Context assembly is bounded by fixed item and byte limits and explains inclusion and omission reasons.
- The shipped MVP does not claim exact tokenizer output, provider billing reduction, pricing impact, or guaranteed savings.
- Token-saving work is limited to local, explainable context reduction. Current byte counts, approximate-token estimates, and reduction ratios are local approximations, not provider tokenizer or billing truth.
- Local savings demos are local fixtures only; they are not real-world billing benchmarks or provider traces.
- Decision and guardrail memory profiles prioritize local notes in bounded packets, but they do not enforce policy or override tracker/docs truth.
- Stale context detection is read-only. It reports changed, missing, or new files relative to the last explicit sync; it does not watch files, refresh caches, or sync automatically.

## Providers and network behavior

- Quotarelay is not a model gateway and does not forward prompts, repository contents, memory, or context packets to model providers.
- No provider keys, BYOK storage, provider routing, provider capability detection, or model recommendations are implemented.
- The local MVP does not upload repository contents, emit telemetry, or collect remote feedback.

## Security and control plane

- Local HTTP endpoints do not implement authentication, authorization, TLS, or access control.
- No encryption-at-rest is claimed for `.quotarelay/` files.
- The control plane reads backend truth and tool-backed route data only. It must show unavailable or not-configured states when the backend cannot be reached.
- The control plane does not expose live health, readiness, progress, provider status, or savings metrics unless the backend exposes those fields in a shipped contract.

## Automation and recovery

- Quotarelay does not run background schedulers, background sync workers, automatic cache warming, or silent state repair.
- Cache clearing and corrupt JSON recovery are explicit operator actions.
- Release, publish, deploy, push, tag, and installer automation are outside the shipped local MVP.

## Deferred platform work

These areas remain closed until `docs/DECISIONS.md` and `EXECUTION_TRACKER.md` explicitly open future slices:

- Auth and local control-plane protection.
- Hosted multi-user tenancy.
- Cloud sync or hosted state.
- BYOK and provider key management.
- Provider request routing.
- Enterprise SSO, audit, compliance, and packaging surfaces.
