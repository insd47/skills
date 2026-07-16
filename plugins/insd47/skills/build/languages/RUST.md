# Rust Guidance

Apply this guide together with the parent `SKILL.md` when Rust files are in scope.

## Use the module tree as the visibility system

**Intent:** Balance — correctly drawn boundaries make plain `pub` sufficient; distance-based visibility is the symptom of a misdrawn boundary.

Prefer ordinary `mod`, `pub`, and selective `pub use` over distance-based visibility modifiers.

```text
device
├── channel
├── command
├── protocol
├── status
└── error
```

Within this tree:

- Keep implementation modules private with `mod child;`.
- Mark an item in a private child `pub` when its parent or sibling descendants need it. The private ancestor still prevents external access.
- Re-export an item from the parent when external callers need that concept.
- Make the child module itself `pub` only when it is a meaningful public subdomain.
- Re-export the representative type at the parent when ordinary callers should not need the specialized path.

For example, keep an internal command at `device::command::Command`, expose a specialized state as `device::Status`, and optionally lift the representative device again as `devices::Device`.

Never use `pub(crate)` or `pub(super)`. A module tree whose responsibilities are separated correctly achieves access control naturally with plain `pub` behind private ancestors; reaching for distance-based visibility is a signal that a boundary is drawn in the wrong place. Reshape the module boundary or selectively re-export the item at its nearest honest ancestor instead. If existing code contains `pub(crate)` or `pub(super)`, treat it as a mistake to clean up when in scope, not a convention to extend — the Boy Scout rule, bounded to the code you already touch.

## Let parents own semantic facades

**Intent:** Flow — a parent tells its subtree's story in domain vocabulary, so callers never need to read the mechanisms below.

Keep a parent module focused on:

- child declarations and selective re-exports;
- the representative public struct, enum, trait, or functions;
- construction of direct child concepts;
- high-level operations expressed in domain vocabulary.

Move mechanisms into children once they acquire their own state, protocol, invariant, or collaborators. Keep small private helper functions when they are merely steps of the same responsibility.

Do not mechanically create one file per struct. Keep tightly coupled request/response frames or a cohesive wire-model family together.

## Compose ownership explicitly

**Intent:** Flow — ownership transfer is the plot of a systems program; constructors and aggregates make it visible.

Use constructors and aggregates to make ownership transfers visible.

- Put exclusive platform resources and their mapping in one composition module.
- Construct low-level drivers inside the module that owns the hardware or runtime detail.
- Pass completed capabilities into services or policies.
- Let the executable entrypoint connect major aggregates and retain only the resources it directly coordinates.

When construction grows, add an intermediate `init`, `build`, or domain-named composition function in the immediate parent. Avoid a generic service locator or dependency container.

Prefer concrete types until a real substitution boundary exists. Add a trait for actual polymorphism, a stable external contract, or a necessary test seam—not solely to satisfy dependency inversion.

## Choose precise but contextual names

**Intent:** Flow — the module and receiver carry context, so names stay short where the context speaks and grow only where meaning would be lost.

Let modules and types carry nouns; let methods carry concise actions.

Prefer contextual names such as `run`, `show`, `read`, `write`, `parse`, `status`, or `work` when the receiver and module make them unambiguous. Use longer names when units, direction, or protocol meaning would otherwise be lost.

Include units in numeric boundary names when confusion is plausible, such as `duration_ms`, `pulse_bpm`, or `alcohol_mg_l_x1000`.

Follow these micro-conventions:

- Suffix representative device and service types with their role (`AlcoholDevice`, `MeasureService`); keep enums and variants as bare domain nouns.
- Name generic parameters after the trait they carry (`UART: Uart`, `I2C: I2c`); use the lifetime `'d` for device-held borrows.
- Prefix state mutators with `mark_` or `set_`; keep getters as bare nouns (`window`, `total_samples`).
- Name loop variables and closure parameters semantically (`col`, `row`, `previous`, `next`, `|left, right|`), not `i` or `|a, b|`.

## Keep public types meaningful

**Intent:** Balance — a public type is a granted responsibility; it must encode a domain value, an invariant, or a contract, not decoration.

Use structs and enums to encode domain values, protocol states, aggregates, and invariants — make illegal states unrepresentable when the domain allows it: an enum over a validated flag pair, a typed unit over a bare integer. Prefer inference for local values.

- Keep fields private when callers should use a stable interpretation; expose narrow accessor methods with domain meaning.
- Reserve fully public fields for composition aggregates and wire or storage model structs.
- Derive standard traits when they serve real use; let tooling order the derive list.
- Construct with `new` returning `Self`, or `Result<Self>` when construction can fail. Do not introduce builders or `try_new` variants.
- Use type aliases such as `Result<T>` when they simplify a cohesive error domain, and local aliases to tame long generic HAL types (`type PpgChannel<'d> = AdcChannelDriver<'d, ...>;`).
- Avoid configuration structs that only move constructor arguments without clarifying a boundary.

## Place constants deliberately

**Intent:** Uniformity — every magic number has exactly one named home, so tuning and protocol facts are found where the domain says they live.

Name every magic number. Keep timing and tuning constants at the top of the file that uses them, typed as what they represent (`const DEBOUNCE: Duration = Duration::from_millis(80);`). Give a device's wire constants — addresses, control bytes, register tables, frame layout — a dedicated `command`, `protocol`, or `params` module.

- Name module constants in SCREAMING_SNAKE, adding a unit token when confusion is plausible (`SAMPLE_PERIOD_MS`, `DEBOUNCE`).
- Compute derived constants from their sources (`const PAGES: usize = HEIGHT / 8;`) instead of restating results.
- Group digits with underscores (`Hertz(9_600)`, `60_000.0`) and use typed literal suffixes where the type is not obvious in context (`0_u8`, `10_f32`).

## Scale errors to the component

**Intent:** Balance — error detail lives where a handling decision is made with it.

Give a device, parser, or protocol its own error enum when it has several meaningful local failures. Preserve driver errors transparently when useful.

Use a shared application error when an orchestration module primarily propagates existing failures. Do not create a service-specific error that merely wraps the same variants without adding a handling decision.

Use `?` and conversion traits for straightforward propagation. Recover, retry, warn, or suppress only where enough context exists to make that decision.

Shape each `error.rs` the same way:

- Colocate `pub type Result<T> = core::result::Result<T, Error>;` with the enum.
- Wrap the underlying driver error as a transparent `#[from]` variant; skip a local enum entirely when a module adds nothing over the driver error.

## Keep concurrency semantic

**Intent:** Flow — async marks genuine waiting, so the reader can trust every `await` to mean the world is being waited on.

Use async only at genuine I/O or workflow boundaries. Join tasks when they form one logical operation or truly need concurrent progress. Do not describe concurrency as a performance optimization without evidence.

Keep timeout values explicit and domain-readable. Separate long-running procedures from state-holding service types when that makes both easier to follow.

## Preserve source shape

**Intent:** Uniformity and beauty — sibling modules share one layout, and the vertical rhythm survives rustfmt unchanged.

Keep the representative public API easy to find. Use a consistent order among sibling modules, commonly:

1. imports and selective re-exports;
2. child module declarations;
3. constants;
4. representative public types and implementations;
5. private helpers and local supporting types;
6. tests.

Within the types section, lead with the top-level type and let its supporting types follow in the order they are first referenced from above, root to leaf. A model file reads like unrolling one abstraction at a time: the event enum first, then the status enum it embeds, then the result struct, then the per-run struct the result is built from. The reader meets each definition just after the type that needs it — never by scrolling up.

Follow `rustfmt`. Within it, apply the vertical rhythm from `SKILL.md` concretely:

- Pack one-line statements together; give every multi-line `if`, `match`, loop, or chained call a blank line above and below.
- Put a blank line after a guard-clause return, and before a trailing `Ok(())` or result expression that follows a block.
- Keep struct fields and enum variants packed with no blank lines, even when each carries a doc comment.
- Separate `impl` methods with exactly one blank line.
- In a `mod.rs`, keep imports and re-exports as one packed block, then a blank line, then the `mod` declarations, then the items.

Prefer early returns and flat control flow over deeply nested matches:

- Use `let .. else` for compute-or-bail bindings.
- Use `if let` for optional side effects and log-and-continue handling.
- Reserve `match` for exhaustive dispatch: single-line arms for byte and glyph tables, block arms only where an arm needs one.
- A symmetric validator may end in `if mismatch { Err(...) } else { Ok(()) }` instead of a guard return.
- Write narrowing numeric conversions with a visible clamp (`value.min(u64::from(u32::MAX)) as u32`), never a bare `as` that can silently truncate.

Write one `use` statement per line, nesting braces only for several items from a single path, sorted alphabetically with `pub use` re-exports interleaved.

## Document in Korean, sparsely

**Intent:** Flow — names and structure carry meaning first; documentation appears only where they cannot.

Write `///` doc comments in Korean, ending in the plain declarative register ("-다"), keeping technical nouns and identifiers in English or backticks: `/// 버튼이 새로 눌린 순간에만 true를 반환한다.`

- Document public device, service, and model types, their public functions, and model fields whose meaning is not obvious from name and unit.
- Leave private helpers undocumented when the name and body carry the meaning.
- Avoid inline `//` comments except provenance notes for ported algorithms (source reference, original formula).
- Do not add section-divider banners, TODOs, or commented-out code.

## Verify Rust changes

**Intent:** Proportionality — the cheapest evidence that could disprove the change, within what the hardware allows; stop once the risk is answered.

Run the narrowest applicable commands available in the repository, typically formatting, `cargo check`, focused tests, Clippy, and the relevant build target.

Add unit tests beside pure parsing, checksums, conversions, state transitions, and invariants. Add integration tests at public boundaries when the environment permits. For embedded code, keep platform-independent logic testable without inventing a large hardware abstraction solely for tests.
