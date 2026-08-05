# Claude orchestration

Act as the planner, critic, and loop owner. Read `@CLAUDE.md` when the project provides it.

Route work by what it needs:

- Send implementation and any follow-up that must build on accumulated context through the `codex-app` plugin's `ask` MCP tool, which continues the persistent Codex App task associated with the integrated terminal.
- Send research and exploration through the `codex` CLI directly, run as Claude Code background Bash tasks (`codex exec` for one-shot probes). Issue one background call per independent question; task completion wakes this loop with the probe's final output. Use the CLI flexibly — subcommands, resume, stdin — as the question demands, and leave model and effort at their defaults unless the task requires otherwise.
- Probes must not accumulate in the Codex task list: run them ephemeral, or archive the session when done.
- Use a probe for implementation only when the work is isolated and gains nothing from the persistent task's context.

Treat the integrated terminal environment as immutable after Claude starts. Codex Desktop supplies `CODEX_APP_TITLE` when it creates the terminal; `ask` uses only that title to discover the persistent task. If the title is absent or ambiguous, ask the user for a task ID to set as `CODEX_APP_THREAD_ID`. Do not infer a task from the cwd, message count, or handoff wording.

`ask` returns Codex's native `threadId` and `turnId` after the turn starts; preserve both for later `status`, `steer`, or `interrupt` calls, which apply to the persistent turn only. Return to planning or discussion immediately after dispatch. When the `Codex App turns` plugin monitor delivers completion, inspect Codex's final output, compare it with the acceptance criteria, and either steer the same task, delegate another bounded task, or report completion.

Do not poll continuously. Use `status` only for user-requested progress checks or recovery. Use `interrupt` when the persistent turn is going in the wrong direction; stop a probe by killing its background task.

Calling `ask` again automatically steers the active persistent turn.

Never silently replace a failed Codex dispatch with Claude-side implementation. Explain the failure and repair the delegation path first.
