# Rust

Apply with `SKILL.md` when Rust files are in scope.

## Visibility: plain `pub` behind private ancestors

The module tree is the entire access-control mechanism. `mod child;` is private; a `pub` item inside it is a subtree contract sealed by the private ancestor. `pub use` at the parent adopts a concept into the parent's vocabulary; `pub mod` when callers deliberately enter the subdomain — they speak the child's path (`infra::judge::JudgeStream`) and import several of its items — and then it beats item-by-item re-export. Never `pub(crate)` or `pub(super)` in production code: needing one means a misdrawn boundary — reshape the tree or re-export at the honest ancestor, and treat existing ones as in-scope cleanup. Single exception: `#[cfg(test)]` seams may use `pub(super)`, where the distance modifier truthfully says "tests in this subtree only".

## Facades

A parent module holds child declarations, selective re-exports, representative types, and high-level operations in domain vocabulary; a mechanism moves into a child once it owns state, a protocol, or collaborators. The service method is the only door: `judge.invoke(request)`, never `invoke::run(&judge.client, ...)`. Construct child concepts in the owning parent; the entrypoint connects major aggregates only. Prefer concrete types until a real substitution boundary exists.

## Shape of flow

- An explicit loop beats a combinator chain when termination is part of the meaning — a generator with a visible `break` on the completion frame over `try_unfold` whose end hides in a `None`. Combinators map per item; loops carry protocol control flow.
- `let .. else` for compute-or-bail bindings; `if let` for optional side effects; `match` for exhaustive dispatch.
- All-mandatory struct literals stay literals — no constructor that relocates the same arguments. Builders are earned only by modes or invariants callers must not hand-assemble.
- Narrowing conversions show a visible clamp, never a bare `as` that can truncate.
- Guard early, return early; a blank line follows each guard.

## Names

- Modules are worlds, functions are yields: `stream::payload(response)`, `stream::sse(payloads)`.
- One-word methods where the receiver disambiguates (`run`, `read`, `parse`); `try_run` is the `Result` core behind an infallible `run` shell.
- Units in numeric boundary names when confusion is plausible (`duration_ms`, `memory_limit_bytes`).
- Semantic loop and closure names (`col`, `previous`, `|left, right|`), never `i` or `|a, b|`.
- Generic parameters carry their trait's name (`UART: Uart`); mutators take `mark_`/`set_`; getters stay bare nouns.

## Errors

- One error vocabulary per crate: anyhow end to end by default; never a second typed layer alongside it in the same crate.
- Open each error-handling file with `use anyhow::{Error, Result}` (drop `Error` when unnamed) so bare `Result<T>` is the default vocabulary. A crate-global error module — typically a thiserror enum layering anyhow beneath — takes precedence: `use crate::error::{Error, Result}`, used the same bare way.
- A crate earns a global error module only when callers branch on variants (contract crates, wire types): thiserror enum, a new variant only for a new handling decision, plain-string messages, `pub type Result<T>` colocated, transparent `#[from]` for wrapped sources. No caller branches → no module.
- `.context("Recorded reason.")` at each meaning boundary; the sink logs the chain once with `{error:#}` and records `error.to_string()`. No per-site `inspect_err`/`map_err` ladders.
- Expected absence: `find(...)?.ok_or(...)` — absence is a value; `get` is for must-exist lookups.
- Detached tokio tasks: `pub async fn run(self, ..)` swallows into one sink; `async fn try_run(&self, ..) -> Result<()>` flows with `?`. A `JoinHandle` nobody joins swallows errors silently — never rely on it.

## Constants

Name every magic number. Timing and tuning constants sit at the top of the using file, typed as what they represent (`const KEEP_ALIVE: Duration = Duration::from_secs(15);`). Wire constants — addresses, control bytes, frame layouts — get a dedicated `command`/`protocol`/`params` module. Compute derived constants from their sources; group digits with underscores.

## Concurrency

Async marks genuine waiting. `tokio::spawn` appears at the assembly point so task lifetime is visible where the system is wired; the spawned future returns `()` and handles its own failures. Join tasks only when they form one logical operation. Timeouts explicit and domain-readable.

## Source shape

Follow `rustfmt`; within it:

- One packed import block, no blank lines inside, `pub use` interleaved alphabetically.
- Module-granularity imports: one `use` per parent module path; braces hold only items directly under that path (`use tokio::sync::{Mutex, mpsc, oneshot};`). Never nest braces or put `::` inside them — `use tokio::{io::AsyncBufReadExt, sync::mpsc};` splits into one line per module. IDEs fold the import block, so vertical length is free; flat lines keep diffs, grep, and merges clean.
- File order: imports/re-exports → `mod` declarations → constants → representative public types top-down (each supporting type just after the type that references it) → private helpers → tests.
- Pack single-line statements; one blank line around every multi-line construct and between `impl` methods; struct fields and enum variants packed even with doc comments.
- When the formatter would wrap a line awkwardly, restructure the line — extract a named intermediate — instead of accepting the wrap.
- Keep harmless duplication that preserves symmetry between siblings; abstract only when the abstraction has one honest name and a stable shared rule.

## Documentation

`///` doc comments in Korean, one declarative sentence ending in 다, technical nouns in English: `/// 진행 event를 중계하고 최종 판정을 뽑는다.` Module `//!` charters state what code cannot show: an invariant, a deliberate absence ("별도 watchdog을 두지 않는다"), a boundary contract, a cancellation-safety guarantee. Document public types, public functions, and fields whose meaning names and units do not carry; leave private helpers bare. Never restate the next line; no TODOs, banners, or commented-out code. Route handlers carry one behavior sentence, never the method or path the router already declares.

## Tests

Boundary tests assert exact edges (`expires_at == now` rejects; the entrance boundary admits). Protocol tests feed adversarial input: oversized payloads split at awkward chunk boundaries, mid-prefix, mid-token. Offline first — fixtures may assemble real clients that never perform I/O (test credentials, `capture_request` harnesses, `#[cfg(test)]` stub constructors). Test names state the invariant (`scored_requests_never_return_test_io`). Retire a test whose subject became structurally guaranteed; note the retirement in the report.

## Verify

Run the narrowest applicable commands: `cargo fmt --check`, clippy on the touched crates, focused tests, then the workspace suite when the change crosses crates. Unit tests live beside pure judgments — parsing, scoring, state transitions, validators. State what ran and what requires an environment you do not have.
