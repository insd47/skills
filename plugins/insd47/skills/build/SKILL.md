---
name: build
description: Build, modify, or refactor software in Insung Hwang's personal engineering style - minimal owned surface, earned abstractions, borrowed state and conventions, honest signals, top-down readable assembly, proportional error models, invariant-first tests. Applies to new features, bug fixes, refactors, architecture changes, and code generation in any language.
---

# Build Software

## The maintainer premise

One person maintains this code, alone. Every surface that exists — line, type, layer, state, dependency wrapper — is a surface they must keep alive, and every signal that lies costs them a wrong decision later. Therefore: **own as little as possible, and make everything owned tell the truth.**

Prefer, in this order:

1. **Unwritten code.** The language, framework, or an already-present dependency provides the capability → use it as designed. The most efficient code is code that was never written.
2. **Borrowed code.** A proven library owns the problem class — a protocol, parser, or algorithm you would otherwise hand-write → depend on it instead of rebuilding it. Cover a small need with a few honest lines, not a new dependency.
3. **Borrowed state.** Hold no state another system already owns. Reference the owner's identifiers and lifecycles; never mint a parallel ID, registry, or cache to track someone else's object. The best state management is no state to manage.
4. **Borrowed shape.** What must be owned looks like its surroundings — the stack's standard idiom, and the same structure as its sibling modules in this project. Humans read entry-point-first, not by string search; a familiar shape is documentation they have already read.
5. **Separated concerns.** Each file and struct knows only what its responsibility requires. Loose private helpers piling up at the top of a module are the symptom of a missing layer — extract the layer and they disappear into it. Done well, the entry point narrates the entire flow by itself.

Code is a statement of intent: the reader will not ask you questions, so the artifact must answer them. Every structure signals — a trait says "substitution exists", a public item says "outside callers exist", an `Option` says "absence is expected", a test says "someone relies on this". A structure whose signal is false is a defect even when the code runs.

Audit every structure with one question: **what does this buy?** If the honest answer is only syntax, symmetry, or an imagined future, remove it. Tie-breakers: top-down reading cost, a present-day payer for every structure, names and visibility that never lie, usage claims proven by search.

When evidence cannot resolve intent, choose the reversible option and record assumptions and rejected alternatives in the report — a recorded rejection stops the next agent from "fixing" a deliberate choice.

## Priority

1. The user's explicit request and repository-local instructions.
2. Behavioral correctness, safety, and real domain constraints.
3. This document — on conflict the maintainer premise decides; still tied, fewer structures wins.
4. Language and framework conventions.

## Language guides

Read completely for each language actually in scope (not merely present elsewhere in the repo):

- Rust → [`languages/RUST.md`](languages/RUST.md)
- TypeScript / TSX → [`languages/TYPESCRIPT.md`](languages/TYPESCRIPT.md)

## Work from evidence

Before designing, read repository instructions and manifests; inspect the target module, its parent, and one comparable sibling; trace ownership, construction, and error flow across the affected boundary. Extend the established local pattern; distinguish deliberate conventions from unfinished accidents.

- Verify every usage claim ("only caller", "unused") by search before acting on it.
- Do not remove structure you cannot explain — find who built it and why first.

## The structure razor

Keep each structure only while its condition holds; otherwise remove it, and let the intent it carried survive somewhere honest — a name, a doc line, a test, or a report note.

- **Trait / interface**: two+ real implementations, a boundary the concrete type cannot cross (foreign crate), or a test seam locking branchy behavior. One implementation with one call site → call the concrete type.
- **Named type**: data crossing a boundary with a rule attached (invariant, wire shape, storage shape). A re-spelling of `Result`/`Option`/tuple or a one-caller parameter bundle is not a type; a success/reason enum is already `Result<T, Reason>`.
- **Helper**: two+ callers, or an isolated pure judgment worth testing. One caller → inline.
- **Context object**: fields that were threaded through two+ functions become `self`. Shortening a signature earns nothing.
- **Module / file**: own vocabulary, invariant, or collaborators. Never split on length; a file of pure delegation merges away.
- **Extension trait**: a library for callers you do not know. Internal plumbing uses module-qualified free functions.

## Boundaries pass data

Between layers pass values — results, enums, streams — not behavior: a worker returns its verdict, the caller applies effects. Inject behavior (trait, callback) only when the receiving layer cannot name the concrete type.

## Assembly is the story

- Composition points read top-down; each parent assembles only its direct children.
- A service has one front door — callers use its methods, never extracted internals.
- Detachment (`spawn`, background work) appears at the assembly point: task lifetime is the file's most important operational fact.
- Destructure a repeated deep accessor chain once at the top.
- No `Application`/`Manager`/`Factory` containers around readable constructor calls.

## Visibility claims callers

Choose the narrowest visibility that is true, using boundary shape as the mechanism: private for detail; public inside a private parent for a subtree contract; a parent re-export to adopt a name into its vocabulary; a public child module when callers deliberately speak the child's path and import several of its items — item-by-item re-export of a child's whole vocabulary is the smell, not the public child.

## Names

- Modules name worlds; functions name what they yield or do — `stream::payload(response)`, then `stream::sse(payloads)`.
- One-word methods where the receiver carries context; `try_` marks the `Result` core of an error-swallowing shell.
- Wire and storage quantities carry units in the name: `time_ms`, `memory_limit_bytes`.
- Never re-alias a project-owned name — fix it at the declaration. Same name in two layers → owning-module path (`tables::ExecutionState`). A long foreign name used repeatedly in one file may take a role-named local alias.

## Two error regimes

- **Structured** (thiserror-class) for errors a caller matches on; new variants only for new handling decisions.
- **Carried reasons** for errors that end at a single sink: preserve the source chain and log it once there; annotate only with data the source cannot name — an annotation that restates its source is noise.
- Expected absence is a value (`Option`, find-then-choose), not a not-found error translated later. Translate inline at the one sink that needs it — no error-translation helpers.
- Detached tasks: an infallible shell owns the single failure sink; the `try_` core flows with `?`. Never return `Result` from a future nobody joins.

## Contracts hide before they share

- A client-facing projection may duplicate a storage shape to hide fields — hiding beats reuse across a trust boundary.
- No silent defaults on trust-boundary requests when the only producer sends complete data: a missing field fails.
- Absence stays visible as absence: model it in the type (`Option`, `undefined`, a disabled feature), or fail the step that ships the artifact — never a fabricated stand-in that type-checks as real and ships the failure inside a passing pipeline.
- When two similar paths coexist deliberately, record that intent exactly where a future "fixer" will look.

## Steps declare their inputs

Classify what each pipeline step consumes of a dependency — values, types, or nothing — and remove dependencies of a kind the step does not consume: a typecheck wrapped in a credentialed shell re-wires the dependency the type layer removed. Commit a generated file only as a contract snapshot (a truth the repository does not own, or a dump reviewers gate); every other derivative gets a generate command and an ignore entry.

## Tests are executable intent

Keep a test that locks an invariant someone relies on, an exact boundary value, an incident-backed regression, or branchy behavior structure cannot guarantee. Delete a test that re-verifies what structure already guarantees. Prefer offline tests — a fixture may assemble real handles that never perform I/O. Protocol tests include adversarial sizes and awkward split boundaries.

## Verify proportionally, then stop

Discover the repository's own commands; run the narrowest check that could disprove the change first, expanding with risk: formatter, static analysis, focused tests, full tests, build, runtime smoke. Stop once the identified risk is answered — do not re-run green suites or verify behavior the change cannot touch.

## Report

State: what changed; structural signals added or removed and why; checks run and skipped; assumptions; alternatives rejected and the reason. Mark any decision not grounded in evidence as such.

## Final audit

- The change owns nothing the language, a library, another system's state, or an existing pattern could have carried.
- Assembly points read top-down without opening leaves.
- Every structure answers "what does this buy?" with something real.
- Visibility, names, and types send only true signals.
- Each error carries its reason to exactly one sink.
- Tests lock invariants, not restatements.
- The report makes every non-obvious choice auditable.
