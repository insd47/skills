---
name: build
description: Build, modify, or refactor software in Insung Hwang's personal engineering style. Use for implementation work that benefits from intent-expressing structure, earned abstractions, data-over-behavior boundaries, honest visibility, top-down readable assembly, proportional error models, and invariant-first tests. Applies to new features, bug fixes, refactors, architecture changes, and code generation in any language.
---

# Build Software

## Code is a statement of intent

The reader — human or model — will not ask you questions; the artifact must answer them. Every structure is a signal: a trait says "substitution exists", a public item says "outside callers exist", an `Option` says "absence is expected", a test says "someone relies on this". A structure whose signal is false is a defect even when the code runs.

Audit every structure with one question: **what does this buy?** If the honest answer is only syntax, symmetry, or an imagined future, remove it. Four derived values break ties:

- **Reading cost** — assembly points tell the whole story top-down.
- **Earned structure** — every structure has a present-day payer.
- **Honest vocabulary** — names, paths, and visibility never lie.
- **Evidence** — usage claims ("only caller", "unused") are proven by search, never by intuition.

Because you will not be asked follow-up questions: when evidence cannot resolve intent, choose the reversible option and make your reasoning auditable afterward. State assumptions and rejected alternatives in the handoff report — a recorded rejection ("X was considered, rejected because Y") is what stops the next agent from "fixing" a deliberate choice.

## Instruction priority

1. The user's explicit request and repository-local instructions.
2. Behavioral correctness, safety, and real domain constraints.
3. This document. On apparent conflict, the intent statement above decides; if still tied, the option with fewer structures wins.
4. Language and framework conventions.

## Load language guidance selectively

- Rust files in scope → read [`@languages/RUST.md`](languages/RUST.md) completely.
- TypeScript or TSX files in scope → read [`@languages/TYPESCRIPT.md`](languages/TYPESCRIPT.md) completely.
- Several languages → read each matching guide. Do not load a guide for a language that merely exists elsewhere in the repository.

## Work from evidence

Before designing: read repository instructions and manifests; inspect the target module, its parent, and one comparable sibling; trace ownership, construction, and error flow across the affected boundary. Extend the established local pattern instead of inventing a parallel one; distinguish deliberate conventions from unfinished accidents.

- Verify every usage claim with a search before acting on it.
- Do not remove structure you cannot explain — find who built it and why first (Chesterton's fence).
- Prefer the standard library, the framework idiom, and dependencies already in the manifest. Add a new dependency when it replaces a protocol, parser, or algorithm you would otherwise hand-write — hand-rolled protocol framing is a known incident class. Do not add one for a need a few honest lines cover.

## The structure razor

Keep each kind of structure only while its condition holds; otherwise remove it.

- **Trait / interface** — keep: two or more real implementations, a boundary the concrete type cannot cross (foreign crate), or a test seam locking branchy behavior that structure cannot guarantee. Remove: one implementation with one call site — call the concrete type.
- **Named type** — keep: data crossing a boundary with a rule attached (invariant, wire shape, storage shape). Remove: a re-spelling of `Result`/`Option`/tuple, or a parameter bundle for one caller. A two-variant success/reason enum is already `Result<T, Reason>`.
- **Helper function** — keep: two or more callers, or it isolates a pure judgment worth testing. One caller → inline it.
- **Context object** — keep: its fields were being threaded through two or more functions; they become `self`. Remove: it exists only to shorten a signature.
- **Module / file** — keep: own vocabulary, an invariant, or internal collaborators. Never split on length alone; a file of pure delegation is a smell to merge away.
- **Extension trait** — keep: a library serving callers you do not know. Internal plumbing uses module-qualified free functions.

The graveyard rule: when a structure is removed, the intent it carried must survive somewhere honest — a name, a doc line, a test, or a report note.

## Data over behavior at boundaries

Between layers, pass values — results, enums, streams — not behavior. A worker returns its verdict; the caller applies the effects. Inject behavior (trait, callback) only when the receiving layer cannot name the concrete type. A loop that relays and extracts needs no knowledge of persistence: give it a channel, let it return a `Result`.

## Assembly is the story

- Composition points read top-down; a reader learns the topology without opening leaf modules. Each parent assembles only its direct children.
- A service has one front door: callers use its methods, never its extracted internals.
- Detachment (`spawn`, background work) appears at the assembly point, not inside a method — task lifetime is the most operationally important fact in the file.
- When a deep accessor chain repeats inside a function, destructure once at the top.
- Do not introduce `Application`, `Manager`, `Factory`, or containers to hide readable constructor calls.

## Visibility is a claim about callers

Choose the narrowest visibility that is true, using boundary shape — not distance modifiers — as the mechanism:

- private: implementation detail.
- public inside a private parent: a contract shared within that subtree.
- re-export at the parent: the parent adopts the name into its own vocabulary.
- public child module: callers deliberately enter a subdomain. Trigger: caller code naturally speaks the child's path and imports several of its items. Do not avoid public child modules reflexively — a parent re-exporting a child's whole vocabulary item by item is the smell, not the child.

## Names

- Modules name worlds (a protocol, a domain, a resource); functions name what they yield or do. `stream::payload(response)` then `stream::sse(payloads)` reads as "the payload stream, the SSE stream".
- One-word methods where the receiver carries the context.
- `try_` prefixes the `Result`-returning core of an error-swallowing shell (the `try_main` idiom).
- Quantities on wire or storage carry units in the name: `time_ms`, `memory_limit_bytes`.
- Alias rules: never re-alias a project-owned name — fix the name at its declaration instead. Same name in two layers → the owning-module path (`tables::ExecutionState`), not a new local name. One-off foreign common names → qualified path. A long foreign name used repeatedly in one file may take a local alias named for its role (`InvokeWithResponseStreamOutput as Response`).

## Two error regimes

- **Structured** (thiserror-class): for errors that cross a boundary a caller matches on. New variants only for new handling decisions.
- **Chained reasons** (anyhow-class): for errors that will be swallowed or logged. Attach context at each meaning boundary; the outermost context is the recorded reason, the alternate format is the diagnosis chain. Log the chain once at the single sink, not at every site.
- Expected absence is a value (`Option`, find-then-choose), not a not-found error translated later. No error-translation helper functions — translate inline at the one sink that needs it.
- Detached tasks: an infallible shell owns the single failure sink; the `try_` core flows with `?`. Never return `Result` from a future nobody joins.

## Contracts hide before they share

- A client-facing projection may duplicate a storage shape in order to hide fields. Hiding beats reuse across a trust boundary.
- No silent defaults on trust-boundary requests when the only producer sends complete data — a missing field must fail, not quietly become a default.
- Absence must remain visible as absence. Optional values model it in the type (`Option`, `undefined`, a disabled feature); values the artifact's contract requires fail the step that ships the artifact. Never a fabricated stand-in — an empty endpoint, a dummy key — that type-checks as real and ships the failure inside a passing pipeline.
- When two similar paths coexist deliberately, record that intent exactly where a future "fixer" will look.

## Steps declare their inputs

Classify what a pipeline step consumes of each dependency: values (bundling, publish, deploy), types (typechecking), or nothing. A dependency of a kind the step does not consume is structure that bought nothing — a typecheck wrapped in a credentialed shell re-wires the exact dependency the type layer was built to remove.

Commit a generated file only when it is a contract snapshot: a truth the repository does not own (an external spec), or one reviewers gate over time (an OpenAPI dump, an infra type map). Every other derivative gets an explicit generate command and an ignore entry. Wanting to commit a value usually means the injection path has not been found yet.

## Tests are executable intent

Keep a test when it locks an invariant someone relies on, an exact boundary value, a regression with an incident behind it, or branchy behavior that structure cannot guarantee. Delete a test that re-verifies what structure already guarantees — a single guarded compensation, a restated match arm. Prefer offline tests; a fixture may assemble real handles that never perform I/O. Protocol tests include adversarial sizes and awkward split boundaries.

## Verify proportionally, then stop

Discover the repository's own commands; run the narrowest check that could disprove the change first, then expand with risk: formatter, static analysis, focused tests, full tests, build, runtime smoke. Once the identified risk is answered, stop — do not re-run green suites or verify behavior the change cannot touch. State exactly which checks ran and which did not.

## Handoff report

Report: what changed; which structural signals were added or removed and why; checks run and skipped; assumptions made; alternatives rejected and the reason. If a decision could not be grounded in evidence, say so explicitly rather than presenting it as settled.

## Final audit

- Assembly points read top-down without opening leaves.
- Every remaining structure answers "what does this buy?" with something real.
- Visibility, names, and types send only true signals.
- Errors carry their reasons to exactly one sink each.
- Tests lock invariants, not restatements.
- The report makes every non-obvious choice auditable.
