---
name: build
description: Builds, modifies, and refactors software in Insung Hwang's engineering style - minimal owned surface, earned abstractions, hierarchical modules, honest types and errors, invariant-focused tests. Use when implementing features, fixing bugs, refactoring, or changing architecture in any language, especially Rust and TypeScript.
---

# Build Software

## Scope and initiative

The user's request, scope, and established decisions take precedence over this skill. Follow repository requirements and preserve correctness and domain constraints; these preferences guide the remaining design choices.

Deliver what was asked, at the scope intended. The rules shape the code you touch; they are not a license to clean up or redesign code the change does not need. Make routine, reversible judgment calls yourself and check in only when different readings of the request would lead to materially different work. If the request seems mistaken or a better approach exists, say so in a sentence and continue with the task as asked rather than quietly narrowing, widening, or transforming it.

Existing comments belong to the maintainer, who often edits them by hand. During structural work, move them with their code instead of deleting them, and report the locations of any that became stale.

## The maintainer premise

One architect owns this code's design, and someone else — possibly from another language — must be able to take it over. Every surface that exists — line, type, layer, state, dependency wrapper — is a surface someone must keep alive, and every signal that lies costs the next reader a wrong decision. Therefore: **own as little as possible, and make everything owned tell the truth.**

Prefer, in this order:

1. **Unwritten code.** The language, framework, or an already-present dependency provides the capability → use it as designed. The most efficient code is code that was never written.
2. **Borrowed code.** A proven library owns the problem class — a protocol, parser, or algorithm you would otherwise hand-write → depend on it instead of rebuilding it. The same holds for formats: a standard encoding and framing over a custom one or binary smuggled through base64, so any other implementation can already read it. Cover a small need with a few honest lines, not a new dependency.
3. **Borrowed state.** Keep authoritative state with its owner and use that owner's identifiers and lifecycle. Local state must serve a responsibility this code actually owns, with a clear lifetime; tracking another system's object alone does not earn a parallel ID, registry, or cache. Configuration is state too: infer it from its owner — the credential's region, a linked resource's name — before adding a constant or environment variable, because every duplicated contract value is a future mismatch between systems. Where an override is genuinely needed, expose it on the builder, not in the environment.
4. **Borrowed shape.** What must be owned looks like its surroundings — the stack's standard idiom, and the same structure as its sibling modules in this project. Humans read by directory structure and recognized file patterns, not by string search as an agent does; a familiar shape is documentation they have already read, and a file that breaks its pattern costs them more than any number of lines.
5. **Separated concerns.** Each file and struct knows only what its responsibility requires. Helpers sharing state, vocabulary, or collaborators may belong to a child module; extract it when that responsibility is real. The entry point should narrate the flow by itself.

Code is a statement of intent: the reader will not ask you questions, so the artifact must answer them. Every structure signals — a trait says "substitution exists", a public item says "outside callers exist", an `Option` says "absence is expected", a test says "someone relies on this". A structure whose signal is false is a defect even when the code runs.

Judge structures within the change with one question: **what does this buy?** Syntax or an imagined future earns nothing. Consider the cost of reading, changing, and operating the result; fewer structures wins only when it preserves meaningful boundaries and clarity. Prove usage claims by search.

Record a deliberate, non-obvious choice where a future maintainer would otherwise undo it. Routine decisions do not need a catalog of rejected alternatives.

## Language guides

Read completely for each language actually in scope (not merely present elsewhere in the repo):

- Rust → [`languages/RUST.md`](languages/RUST.md)
- TypeScript / TSX → [`languages/TYPESCRIPT.md`](languages/TYPESCRIPT.md)

## Work from evidence

Read the relevant repository instructions and manifests; inspect the target module and the parent or sibling needed to understand its role. Trace ownership, construction, and error flow across the affected boundary. Extend the established local pattern; distinguish deliberate conventions from unfinished accidents. Explore further when an unresolved question could change the implementation.

- Verify every usage claim ("only caller", "unused") by search before acting on it.
- Do not remove structure you cannot explain — find who built it and why first.

## The structure razor

Use these criteria to justify structures within the requested change. Caller counts are evidence of reuse; also account for the invariant, responsibility, and reading cost a structure carries. Preserve that intent when removing a structure.

- **Trait / interface**: two+ real implementations, a boundary the concrete type cannot cross (foreign crate), or a test seam locking branchy behavior. Use the concrete type when none of these applies.
- **Named type**: data crossing a boundary with a rule attached (invariant, wire shape, storage shape). Re-spelling `Result`/`Option`/tuple or shortening one signature does not earn a type; use `Result<T, Reason>` when it already expresses the success/failure contract.
- **Helper**: shared logic, an isolated pure judgment worth testing, or a named step that makes assembly readable. Inline a one-caller helper when the call adds no meaning or boundary.
- **Context object**: fields that were threaded through two+ functions become `self`. Shortening a signature earns nothing.
- **Module / file**: own vocabulary, invariant, or collaborators. Never split on length; a file of pure delegation merges away. Place each item at the lowest common ancestor of its users: logic only one domain uses lives under that domain, and something two siblings share moves up to their common parent.
- **Extension trait**: a library for callers you do not know, or a one-word verb that keeps a chain on a foreign type readable. Language guides decide how internal plumbing is spelled.

## Split by domain, assemble in the caller

Split an abstraction by domain responsibility into components that do not know each other: each constructor takes its own configuration, never a sibling component, and the caller assembles the flow by passing one component's output to the next. The assembly then reads as the story, and each component stays usable alone.

This is the horizontal half of the module tree. The tree hides each component's internals beneath it, so no outsider reaches in; assembly keeps siblings from depending on each other, which visibility alone cannot stop. The caller that assembles is the common parent, and it wires only its children's front doors — never internals lifted up for the purpose.

Between layers pass values — results, enums, streams — not behavior: a worker returns its verdict, the caller applies effects. A component reports every fact it observes and the caller decides what to show. Inject behavior (trait, callback) only when the receiving layer cannot name the concrete type — for instance, data the caller owns, requested through a provider trait at the moment it is needed.

Progress arrives by push — a stream, a subscription, a notification — rather than by polling; poll only when the source offers nothing else.

## Assembly is the story

- Composition points read top-down; each parent assembles only its direct children.
- A service has one front door — callers use its methods, never extracted internals.
- Detachment (`spawn`, background work) appears at the assembly point: task lifetime is the file's most important operational fact.
- Destructure a repeated deep accessor chain once at the top.
- No `Application`/`Manager`/`Factory` containers around readable constructor calls.

## Visibility claims callers

Choose the narrowest visibility that is true, using boundary shape as the mechanism: private for detail; public inside a private parent for a subtree contract; a parent re-export to adopt a name into its vocabulary; a public child module when callers deliberately speak the child's path and import several of its items — item-by-item re-export of a child's whole vocabulary is the smell, not the public child.

## Names

- Modules name worlds; operations name what they yield or do, in one word where the receiver or module carries the context.
- `try_` marks the `Result` core of an error-swallowing shell.
- Names stay symmetric with their counterparts: a package matches its repository, a binary matches the resource it deploys as, a port mirrors the upstream name it ports.
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

Choose tests that protect relied-on invariants, exact boundary values, incident-backed regressions, or branchy behavior structure cannot guarantee. Avoid tests that merely restate a reversible, low-impact edit. Retire an existing test only when its guarantee is demonstrably structural or covered elsewhere and no distinct regression protection is lost. Tests adapt to the production structure, never the reverse: do not split a router, widen a visibility, or add a seam so a test can reach something — test through the assembled surface instead. Prefer offline tests — a fixture may assemble real handles that never perform I/O. Protocol tests include adversarial sizes and awkward split boundaries.

## Verify proportionally, then stop

Use the repository's own commands and required checks, choosing the narrowest ones that could disprove the change. Formatter, static analysis, tests, build, and runtime smoke are options matched to risk, not a sequence to run on every edit. Once required checks pass, finish; do not re-run green suites.

## Report

Lead with the outcome, then the design choices that matter and what was checked, in concise prose. Distinguish static checks from observed runtime behavior and state material gaps or assumptions.
