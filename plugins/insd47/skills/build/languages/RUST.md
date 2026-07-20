# Rust Guidance

Apply this guide together with the parent `SKILL.md` when Rust files are in scope.

## Visibility: plain `pub` behind private ancestors

The module tree is the entire access-control mechanism.

- `mod child;` is private by default; a `pub` item inside it is a subtree contract — the private ancestor seals it from the outside.
- `pub use` at the parent adopts a concept into the parent's vocabulary.
- `pub mod child;` when callers deliberately enter the subdomain. Trigger: caller code naturally reads with the child's path (`infra::judge::JudgeStream`) and imports several of its items. When this trigger fires, prefer `pub mod` over re-exporting the child's vocabulary item by item.
- **Never `pub(crate)` or `pub(super)` in production code.** Needing one means a boundary is misdrawn — reshape the tree or re-export at the honest ancestor until plain `pub` behind private ancestors suffices. Treat existing ones as cleanup targets when in scope.
- Single exception: `#[cfg(test)]` seams. A test-only fixture (`pub(super) fn stub()`) may use `pub(super)`, because plain `pub` would falsely invite production callers. The distance modifier is the true signal here: "tests in this subtree only."

## Facades own their subtree's story

A parent module holds child declarations, selective re-exports, the representative types, and high-level operations in domain vocabulary. Mechanisms move into children once they acquire their own state, protocol, or collaborators.

- The service method is the only door: `judge.invoke(request)`, never a caller reaching for `invoke::run(&judge.client, ...)`.
- Construction of child concepts happens in the parent that owns them; the executable entrypoint connects major aggregates only.
- Prefer concrete types until a real substitution boundary exists (see the structure razor in `SKILL.md`).

## Shape of flow

- Prefer an explicit loop generator over a combinator chain when termination is part of the meaning: an `async_stream` generator with a visible `break` on the completion frame beats `try_unfold` + `filter_map` whose end condition hides inside a `None`. Combinators are for per-item mapping; loops are for protocol control flow.
- `let .. else` for compute-or-bail bindings; `if let` for optional side effects; `match` for exhaustive dispatch.
- Struct literals whose fields are all mandatory stay literals — do not write a constructor that merely relocates the same nine arguments.
- Builders are earned only by construction with modes or invariants that callers must not hand-assemble (preset entry points encoding a safety rule); plain aggregates use literals or `new`.
- Write narrowing conversions with a visible clamp, never a bare `as` that can silently truncate.
- Guard early, return early; a blank line follows each guard before the main work resumes.

## Names in Rust

- Modules are worlds, functions are yields: `stream::payload(response)`, `stream::sse(payloads)`.
- One-word methods where the receiver disambiguates: `run`, `read`, `parse`, `status`. Longer names only when units, direction, or protocol meaning would be lost.
- `try_run` is the `Result` core behind an infallible `run` shell.
- Units in numeric boundary names when confusion is plausible: `duration_ms`, `memory_limit_bytes`.
- Name loop variables and closure parameters semantically (`col`, `previous`, `|left, right|`), never `i` or `|a, b|`.
- Generic parameters carry their trait's name (`UART: Uart`); state mutators take `mark_`/`set_`; getters stay bare nouns.

## Errors in Rust

- Contract crates (models, wire types): thiserror enums; a new variant only for a new handling decision; messages are plain strings inside variants.
- Plumbing, services, and background tasks: anyhow end to end. `.context("Recorded reason.")` at each meaning boundary; the sink logs the chain once with `{error:#}` and records `error.to_string()` (outermost reason only). No per-site `inspect_err`/`map_err` ladders.
- Expected absence: `find(...)?.ok_or(...)` — absence is a value, not a `NotFound` translated later. `get` is for must-exist lookups.
- Detached tokio tasks: `pub async fn run(self, ..)` swallows into one sink; `async fn try_run(&self, ..) -> Result<()>` flows with `?`. A `JoinHandle` nobody joins swallows errors silently — never rely on it.
- Shape each `error.rs` the same way: `pub type Result<T> = ...;` colocated with the enum; wrap an underlying error as a transparent `#[from]` variant; skip a local enum entirely when the module adds no decision over the underlying error.

## Constants

Name every magic number. Timing and tuning constants sit at the top of the using file, typed as what they represent (`const KEEP_ALIVE: Duration = Duration::from_secs(15);`). Wire constants — addresses, control bytes, frame layouts — get a dedicated `command`/`protocol`/`params` module. Compute derived constants from their sources; group digits with underscores.

## Concurrency

Async marks genuine waiting. `tokio::spawn` appears at the assembly point so task lifetime is visible where the system is wired; the spawned future returns `()` and handles its own failures. Join tasks only when they form one logical operation. Keep timeouts explicit and domain-readable.

## Source shape

Follow `rustfmt`; within it, keep one vertical rhythm:

- One packed import block, no blank lines inside, one `use` per line, `pub use` interleaved alphabetically.
- File order: imports/re-exports → `mod` declarations → constants → representative public types top-down (stepdown rule: each supporting type appears just after the type that references it) → private helpers → tests.
- Pack single-line statements; one blank line around every multi-line construct; one blank line between `impl` methods; struct fields and enum variants packed even with doc comments.
- When the formatter would wrap a line awkwardly, restructure the line — extract a named intermediate, split semantic units — instead of accepting the wrap.
- Keep harmless duplication that preserves symmetry between siblings; abstract only when the abstraction has one honest name and a stable shared rule.

## Document in Korean, sparsely

`///` doc comments in Korean, single declarative sentence ending in 다, technical nouns in English: `/// 진행 event를 중계하고 최종 판정을 뽑는다.`

- Module `//!` charters state what code cannot show: an invariant, a deliberate absence ("별도 watchdog을 두지 않는다"), a boundary contract, a cancellation-safety guarantee.
- Document public types, public functions, and fields whose meaning is not carried by name and unit. Leave private helpers bare when name and body suffice.
- Never document what the next line already says; never leave TODOs, banners, or commented-out code.
- Route handlers carry one behavior sentence; never restate the method or path that the router and file tree already declare.

## Tests in Rust

- Boundary tests assert exact edges (`expires_at == now` rejects; the entrance boundary admits).
- Protocol tests feed adversarial input: oversized payloads split at awkward chunk boundaries, mid-prefix, mid-token.
- Offline first: fixtures may assemble real clients that never perform I/O (test credentials, `capture_request`-style harnesses, `#[cfg(test)]` stub constructors).
- Test names state the invariant: `scored_requests_never_return_test_io`, `keeps_an_overlapping_group_alive_after_an_exclusive_failure`.
- Do not port a test whose subject became structurally guaranteed; note the retirement in the report instead.

## Verify Rust changes

Run the narrowest applicable commands: `cargo fmt --check`, `cargo clippy` on the touched crates, focused tests, then the workspace suite when the change crosses crates. Unit tests live beside pure judgments — parsing, scoring, state transitions, validators. State what ran and what requires an environment you do not have.
