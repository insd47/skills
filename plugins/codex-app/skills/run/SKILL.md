---
name: run
description: Start an independent disposable Codex App worker as a Claude Code background task. Use for parallel or disposable research and exploration; use for implementation only when the work is isolated and must not consume the persistent Codex thread's context — implementation that builds on that thread belongs to ask.
argument-hint: "[--model <model>] [--effort <effort>] [--sandbox read-only|workspace-write|danger-full-access] <prompt>"
allowed-tools: Bash(node:*)
---

Forward the raw arguments without rewriting them:

`$ARGUMENTS`

Use exactly one Bash call:

```typescript
Bash({
  command: `node "${CLAUDE_PLUGIN_ROOT}/scripts/codex-app.mjs" run "$ARGUMENTS"`,
  description: "Run Codex worker",
  run_in_background: true
})
```

The Bash tool must own backgrounding. Do not append `&`, detach the Node process, call `BashOutput`, poll status, or wait in this turn. For parallel work, issue one background Bash call per independent prompt; never multiplex unrelated questions into one worker merely to reduce tool calls.

Each command remains alive until its Codex turn finishes, prints the final result, archives the disposable Codex thread, and exits. Claude Code's background-task completion then wakes this loop. Evaluate the result before choosing another worker or steering the persistent task.

Leave model and effort unset unless the user or task requires an override. Default to `workspace-write`; use `read-only` for research that must not edit files. Require explicit user intent before selecting `danger-full-access`.
