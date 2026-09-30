# Rust

Apply with `SKILL.md` when Rust files are in scope, under its rule on whose conventions apply.

## Files and hierarchy

- Every file follows one of the shapes under File shapes, and its name says which world it holds.
- A folder module is `foo/mod.rs`, so one folder holds one whole module; a leaf stays a flat `foo.rs`. `mod.rs` holds declarations, selective re-exports, and the module's representative items.

## Visibility: plain `pub` behind private ancestors

Use the module tree as the access-control mechanism. `mod child;` is private; a `pub` item inside it is a subtree contract sealed by the private ancestor. `pub use` at the parent adopts a concept into the parent's vocabulary — a library re-exports its entry types at the root (`pub use client::{Builder, Client};`), while an application never lifts types to the crate root; `pub mod` when callers deliberately enter the subdomain — they speak the child's path (`infra::judge::JudgeStream`) and import several of its items — and then it beats item-by-item re-export. When a parent adopts a child's whole vocabulary (a module split into files only for organization, every item public), re-export it with a glob (`pub use request::*;`). A published crate's curated entry surface stays an explicit list, so a new `pub` item in a child never silently becomes public API. For boundaries created or restructured by this change, use private ancestors and deliberate re-exports instead of `pub(crate)` or `pub(super)` in production code. Reaching for a distance modifier means the tree is wrong: the item sits too high or its user sits outside the domain that owns it — move the logic under that domain instead. This preference does not expand the cleanup scope. Tests need no exception: a `mod tests` child already sees its parent's private items, so a test lives beside what it tests instead of widening visibility.

## Facades

A parent module holds child declarations, selective re-exports, representative types, and high-level operations in domain vocabulary; a mechanism moves into a child once it owns state, a protocol, or collaborators. The service method is the only door: `judge.invoke(request)`, never `invoke::run(&judge.client, ...)`. Construct child concepts in the owning parent; the entrypoint connects major aggregates only. Prefer concrete types until a real substitution boundary exists.

## Composition

Independent components compose through streams in the caller. A request is a builder implementing `IntoFuture`; a result stream is a named opaque type implementing `Stream`; a provider trait returns `impl Future + Send` rather than boxing into `dyn`:

```rust
let runner = Runner::new(&config); // SdkConfig the caller loaded
let checker = Checker::new(&wasm)?;
let scorer = Scorer::new(&problem)?;

let runs = runner.run(source, problem.limits).deadline(deadline).problem(&problem, &tests).await?;
let mut scores = scorer.score(checker.check(runs, &tests));
```

## File shapes

A reader recognizes a file by its pattern before reading a line, so each file commits to one shape and never mixes in another. The common failure this prevents is a stray free function wedged into a type or handler file.

- **Type file** — types and their `impl`s, named after the principal type (`runner.rs` → `Runner`). The principal may be a struct, an enum, or a trait with the function that selects its implementation (`trait Compiler` + `from_language`). Several types may share the file when they belong to one flow (a client, its request builder, its result stream), each followed by its own `impl` blocks; so may private serde shapes used only here (`Marker`, `Payload`). What the shape rules out is free functions mixed in with types: operations on a type are methods or associated functions (`Resources::load()`, not `sst::resources::<Resources>()`). A computation that would otherwise be a free function returning its own result type becomes that type, constructed with its inputs and run by a one-word verb: `Evaluation::new(groups, results).run()`, not `evaluate(groups, results) -> Evaluation`. Private API hides by keeping the whole file behind a private module.
- **Vocabulary file** — several peer serialized shapes with no principal among them and no inherent `impl` (`types.rs`, `event.rs`): a wire or storage contract read as one vocabulary.
- **Extension file** — `impl` blocks extending a type declared elsewhere, one concern per file: a split inherent impl (`storage/presign.rs` → `impl Storage`) or a trait impl kept apart by the orphan rule or by ownership (`impl IntoResponse for Error` under `http/response/`).
- **Handler file** — a set of sibling handlers sharing one convention: route verbs (`pub async fn add()`, `pub async fn delete()`) or socket events (`pub async fn notice()`, `pub async fn message()`). Only handlers; helpers live in the modules they assemble, and request and response shapes in the model layer.
- **Function module** — wiring and small units with no principal type: `pub fn init`, `fn handler`, `fn auth`, or pure utilities. Free functions are the clean shape here; do not invent a type to host them. Callers speak the module path (`auth::verify(token)`).

Validation takes whichever form the codebase's conventions make natural. A type whose module already owns an error vocabulary able to express the failure validates itself as a method. When the data model's module opens no error type, as with a plain protocol or data crate, and the failure belongs to the layer that consumes it, do not invent an error type for the model; put the checks in that layer as a function module named `validate`, each function named after what it checks (`validate::problem(&problem)`), returning that layer's error.

When an operation belongs to a domain type, it is a method. When the missing verb belongs to a foreign type, a small internal extension trait supplies it so the chain stays readable (`result.status(StatusCode::NOT_FOUND)?`). APIs read as method chains along the resource path: `contest.contests().id(id).notices().get()`.

Prefer a derive to a hand-written trait impl when callers gain the derive's ergonomics, and let the borrowed crate do its whole job: a derive that only marks the type composes with `#[serde(rename_all = "PascalCase")]` instead of re-implementing serde's name mapping. Omit derives nobody uses (`Debug` included).

## Shape of flow

- An explicit loop beats a combinator chain when termination is part of the meaning — a generator with a visible `break` on the completion frame over `try_unfold` whose end hides in a `None`. Combinators map per item; loops carry protocol control flow.
- `let .. else` for compute-or-bail bindings; `if let` for optional side effects; `match` for exhaustive dispatch.
- Typed error translation picks its shape by coverage: one exceptional variant → borrow-match it (`if let Err(Variant(_)) = &result { return …; }`) and let `?` carry the rest; every variant translated → `map_err` with a `match` inside. Never bend `let .. else` into error translation — its `else` block cannot bind the failure payload.
- All-mandatory struct literals stay literals — no constructor that relocates the same arguments. Builders are earned by optional overrides (`.stage("dev")`), modes, or invariants callers must not hand-assemble.
- Narrowing conversions show a visible clamp, never a bare `as` that can truncate.

## Names

- One-word operations where the receiver or module disambiguates (`ctx.index.sync(&name)`, `auth::verify(token)`); `try_run` is the `Result` core behind an infallible `run` shell.
- Semantic loop and closure names (`col`, `previous`, `|left, right|`), never `i` or `|a, b|`.
- Generic parameters carry their trait's name (`UART: Uart`); mutators take `mark_`/`set_`; getters stay bare nouns.

## Errors

- Packages others depend on define their errors with thiserror, marked `#[non_exhaustive]`, so callers can match on them. Applications use anyhow end to end. Never mix both regimes in one crate.
- Open each error-handling file with the regime's vocabulary — `use crate::error::{Error, Result}` or `use anyhow::Result` — so bare `Result<T>` is the default spelling. A package's error module also exports `pub type Result<T, E = Error> = std::result::Result<T, E>;` and the crate re-exports it beside `Error`; the default parameter keeps signatures with a foreign error type spellable as `Result<T, OtherError>`.
- No decoration ladders: an annotation that restates what the source error already says is noise — delete it, the source survives in the chain. Identifying data the source cannot name (which env var, which resource) belongs in a variant parameter — or, under anyhow, in the one annotation that carries it. A conversion moves an error between vocabularies, never logs — logging happens once at the sink.
- Spend as few lines as possible on failure. Under anyhow, an `Option` becomes a failure through `.context("…")` — not `ok_or_else(|| anyhow!(…))` — and a `bool` guard joins the same chain through `then_some(())`. A `Result` takes `.context` only under the decoration rule above (a user-facing explanation or identifying data the source cannot name); otherwise `?` carries it unchanged, and `map_err` is left for typed translation:

  ```rust
  // routes/registry/yank.rs — `status` is the HTTP crate's extension trait (see HTTP services)
  claims.scope.yank.then_some(()).context("yank은 사람 토큰으로만 할 수 있습니다").status(StatusCode::FORBIDDEN)?;
  let record = ctx.crates.find(&name).await?.context("없는 crate입니다").status(StatusCode::NOT_FOUND)?;
  ```

- Expected absence is a value: services return `Option` from `find`; `get` is for must-exist lookups.
- Detached tokio tasks: `pub async fn run(self, ..)` swallows into one sink; `async fn try_run(&self, ..) -> Result<()>` flows with `?`. A `JoinHandle` nobody joins swallows errors silently — never rely on it.

## Constants

Keep constants few; infer configuration as `SKILL.md` describes. What remains gets a name: protocol facts (hosts, control bytes) and tuning values. Timing and tuning constants sit at the top of the using file, typed as what they represent (`const KEEP_ALIVE: Duration = Duration::from_secs(15);`). Wire constants — addresses, control bytes, frame layouts — get a dedicated `command`/`protocol`/`params` module. Compute derived constants from their sources; group digits with underscores. Group constants by topic with a blank line between groups, and keep each group packed.

## Concurrency

Async marks genuine waiting. A spawned future returns `()` and handles its own failures. Join tasks only when they form one logical operation. Timeouts explicit and domain-readable.

## Lints

Enforce what a tool can check instead of relying on review. Declare in the workspace (or single crate) `Cargo.toml`:

```toml
[lints.clippy]
self_named_module_files = "warn"   # folder modules use foo/mod.rs
redundant_pub_crate = "warn"       # no pub(crate) inside a private module
```

## Source shape

Follow `rustfmt`; within it:

- One packed import block with `use` and `pub use` interleaved alphabetically, and one packed `mod` block with `mod` and `pub mod` interleaved the same way; never separate items of one block by visibility with a blank line.
- Module-granularity imports: one `use` per parent module path; braces hold only items directly under that path (`use tokio::sync::{Mutex, mpsc, oneshot};`). Never nest braces or put `::` inside them — `use tokio::{io::AsyncBufReadExt, sync::mpsc};` splits into one line per module. IDEs fold the import block, so vertical length is free; flat lines keep diffs, grep, and merges clean.
- File order: imports/re-exports → `mod` declarations → constants → types top-down, each immediately followed by its `impl` blocks (inherent first, then trait impls) and each supporting type just after the type that references it → private helpers → tests. A reader looking at fields should not scroll past other types to find the behavior.
- Pack single-line statements; one blank line around every multi-line construct and between `impl` methods; struct fields and enum variants packed even with doc comments.
- rustfmt posture: `max_width = 120` and nothing else — short calls inline, long chains vertical, small struct literals free to go multiline. When the formatter would wrap a line awkwardly, restructure the line — extract a named intermediate — instead of accepting the wrap or widening limits.
- Platform-split imports live inside the `#[cfg]` block that uses them; collapse per-platform configuration into one block per function so the cfg seam is a single visible joint, not scattered top-level `#[cfg] use` lines.
- `#[cfg]` attaches to a declaration, a binding (`let x = { … };`), an expression statement, or a module/function — never to a naked `{ … }` whose only job is grouping. A platform region that wants a block either earns it as a binding or moves into a function owned by the platform-seam module.
- Bodies reuse imported paths: when a parent module is already in scope, extend it (`commands::auth::login`) instead of restating an absolute `crate::…` path — macro arguments included.

## HTTP services

- The route tree mirrors the URL: every path segment is a folder, an item segment is uniformly `id/`, and its parameter is named after the parent collection (`problems/id/` → `problem_id`). A leaf folder's `mod.rs` holds only verb handlers (`get`, `post`, `put`, `delete`) with the whole flow inline — queries, branches, and response assembly visible. `router()` exists only at nesting points, where the ancestor mounts its leaves' verbs.
- A route folder may hold other files only for fragments: a pure validation cluster (`validate.rs`), a piece several verbs share, or a variant-dispatch target. A file that holds one caller's entire flow is a shadow controller; its flow belongs back in the verb. Request and response shapes live in the model layer, not beside the verbs.
- Exception: when an external protocol fixes the paths and the API is small (a Cargo registry), one flat `router()` lists the protocol's literal paths and files are named after the resource.
- Shared HTTP parts live under `http/` — extractors, middleware, response and error conversion. Domain logic lives in its own modules and returns `bool`, `Option`, or `anyhow::Result`; the handler attaches the HTTP status at the point of failure.
- Handlers take `ctx: Context`, where `pub type Context = axum::extract::State<State>;`. Path parameters arrive through typed extractors with exact field names (`ProblemId { problem_id }`, `Release { name, version }`) that reach state through `FromRef`, so extractors never know the route tree.
- The binary only wires the runtime around `router()`.

## Tests

Unit tests live in the file they test, inside `#[cfg(test)] mod tests { … }` at the bottom; a sibling `tests.rs` reads as source in the folder listing. Integration tests stay in the crate's `tests/` directory as Cargo defines it. Boundary tests assert exact edges (`expires_at == now` rejects; the entrance boundary admits). Offline first — fixtures may assemble real clients that never perform I/O (test credentials, `capture_request` harnesses, `#[cfg(test)]` stub constructors). Sample data stays inline in the test module (`json!`) rather than in a `tests/fixtures/` tree. Test names state the invariant (`scored_requests_never_return_test_io`). Use the shared test-retirement rule in `SKILL.md` and explain why a retired test no longer adds coverage.

## Verify

Follow the verification rule in `SKILL.md`: `cargo fmt --check`, clippy on touched crates, and focused tests as the risk calls for; cross-crate behavior may need the workspace suite.
