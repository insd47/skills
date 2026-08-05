---
name: ask
description: Send one prompt through Codex Desktop IPC to the visible Codex App task associated with this workspace. Use for implementation and follow-up work that must accumulate in the persistent Codex thread, and for steering an in-flight Codex turn while Claude remains the orchestration loop owner; for parallel disposable research use run instead.
argument-hint: "<prompt>"
allowed-tools: Bash(node:*)
---

Forward the raw prompt without rewriting it:

`$ARGUMENTS`

Use exactly one Bash call:

```typescript
Bash({
  command: `node "${CLAUDE_PLUGIN_ROOT}/scripts/codex-app.mjs" ask "$ARGUMENTS"`,
  description: "Ask Codex App",
  run_in_background: true
})
```

The Bash tool must own backgrounding. Do not append `&`, detach the Node process, call `BashOutput`, poll status, or wait in this turn.

If the associated Codex thread already has an active plugin job, the command steers that turn and exits; the original background task remains responsible for the final completion event. Otherwise the command remains alive until Codex completes, then its final stdout wakes this Claude loop.

The command selects exactly one unarchived task whose title matches `CODEX_APP_TITLE`. If that variable is absent or the title is ambiguous, report the error and ask the user for `CODEX_APP_THREAD_ID`; never infer from the cwd, message history, or handoff wording, and never replace the dispatch with a separate Codex task.

After dispatch, continue useful planning or tell the user that Codex is running. When completion arrives, evaluate Codex's output and choose the next loop action. Do not substitute Claude-side implementation if dispatch fails.
