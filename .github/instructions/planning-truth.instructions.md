---
description: "Use when editing the tracker, repo truth docs, API surface docs, or instruction files. Keeps planning honest, single-sourced, and non-drifting."
applyTo: "{README.md,EXECUTION_TRACKER.md,docs/**/*.md,openapi/**/*.yaml,.github/copilot-instructions.md,.github/instructions/*.instructions.md}"
---

# Planning And Truth Guardrails

- `EXECUTION_TRACKER.md` is the active local execution surface until a canonical external tracker is adopted.
- `README.md` documents shipped behavior only.
- `docs/VISION.md` is long-range direction only.
- `docs/MVP_SCOPE.md` defines the current v1 boundary and explicit non-goals.
- `docs/ARCHITECTURE.md` documents current runtime boundaries and ownership.
- `docs/DECISIONS.md` records locked decisions only.
- `openapi/controlplane.yaml` is schema truth for the control-plane API.
- If tracker, docs, tests, and implementation truth disagree, treat that as a blocking contradiction.
- Broad product direction may live in `docs/VISION.md`, but implementation starts only from a bounded tracker slice with explicit allowed files, non-goals, and validation.
- Do not use `docs/VISION.md` as a live backlog.
- Do not write future behavior into `README.md` as if it already ships.
- If scope changes, update `EXECUTION_TRACKER.md` first before widening implementation.
- When a shared backend or frontend contract changes, make the affected boundary proofs explicit in the tracker before closing the slice.
