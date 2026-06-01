---
description: "Use when editing the Terajs dashboard UI. Keeps the control plane operational, terse, and honest."
applyTo: "{web/dashboard/src/**,web/dashboard/tests/**}"
---

# Frontend UI Guardrails

- Preserve terse operational copy. Do not add chatty helper text or marketing filler inside the product UI.
- Do not invent workflow states, savings claims, readiness states, or fake provider health.
- Prefer flatter modern surfaces with restrained separators and compact controls.
- Avoid bubble-card-inside-card layouts, decorative analytics panels, and empty filler chrome.
- Keep pages focused on the current task. Do not add unrelated summary panels unless they reduce ambiguity.
- Align all visible status and action copy to real backend contract state.
- If the backend contract is unclear or contradictory, surface the mismatch instead of masking it with UI.
- Treat debug and scaffold states as debug and scaffold states. Do not present them as production truth.
