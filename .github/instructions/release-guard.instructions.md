---
description: "Local release guard. Only prepare releases locally when explicitly asked. Never publish, deploy, tag releases, or run remote release workflows from the agent."
applyTo: "**"
---

# Release Guard

- Never publish, deploy, or trigger release workflows from the agent.
- Never run registry auth, publish, or tag commands.
- Allowed release work is local prep only: docs, versions, changelog, validation, commit, and push when explicitly asked.
- After local prep, stop and hand off to the user.
