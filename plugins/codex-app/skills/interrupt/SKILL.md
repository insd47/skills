---
name: interrupt
description: Interrupt an active codex-app turn while preserving its Codex thread. Use when a delegated task is going in the wrong direction, is no longer needed, or must release resources.
argument-hint: "[job-id]"
disable-model-invocation: true
allowed-tools: Bash(node:*)
---

!`node "${CLAUDE_PLUGIN_ROOT}/scripts/codex-app.mjs" interrupt "$ARGUMENTS"`
