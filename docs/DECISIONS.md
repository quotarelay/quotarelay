# Decisions

## Locked decisions

1. Quotarelay remains MCP-first. The product center is context retrieval and assembly, not provider forwarding.
2. Rust owns backend services and shared crates in this workspace.
3. Local filesystem persistence under `.quotarelay/` is the default storage model for the current MVP slices.
4. Retrieval remains bounded by explicit item and byte limits, with typed explainability reasons.
5. Control-plane surfaces may only render backend truth and proof data that already exists.
6. Repository registration state is engine-owned and exposed through thin MCP adapters.
7. Derived repository sync truth reuses repo inventory and recent run history instead of introducing a second sync-state subsystem.
8. The solo-developer local core remains free and useful; monetization targets team and enterprise collaboration, policy, reporting, support, and governance needs.
9. Team value comes from shared local-first context assets: guardrails, decision memory, repo profiles, validation recipes, MCP client presets, onboarding packs, and savings proof.
10. Team and enterprise features must not require uploading repository contents by default. Prefer local engines per developer plus shared policy/configuration artifacts.
11. Token savings must be proven through local bounded context reduction, byte/token estimates, cache hits, and benchmarks; do not claim exact provider billing savings without provider-specific proof.
12. Provider forwarding, hosted multi-user tenancy, enterprise SSO, cloud sync, and BYOK implementation remain closed until explicit decision slices open them.
13. Product priority is context compression and optimization for coding agents: reduce unnecessary tokens, expose cache reuse, surface stale state, and explain omissions before adding broader feature families.
14. Ambiguous requests should produce bounded clarifying questions when that will save context, rather than falling back to broad repository dumps.
