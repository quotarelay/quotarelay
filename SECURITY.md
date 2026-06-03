# Security Policy

## Supported Surface

Quotarelay is currently a local-first MVP. It provides:

- a stdio MCP server,
- a local CLI,
- a local HTTP control-plane backend that must bind to loopback addresses only,
- local filesystem state under `.quotarelay` in roots explicitly passed to tools.

No hosted service, authentication system, provider gateway, cloud sync, telemetry pipeline, BYOK storage, or multi-user tenancy is shipped.

## Reporting Vulnerabilities

Please report suspected vulnerabilities privately through the repository owner's preferred GitHub security advisory flow when available. Do not open public issues containing secrets, exploit payloads, private repository contents, or local state dumps.

Include:

- affected commit or release,
- operating system,
- exact command or MCP/HTTP route used,
- whether `.quotarelay` state or repository contents were exposed,
- minimal reproduction steps using non-sensitive sample data.

## Local Data and Privacy

Quotarelay can store source snippets, repository paths, queries, memory notes, cache entries, and context history under `.quotarelay`. Treat that directory as project-local working data and do not commit it. The repository `.gitignore` excludes `.quotarelay`, `.env`, build outputs, dependency directories, and temporary local artifacts.

## Network Boundary

The HTTP backend is intended for local operator use. It refuses non-loopback bind addresses and does not implement auth, TLS, CORS policy, user identity, or access control. Keep it bound to `127.0.0.1` or another loopback address.

## Non-Goals

The public MVP does not:

- forward prompts or repository contents to model providers,
- store provider API keys,
- provide cloud sync or hosted storage,
- provide multi-user access control,
- claim exact provider billing savings.
