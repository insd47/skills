---
name: status
description: Show running and recent codex-app background jobs for the current project. Use only for explicit progress checks, diagnosis, or recovery rather than routine polling.
argument-hint: "[job-id]"
disable-model-invocation: true
allowed-tools: Bash(node:*)
---

!`node "${CLAUDE_PLUGIN_ROOT}/scripts/codex-app.mjs" status "$ARGUMENTS"`

Present the command output without adding another progress summary.
