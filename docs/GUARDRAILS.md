# Guardrails

- `README.md` documents shipped behavior only.
- `EXECUTION_TRACKER.md` is the active local execution mirror.
- `docs/VISION.md` is directional only.
- Handlers and routes stay thin; persistence and retrieval logic belong in backend crates.
- The control plane must not invent live savings, health, or readiness state.
- Keep outputs bounded and explainable before adding broader retrieval or optimization layers.
- Prefer local filesystem state and explicit contracts over convenience abstractions.
- Do not publish, deploy, or run release automation from the agent.

## Separation of concerns

- MCP handlers, HTTP handlers, and CLI command parsing are adapters only: parse input, call the owning crate/service, serialize output, and map errors.
- Business rules, retrieval rules, context packing, memory behavior, cache behavior, and repository registration semantics belong in engine/service crates, not in adapters.
- Repository indexing owns file walking, ignore rules, index persistence, document extraction, and code-search primitives.
- Persistence code owns state-file paths, serialization, recovery, and migrations for its own storage area.
- UI code renders backend truth and tool-backed results; it must not duplicate backend readiness, retrieval, memory, cache, or repository-state logic.
- Do not mix API adapter code with business rules or persistence logic in the same new module.
- When an existing oversized file already mixes concerns, new work should extract a cohesive seam by responsibility rather than adding another helper pile.
- Cross-boundary contracts must be explicit data structures or narrow functions, not broad imports of whole adapter modules.

## Maintainability guardrails

- Target source files at 500 lines or fewer.
- New source files must not exceed 500 lines.
- Existing source files over 500 lines are legacy-debt files, not acceptable growth targets.
- If a slice touches an oversized source file, the agent must either extract a cohesive seam into a smaller module during the same slice or record an explicit blocker/refactor follow-up in `EXECUTION_TRACKER.md`.
- Do not add unrelated behavior to oversized files just because they already contain adjacent code.
- Tests may exceed 500 lines only when the test module is split by behavior and the production seam remains small; prefer focused test modules over one giant test pile.
- Generated lockfiles and machine-generated artifacts are exempt from the 500-line source-file limit, but agents must not hand-edit them except through the owning package tool.
