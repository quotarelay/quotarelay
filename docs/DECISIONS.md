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
15. Auth and authorization are deferred for the local MVP; local HTTP endpoints are operator-local tools, not protected network services.
16. Operator identity is deferred; current local state is scoped by filesystem root and explicit operator action, not user accounts.
17. Audit logs are deferred; shipped history and feedback records are local operational records, not compliance audit trails.
18. BYOK key storage is rejected for the local MVP; no provider keys are stored until a future decision defines storage, encryption, and rotation rules.
19. Provider capability truth is rejected for the local MVP; Quotarelay does not detect models, route provider calls, or recommend providers.
20. Hosted multi-user feasibility remains a decision-only topic; no multi-user implementation starts until tenancy, identity, storage, and repository-content boundaries are approved.
21. Local team policy profiles are open for implementation as a privacy-preserving team layer. They may store guardrails, validation recipes, MCP client presets, and an explicit source-upload preference under local `.quotarelay` state.
22. Hosted login and organization management are open for planning only. Implementation must wait for an approved threat model covering auth, authorization, tenant isolation, audit logs, retention, abuse controls, and source-content opt-in.
23. Quotarelay productization is agent-tool first: `quotarelay-mcp` is the branded local MCP command, while desktop, mobile, Electron, tray, and app-store surfaces are deferred until user evidence shows a companion wrapper is needed.
