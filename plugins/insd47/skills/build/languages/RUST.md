# Rust

Apply with `SKILL.md` when Rust files are in scope. The maintainer's older Rust code does not yet follow every rule here; code this change touches takes the shape described here rather than the legacy shape, but that never widens the change to untouched code.

## Files and hierarchy

- Every file follows one of the shapes under File shapes, and its name says which world it holds.
- A folder module is `foo/mod.rs`, so one folder holds one whole module; a leaf stays a flat `foo.rs`. `mod.rs` holds declarations, selective re-exports, and the module's representative items.
- A module used by only one parent lives under that parent. The crate root keeps only what several modules share.
- One crate sits flat at the repository root; a feature flag beats splitting off a `-build` crate. The README replaces `examples/`.

## Visibility: plain `pub` behind private ancestors

Use the module tree as the access-control mechanism. `mod child;` is private; a `pub` item inside it is a subtree contract sealed by the private ancestor. `pub use` at the parent adopts a concept into the parent's vocabulary — a library re-exports its entry types at the root (`pub use client::{Builder, Client};`), while an application never lifts types to the crate root; `pub mod` when callers deliberately enter the subdomain — they speak the child's path (`infra::judge::JudgeStream`) and import several of its items — and then it beats item-by-item re-export. For boundaries created or restructured by this change, use private ancestors and deliberate re-exports instead of `pub(crate)` or `pub(super)` in production code. Reaching for a distance modifier means the tree is wrong: the item sits too high or its user sits outside the domain that owns it — move the logic under that domain instead. This preference does not expand the cleanup scope. Tests need no exception: a `mod tests` child already sees its parent's private items, so a test lives beside what it tests instead of widening visibility.

## Facades

A parent module holds child declarations, selective re-exports, representative types, and high-level operations in domain vocabulary; a mechanism moves into a child once it owns state, a protocol, or collaborators. The service method is the only door: `judge.invoke(request)`, never `invoke::run(&judge.client, ...)`. Construct child concepts in the owning parent; the entrypoint connects major aggregates only. Prefer concrete types until a real substitution boundary exists.

## Composition

Independent components compose through streams in the caller. A request is a builder implementing `IntoFuture`; a result stream is a named opaque type implementing `Stream`; a provider trait returns `impl Future + Send` rather than boxing into `dyn`:

```rust
let runner = Runner::new().await?;
let checker = Checker::new();
let scorer = Scorer::new(&policy);

let scores = scorer.score(checker.check(runner.run(source).cases(cases).await?, &assets));
```

## File shapes

A reader recognizes a file by its pattern before reading a line, so each file commits to one shape and never mixes in another. The common failure this prevents is a stray free function wedged into a type or handler file.

- **Type file** — one principal type and its `impl`, named after it (`runner.rs` → `Runner`). The principal may be a struct, an enum, or a trait with the function that selects its implementation (`trait Compiler` + `from_language`). Small data models the principal type owns may sit beside it, as may private serde shapes used only here (`Marker`, `Payload`). Operations on the type are methods or associated functions (`Resources::load()`, not `sst::resources::<Resources>()`); no free functions. A computation that would otherwise be a free function returning its own result type becomes that type, constructed with its inputs and run by a one-word verb: `Evaluation::new(groups, results).run()`, not `evaluate(groups, results) -> Evaluation`. Private API hides by keeping the whole file behind a private module.
- **Vocabulary file** — several peer serialized shapes with no principal among them and no inherent `impl` (`types.rs`, `event.rs`): a wire or storage contract read as one vocabulary.
- **Extension file** — `impl` blocks extending a type declared elsewhere, one concern per file: a split inherent impl (`storage/presign.rs` → `impl Storage`) or a trait impl kept apart by the orphan rule or by ownership (`impl IntoResponse for Error` under `http/response/`).
- **Handler file** — a set of sibling handlers sharing one convention: route verbs (`pub async fn add()`, `pub async fn delete()`) or socket events (`pub async fn notice()`, `pub async fn message()`). Only handlers; helpers live in the modules they assemble, and request and response shapes in the model layer.
- **Function module** — wiring and small units with no principal type: `pub fn init`, `fn handler`, `fn auth`, or pure utilities. Free functions are the clean shape here; do not invent a type to host them. Callers speak the module path (`auth::verify(token)`).

When an operation belongs to a domain type, it is a method. When the missing verb belongs to a foreign type, a small internal extension trait supplies it so the chain stays readable (`result.status(StatusCode::NOT_FOUND)?`). APIs read as method chains along the resource path: `contest.contests().id(id).notices().get()`.

Prefer a derive to a hand-written trait impl when callers gain the derive's ergonomics, and let the borrowed crate do its whole job: a derive that only marks the type composes with `#[serde(rename_all = "PascalCase")]` instead of re-implementing serde's name mapping. Omit derives nobody uses (`Debug` included).

## Shape of flow

- An explicit loop beats a combinator chain when termination is part of the meaning — a generator with a visible `break` on the completion frame over `try_unfold` whose end hides in a `None`. Combinators map per item; loops carry protocol control flow.
- `let .. else` for compute-or-bail bindings; `if let` for optional side effects; `match` for exhaustive dispatch.
- Typed error translation picks its shape by coverage: one exceptional variant → borrow-match it (`if let Err(Variant(_)) = &result { return …; }`) and let `?` carry the rest; every variant translated → `map_err` with a `match` inside. Never bend `let .. else` into error translation — its `else` block cannot bind the failure payload.
- All-mandatory struct literals stay literals — no constructor that relocates the same arguments. Builders are earned by optional overrides (`.stage("dev")`), modes, or invariants callers must not hand-assemble.
- Narrowing conversions show a visible clamp, never a bare `as` that can truncate.
- Guard early, return early; a blank line follows each guard.

## Names

- One-word operations where the receiver or module disambiguates (`ctx.index.sync(&name)`, `auth::verify(token)`); `try_run` is the `Result` core behind an infallible `run` shell.
- Semantic loop and closure names (`col`, `previous`, `|left, right|`), never `i` or `|a, b|`.
- Generic parameters carry their trait's name (`UART: Uart`); mutators take `mark_`/`set_`; getters stay bare nouns.

## Errors

- Every crate carries failures as `anyhow::Error`; what varies is whether an edge classifies them. Internal tools with no consumer that needs errors by NAME (translation keys, wire codes, caller branching) run anyhow end to end. A surface with named consumers adds a classification at its edge — never a parallel error hierarchy that absorbs foreign types, which cuts the chain and becomes a write-only registry.
- The classification is one flat enum per contract surface (`ErrorCode`): the variant name is the wire/i18n key (strum-derived), parameters ride in the variant (`FileTooLarge(u64)`), and a variant exists only when the consumer renders or acts differently on it. The producer edge owns the exhaustive status/severity `match`.
- The edge error is `struct Error { code: Option<ErrorCode>, cause: anyhow::Error }` (or `status: StatusCode` when the only consumer reads HTTP status) with one blanket `impl<E: Into<anyhow::Error>> From<E> for Error` that classifies once, at construction, by downcasting the chain (`keygrip::Error::NotFound` → `NotFound`). Classifying at construction keeps the code alive through any later `.context`; a new local meaning is tagged at the failure site (`.context(ErrorCode::Forbidden)`, `.status(StatusCode::FORBIDDEN)`) instead of a new adapter. The edge error deliberately does not implement `std::error::Error` — the blanket `From` would otherwise overlap the reflexive `From<T> for T`. For the same reason a thiserror enum with `#[error(transparent)] Other(#[from] anyhow::Error)` cannot also take foreign errors through `?`. The core is about fifty lines and is duplicated per edge crate on purpose: orphan rules forbid implementing `IntoResponse` or `Serialize` for a shared type.
- thiserror stays for named domain types a caller matches on (`BuildError::Budget`) and for libraries' public errors, marked `#[non_exhaustive]`.
- Open each error-handling file with the regime's vocabulary — `use crate::error::{Error, Result}` or `use anyhow::Result` — so bare `Result<T>` is the default spelling.
- No decoration ladders: an annotation that restates what the source error already says is noise — delete it, the source survives in the chain. Identifying data the source cannot name (which env var, which resource) belongs in a variant parameter — or, under anyhow, in the one annotation that carries it. A conversion moves an error between vocabularies, never logs — logging happens once at the sink.
- Spend as few lines as possible on failure. Under anyhow, an `Option` becomes a failure through `.context("…")` — not `ok_or_else(|| anyhow!(…))` — and a `bool` guard joins the same chain through `then_some(())`. A `Result` takes `.context` only under the decoration rule above (a user-facing explanation or identifying data the source cannot name); otherwise `?` carries it unchanged, and `map_err` is left for typed translation:

  ```rust
  // routes/registry/yank.rs — `status` is the HTTP crate's extension trait (see HTTP services)
  claims.scope.yank.then_some(()).context("yank은 사람 토큰으로만 할 수 있습니다.").status(StatusCode::FORBIDDEN)?;
  let record = ctx.crates.find(&name).await?.context("없는 crate입니다.").status(StatusCode::NOT_FOUND)?;
  ```

- Expected absence is a value: services return `Option` from `find`; `get` is for must-exist lookups.
- Detached tokio tasks: `pub async fn run(self, ..)` swallows into one sink; `async fn try_run(&self, ..) -> Result<()>` flows with `?`. A `JoinHandle` nobody joins swallows errors silently — never rely on it.

## Constants

Keep constants few. Contract and configuration values — region, application name, resource names — are inferred from credentials or linked resources, since each hard-coded copy can drift from the infrastructure it names. What remains gets a name: protocol facts (hosts, control bytes) and tuning values. Timing and tuning constants sit at the top of the using file, typed as what they represent (`const KEEP_ALIVE: Duration = Duration::from_secs(15);`). Wire constants — addresses, control bytes, frame layouts — get a dedicated `command`/`protocol`/`params` module. Compute derived constants from their sources; group digits with underscores.

## Concurrency

Async marks genuine waiting. `tokio::spawn` appears at the assembly point so task lifetime is visible where the system is wired; the spawned future returns `()` and handles its own failures. Join tasks only when they form one logical operation. Timeouts explicit and domain-readable.

## Lints

Enforce what a tool can check instead of relying on review. Declare in the workspace (or single crate) `Cargo.toml`:

```toml
[lints.clippy]
self_named_module_files = "warn"   # folder modules use foo/mod.rs
redundant_pub_crate = "warn"       # no pub(crate) inside a private module
```

## Source shape

Follow `rustfmt`; within it:

- One packed import block, no blank lines inside, `pub use` interleaved alphabetically.
- Module-granularity imports: one `use` per parent module path; braces hold only items directly under that path (`use tokio::sync::{Mutex, mpsc, oneshot};`). Never nest braces or put `::` inside them — `use tokio::{io::AsyncBufReadExt, sync::mpsc};` splits into one line per module. IDEs fold the import block, so vertical length is free; flat lines keep diffs, grep, and merges clean.
- File order: imports/re-exports → `mod` declarations → constants → representative public types top-down (each supporting type just after the type that references it) → private helpers → tests.
- Pack single-line statements; one blank line around every multi-line construct and between `impl` methods; struct fields and enum variants packed even with doc comments.
- Bodies read as paragraphs: statements group by sub-goal — validate, load, decide, apply, respond — with one blank line between groups, and a guard's early return closes its paragraph. Never emit a wall of packed statements spanning multiple sub-goals.
- rustfmt posture: `max_width = 120` and nothing else — short calls inline, long chains vertical, small struct literals free to go multiline. When the formatter would wrap a line awkwardly, restructure the line — extract a named intermediate — instead of accepting the wrap or widening limits. A statement the formatter split across lines gets a blank line on each side, so the paragraph rhythm stays readable after wrapping.
- Platform-split imports live inside the `#[cfg]` block that uses them; collapse per-platform configuration into one block per function so the cfg seam is a single visible joint, not scattered top-level `#[cfg] use` lines.
- `#[cfg]` attaches to a declaration, a binding (`let x = { … };`), an expression statement, or a module/function — never to a naked `{ … }` whose only job is grouping. A platform region that wants a block either earns it as a binding or moves into a function owned by the platform-seam module.
- Bodies reuse imported paths: when a parent module is already in scope, extend it (`commands::auth::login`) instead of restating an absolute `crate::…` path — macro arguments included.
- Keep harmless duplication that preserves symmetry between siblings; abstract only when the abstraction has one honest name and a stable shared rule.

## HTTP services

- The route tree mirrors the URL: every path segment is a folder, an item segment is uniformly `id/`, and its parameter is named after the parent collection (`problems/id/` → `problem_id`). A leaf folder's `mod.rs` holds only verb handlers (`get`, `post`, `put`, `delete`) with the whole flow inline — queries, branches, and response assembly visible. `router()` exists only at nesting points, where the ancestor mounts its leaves' verbs.
- A route folder may hold other files only for fragments: a pure validation cluster (`validate.rs`), a piece several verbs share, or a variant-dispatch target. A file that holds one caller's entire flow is a shadow controller; its flow belongs back in the verb. Request and response shapes live in the model layer, not beside the verbs.
- Exception: when an external protocol fixes the paths and the API is small (a Cargo registry), one flat `router()` lists the protocol's literal paths and files are named after the resource.
- Shared HTTP parts live under `http/` — extractors, middleware, response and error conversion. Domain logic lives in its own modules and returns `bool`, `Option`, or `anyhow::Result`; the handler attaches the HTTP status at the point of failure.
- Handlers take `ctx: Context`, where `pub type Context = axum::extract::State<State>;`. Path parameters arrive through typed extractors with exact field names (`ProblemId { problem_id }`, `Release { name, version }`) that reach state through `FromRef`, so extractors never know the route tree.
- The binary only wires the runtime: `lambda_http::run(router().await?)`. A non-HTTP consumer is one top-level library type with `init` and `handle`, not a route.

## Documentation

Libraries document their public surface; application code does not carry comments the flow already explains, keeping only facts the code cannot show (an external wire format, a deliberate absence). `///` doc comments in Korean, one declarative sentence ending in 다, technical nouns in English: `/// 진행 event를 중계하고 최종 판정을 뽑는다.` Korean particles attach to English or code tokens without a space (`event를`, `` `Status`로 ``). Module `//!` charters state what code cannot show: an invariant, a deliberate absence ("별도 watchdog을 두지 않는다"), a boundary contract, a cancellation-safety guarantee. In libraries, document public types, public functions, and fields whose meaning names and units do not carry; leave private items bare. Never restate the next line; no TODOs, banners, or commented-out code. Route handlers carry one behavior sentence, never the method or path the router already declares.

## Tests

Boundary tests assert exact edges (`expires_at == now` rejects; the entrance boundary admits). Protocol tests feed adversarial input: oversized payloads split at awkward chunk boundaries, mid-prefix, mid-token. Offline first — fixtures may assemble real clients that never perform I/O (test credentials, `capture_request` harnesses, `#[cfg(test)]` stub constructors). Tests sit at the bottom of the file (`mod tests`) or in a sibling `tests.rs` the module declares; either is fine. Sample data stays inline in the test module (`json!`) rather than in a `tests/fixtures/` tree. Test names state the invariant (`scored_requests_never_return_test_io`). Use the shared test-retirement rule in `SKILL.md` and explain why a retired test no longer adds coverage.

## Verify

Follow the verification rule in `SKILL.md`: `cargo fmt --check`, clippy on touched crates, and focused tests as the risk calls for; cross-crate behavior may need the workspace suite. Unit tests live beside pure judgments — parsing, scoring, state transitions, validators.
