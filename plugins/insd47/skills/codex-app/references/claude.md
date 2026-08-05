# Claude orchestration

Act as the planner, critic, and loop owner. Read `@CLAUDE.md` when the project provides it.

Use the Claude-only `codex-app` plugin when it is installed:

- Send implementation and any follow-up that must build on accumulated context through `/codex-app:ask`, which continues the persistent Codex App task associated with the integrated terminal.
- Send research and exploration through parallel `/codex-app:run` workers. Use `run` for implementation only when the work is isolated and gains nothing from the persistent thread's context.
- Prefer several independent `run` calls for genuinely parallel questions; keep follow-up work in the same worker only when prior context materially saves tokens.

Treat the integrated terminal environment as immutable after Claude starts. Codex Desktop supplies `CODEX_APP_TITLE` when it creates the terminal; `/codex-app:ask` uses only that title to discover the persistent task. If the title is absent or ambiguous, ask the user for a task ID to set as `CODEX_APP_THREAD_ID`. Do not infer a task from the cwd, message count, or handoff wording.

Treat every delegated shell invocation as a Claude background task. Return to planning or discussion immediately after dispatch. When the background task completes, inspect Codex's final output, compare it with the acceptance criteria, and either steer the same task, delegate another bounded task, or report completion.

Do not poll continuously. Use `/codex-app:status` only for user-requested progress checks or recovery. Use `/codex-app:interrupt` when the direction is wrong or the task should stop.

Calling `/codex-app:ask` again automatically steers an active persistent task. Use `/codex-app:steer <job-id> <prompt>` only to redirect a specific parallel `run` worker.

Never silently replace a failed Codex dispatch with Claude-side implementation. Explain the dispatch failure and repair the delegation path first.
