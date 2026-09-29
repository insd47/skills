---
name: build
description: Builds, modifies, and refactors software in Insung Hwang's engineering style - minimal owned surface, earned abstractions, hierarchical modules, honest types and errors, invariant-focused tests. Use when implementing features, fixing bugs, refactoring, or changing architecture in any language, especially Rust and TypeScript.
---

# Build Software

## Whose conventions

Explicit requests from the user come first, then the repository's AGENTS.md or CLAUDE.md. Below that, who maintains the repository decides:

- **The maintainer's own repository**, where they are the primary author in the git history, or a new repository with no conventions yet: this skill is the convention. Code and text you touch take the shape described here even where older material differs, because the older material may predate these rules or come from unreviewed generated changes. That never widens the change to untouched material.
- **A repository someone else maintains**: its conventions win. Match the closest material first, then the module, then the project. Use this skill only for decisions the codebase leaves open, and never restructure existing work toward it.

## Scope and initiative

Preserve correctness and domain constraints; these preferences guide the remaining design choices.

Deliver what was asked, at the scope intended. The rules shape the code you touch; they are not a license to clean up or redesign code the change does not need. Make routine, reversible judgment calls yourself and check in only when different readings of the request would lead to materially different work. If the request seems mistaken or a better approach exists, say so in a sentence and continue with the task as asked rather than quietly narrowing, widening, or transforming it.

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

Keep harmless duplication that preserves symmetry between siblings; abstract only when the abstraction has one honest name and a stable shared rule.

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
- Names stay symmetric with their counterparts: a package matches its repository, a binary matches the resource it deploys as, a port mirrors the upstream name it ports.
- Wire and storage quantities carry units in the name: `time_ms`, `memory_limit_bytes`.
- Never re-alias a project-owned name — fix it at the declaration. Same name in two layers → owning-module path (`tables::ExecutionState`). A long foreign name used repeatedly in one file may take a role-named local alias.

## Two error regimes

- **Structured** errors for a package others depend on, so callers can match on them; new variants only for new handling decisions.
- **Carried reasons** for an application, where errors end at a single sink: preserve the source chain and log it once there; annotate only with data the source cannot name — an annotation that restates its source is noise.
- Expected absence is a value (`Option`, find-then-choose), not a not-found error translated later. Translate inline at the one sink that needs it — no error-translation helpers.

## Contracts hide before they share

- A client-facing projection may duplicate a storage shape to hide fields — hiding beats reuse across a trust boundary.
- No silent defaults on trust-boundary requests when the only producer sends complete data: a missing field fails.
- Absence stays visible as absence: model it in the type (`Option`, `undefined`, a disabled feature), or fail the step that ships the artifact — never a fabricated stand-in that type-checks as real and ships the failure inside a passing pipeline.

## Generated files

Commit a generated file only as a contract snapshot (a truth the repository does not own, or a dump reviewers gate); every other derivative gets a generate command and an ignore entry.

## Tests are executable intent

Choose tests that protect relied-on invariants, exact boundary values, incident-backed regressions, or branchy behavior structure cannot guarantee. Avoid tests that merely restate a reversible, low-impact edit. Retire an existing test only when its guarantee is demonstrably structural or covered elsewhere and no distinct regression protection is lost. Tests adapt to the production structure, never the reverse: do not split a router, widen a visibility, or add a seam so a test can reach something — test through the assembled surface instead. Place and name tests the way the language's ecosystem does. Prefer offline tests — a fixture may assemble real handles that never perform I/O. Protocol tests include adversarial sizes and awkward split boundaries.

## Vertical rhythm

Bodies read as paragraphs: statements group by sub-goal — validate, load, decide, apply, respond — with one blank line between groups. Guard early and return early; a guard's early return closes its paragraph. Pack consecutive single-line statements, and never emit a wall of packed statements spanning several sub-goals.

Formatters (rustfmt, prettier) break long lines into multi-line constructs, and multi-line constructs packed against each other read as a wall. After formatting, re-read the changed code for rhythm: a blank line before and after every construct that now spans several lines, and paragraph breaks between sub-goals.

## Verify proportionally, then stop

Use the repository's own commands and required checks, choosing the narrowest ones that could disprove the change. Formatter, static analysis, tests, build, and runtime smoke are options matched to risk, not a sequence to run on every edit. Once required checks pass, finish; do not re-run green suites.

## Report

Lead with the outcome, then the design choices that matter and what was checked, in concise prose. Distinguish static checks from observed runtime behavior and state material gaps or assumptions.
