# Rust Guidance

Apply this guide together with the parent `SKILL.md` when Rust files are in scope.

## Use the module tree as the visibility system

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

Do not use `pub(crate)` or `pub(super)` as a shortcut. Reshape the module boundary or selectively re-export the item at its nearest honest ancestor. Use broader visibility only when the user explicitly requires a genuinely crate-wide contract.

## Let parents own semantic facades

Keep a parent module focused on:

- child declarations and selective re-exports;
- the representative public struct, enum, trait, or functions;
- construction of direct child concepts;
- high-level operations expressed in domain vocabulary.

Move mechanisms into children once they acquire their own state, protocol, invariant, or collaborators. Keep small private helper functions when they are merely steps of the same responsibility.

Do not mechanically create one file per struct. Keep tightly coupled request/response frames or a cohesive wire-model family together.

## Compose ownership explicitly

Use constructors and aggregates to make ownership transfers visible.

- Put exclusive platform resources and their mapping in one composition module.
- Construct low-level drivers inside the module that owns the hardware or runtime detail.
- Pass completed capabilities into services or policies.
- Let the executable entrypoint connect major aggregates and retain only the resources it directly coordinates.

When construction grows, add an intermediate `init`, `build`, or domain-named composition function in the immediate parent. Avoid a generic service locator or dependency container.

Prefer concrete types until a real substitution boundary exists. Add a trait for actual polymorphism, a stable external contract, or a necessary test seam—not solely to satisfy dependency inversion.

## Choose precise but contextual names

Let modules and types carry nouns; let methods carry concise actions.

Prefer contextual names such as `run`, `show`, `read`, `write`, `parse`, `status`, or `work` when the receiver and module make them unambiguous. Use longer names when units, direction, or protocol meaning would otherwise be lost.

Include units in numeric boundary names when confusion is plausible, such as `duration_ms`, `pulse_bpm`, or `alcohol_mg_l_x1000`.

## Keep public types meaningful

Use structs and enums to encode domain values, protocol states, aggregates, and invariants. Prefer inference for local values.

- Keep fields private when callers should use a stable interpretation.
- Expose narrow accessors with domain meaning.
- Derive standard traits when they serve real use.
- Use type aliases such as `Result<T>` when they simplify a cohesive error domain.
- Avoid configuration structs that only move constructor arguments without clarifying a boundary.

## Scale errors to the component

Give a device, parser, or protocol its own error enum when it has several meaningful local failures. Preserve driver errors transparently when useful.

Use a shared application error when an orchestration module primarily propagates existing failures. Do not create a service-specific error that merely wraps the same variants without adding a handling decision.

Use `?` and conversion traits for straightforward propagation. Recover, retry, warn, or suppress only where enough context exists to make that decision.

## Keep concurrency semantic

Use async only at genuine I/O or workflow boundaries. Join tasks when they form one logical operation or truly need concurrent progress. Do not describe concurrency as a performance optimization without evidence.

Keep timeout values explicit and domain-readable. Separate long-running procedures from state-holding service types when that makes both easier to follow.

## Preserve source shape

Keep the representative public API easy to find. Use a consistent order among sibling modules, commonly:

1. imports and selective re-exports;
2. child module declarations;
3. constants;
4. representative public types and implementations;
5. private helpers and local supporting types;
6. tests.

Follow `rustfmt`. Prefer early returns for invalid states and flat control flow over deeply nested matches.

## Verify Rust changes

Run the narrowest applicable commands available in the repository, typically formatting, `cargo check`, focused tests, Clippy, and the relevant build target.

Add unit tests beside pure parsing, checksums, conversions, state transitions, and invariants. Add integration tests at public boundaries when the environment permits. For embedded code, keep platform-independent logic testable without inventing a large hardware abstraction solely for tests.
