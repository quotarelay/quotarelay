# Decisions

## Locked decisions

1. Quotarelay remains MCP-first. The product center is context retrieval and assembly, not provider forwarding.
2. Rust owns backend services and shared crates in this workspace.
3. Local filesystem persistence under `.quotarelay/` is the default storage model for the current MVP slices.
4. Retrieval remains bounded by explicit item and byte limits, with typed explainability reasons.
5. Control-plane surfaces may only render backend truth and proof data that already exists.
6. Repository registration state is engine-owned and exposed through thin MCP adapters.
7. Derived repository sync truth reuses repo inventory and recent run history instead of introducing a second sync-state subsystem.