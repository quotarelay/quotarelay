# Guardrails

- `README.md` documents shipped behavior only.
- `EXECUTION_TRACKER.md` is the active local execution mirror.
- `docs/VISION.md` is directional only.
- Handlers and routes stay thin; persistence and retrieval logic belong in backend crates.
- The control plane must not invent live savings, health, or readiness state.
- Keep outputs bounded and explainable before adding broader retrieval or optimization layers.
- Prefer local filesystem state and explicit contracts over convenience abstractions.
- Do not publish, deploy, or run release automation from the agent.