---
description: "Use when editing Go backend services, internal modules, repositories, or provider adapters. Enforces adapter, service, repository, and domain boundaries."
applyTo: "{services/**/*.go,internal/**/*.go}"
---

# Backend Boundary Guardrails

- HTTP handlers are adapters only.
- Service layers own orchestration.
- Repository layers own persistence only.
- Provider adapters own provider-specific transport logic only.
- Pure deterministic rules belong in domain-owned modules, not handlers or repositories.
- Database access stays inside repository-owned code.
- Route handlers must not import provider SDK code directly.
- Heavy work such as embeddings, rollups, or archive processing must stay off the hot request path.
- Keep the gateway stateless.
- Fail open on optimization-stage failure by forwarding the request unmodified and recording an event.
