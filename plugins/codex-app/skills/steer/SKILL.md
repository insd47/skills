---
name: steer
description: Redirect one active codex-app background job without replacing its Codex turn or Claude background task. Use for a specific parallel run worker; repeated ask calls already steer the persistent task automatically.
argument-hint: "<job-id> <prompt>"
allowed-tools: Bash(node:*)
---

!`node "${CLAUDE_PLUGIN_ROOT}/scripts/codex-app.mjs" steer "$ARGUMENTS"`

Confirm that the original background task remains active. Do not start another worker or poll for its result.
