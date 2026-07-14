---
name: build
description: Build, modify, or refactor software in Insung Hwang's personal engineering style. Use for implementation work that benefits from hierarchical semantic modules, narrow and intentional visibility, simple recursive composition, readable entrypoints, structural symmetry, inferred types, proportional error models, SOLID without ceremony, and YAGNI without flattening meaningful boundaries. Applies to new features, bug fixes, refactors, architecture changes, and code generation in any language.
---

# Build Software

Implement working software whose structure is understandable from its module tree and top-level composition.

## Hold the core values

Every rule in this skill serves one philosophy: SOLID and YAGNI as the structural backbone, in service of clean, beautiful, human-readable code. Concretely, each rule protects one or more of these four named values:

- **Flow** — the level of abstraction is right, so control and meaning read top-to-bottom like a narrative.
- **Balance** — responsibility is distributed where it belongs, so each module owns one coherent concern.
- **Uniformity** — similar problems look similar across the project, so one understanding transfers everywhere.
- **Beauty** — code stays visually clean through the formatter; a forced wrap that makes similar structures look different is a defect to design away, not to accept.

Each section below names its intent. When a rule does not cover your situation, choose the option that best preserves the named intent. When two rules appear to conflict, the intent decides.

## Instruction priority

Apply these rules in order:

1. Follow the user's explicit request and repository-local instructions.
2. Preserve behavioral correctness, safety, and real domain constraints.
3. Preserve the design values in this document.
4. Use SOLID and YAGNI as supporting lenses, never as reasons to violate those values.
5. Follow language and framework conventions where they do not conflict with the above.

Do not invoke SOLID to add ceremonial interfaces, factories, layers, or dependency injection. Do not invoke YAGNI to flatten a meaningful module tree, leak implementation details, or make the composition root unreadable.

## Load language guidance selectively

- When Rust files are in scope, read [`@languages/RUST.md`](languages/RUST.md) completely.
- When TypeScript or TSX files are in scope, read [`@languages/TYPESCRIPT.md`](languages/TYPESCRIPT.md) completely.
- When several languages are in scope, read each matching guide.
- Do not load a language guide merely because that language exists elsewhere in the repository.

## Work from evidence

**Intent:** Uniformity — a change should read as if the codebase grew it, so understanding gained anywhere in the project transfers to the new code.

Before designing or editing:

1. Read repository instructions, manifests, and relevant framework documentation.
2. Inspect the target module, its parent, its children, and at least one comparable sibling.
3. Trace ownership, construction, public imports, error flow, and tests across the affected boundary.
4. Distinguish deliberate local conventions from unfinished code or accidental inconsistencies.
5. State only assumptions that materially affect the design.

Prefer extending an established local pattern over inventing a parallel one. Correct a local inconsistency only when it is in scope or prevents a coherent implementation.

## Design the semantic tree

**Intent:** Balance — the tree is where responsibility gets distributed; a reader should learn what the system is and who owns what from the module tree alone.

Treat the module tree as both a map of meaning and a map of authority.

Create a child module when a part develops one or more of these:

- its own vocabulary or domain concepts;
- an invariant, protocol, or transformation rule;
- state or dependencies that form a coherent responsibility;
- several implementation parts that collaborate internally;
- a reason to change independently from its parent;
- enough detail that it obscures the parent's public story.

Do not split solely because a file is long. Do not keep unrelated responsibilities together solely because each is short.

Let a large module grow a child and grandchild tree recursively. At every level:

- let leaves own mechanisms and local invariants;
- let the parent assemble its direct children;
- let the parent expose a smaller, more semantic capability;
- keep grandchildren out of the parent's caller-facing story.

Aim for one cohesive reason to understand or change a file, not mechanically one type per file. Keep tightly coupled protocol pairs or model families together when separating them would hide their relationship.

## Expose the smallest natural surface

**Intent:** Balance — when boundaries are drawn correctly, access control falls out of plain visibility; a strained visibility trick is a symptom of a misplaced boundary, never a tool.

Keep declarations private by default. Make an item visible only as far as the nearest common ancestor of its real consumers.

Use these meanings:

- A public item inside a private child module is an internal contract shared within that subtree.
- A selective re-export means the parent adopts that item as part of its own vocabulary.
- A public child module means the child is a meaningful subdomain that callers may deliberately enter.
- A root-level export is reserved for a representative concept that ordinary consumers should find immediately.

Do not create catch-all barrel files. Do not re-export every child declaration for convenience. Do not bypass a well-shaped tree with crate-wide or application-wide visibility.

When deciding where an item belongs, ask: "What is the lowest module that can honestly own this name and still serve every real consumer?"

## Compose one level at a time

**Intent:** Flow — each level of composition is a short story about the level below, so the whole system explains itself top-down without opening leaves.

Treat composition roots as recursive, not unique.

Each parent should assemble only concepts from the next semantic level. It may know its direct children, but it should not manually wire their internal descendants.

Keep top-level entrypoints as short, readable stories of the system:

1. initialize the environment;
2. construct major subsystems;
3. connect their ownership and dependencies;
4. start the runtime or perform the task;
5. apply top-level recovery policy.

Centralize declarations that share one exclusive resource map or system-wide contract. Examples include board pins and peripherals, infrastructure resources, route assembly, process wiring, and service ownership.

If assembly becomes complicated, move the coherent portion into the immediately lower module and return an aggregate or capability with a clear name. Do not introduce an `Application`, `Manager`, `Factory`, or container merely to hide a few readable constructor calls.

The top level should reveal system topology and execution order, not low-level configuration.

## Optimize for readable shape

**Intent:** Uniformity and beauty — parallel concepts look parallel, and the shape survives the formatter unchanged.

Treat visual structure as part of maintainability.

- Make sibling modules structurally symmetrical when they represent parallel concepts.
- Keep comparable declarations in comparable positions.
- Prefer top-to-bottom control flow that reads as a narrative.
- Keep the principal operation or export easy to find.
- Accept small, measured runtime or allocation costs when they materially improve clarity and do not threaten actual requirements.

Apply one vertical rhythm in every language:

- Pack consecutive single-line statements and declarations together with no blank lines between them.
- Put one blank line above and below every multi-line construct — a block, a loop, a match or conditional with a body, a multi-line call or literal, a long method chain — except at the start or end of its enclosing block.
- Put a blank line after an early-return guard before the main work resumes.
- When single-line statements accumulate into a long run, split them into logically grouped clusters separated by blank lines.

When the formatter would wrap a long line awkwardly, restructure the line instead of accepting the wrap: extract an intermediate constant with a meaningful name, split arguments into semantic units, or move the expression out of the call site. An ugly forced wrap is a defect in the line, not a formatter setting to fight.

Do not remove harmless duplication if it preserves useful symmetry. Abstract repeated code only when the abstraction has one honest name, reduces conceptual load, and represents a stable shared rule.

## Use types selectively

**Intent:** Flow — a type appears exactly where meaning crosses a boundary, so the reader is neither starved of contracts nor drowned in restated inference.

Prefer inference for local values and implementation details. Introduce a named type when it carries information across a meaningful boundary or protects a real rule, such as:

- a domain value or invariant;
- a wire, storage, or infrastructure contract;
- an aggregate shared by several sibling modules;
- a public result that downstream composition depends on;
- an error distinction callers can act on.

Do not create wrapper types, configuration objects, interfaces, or traits merely to make the design look formal. A type must clarify ownership, constrain invalid states, or stabilize a shared contract.

## Choose the simplest useful error model

**Intent:** Balance — error detail lives exactly where a decision is made with it; anything more is ceremony, anything less erases diagnosis.

Model errors according to the distinctions the current caller needs, not according to a universal layer policy.

- Preserve detailed local errors when a component has several meaningful failure modes that aid diagnosis or recovery.
- Reuse or aggregate errors when an intermediate module mostly forwards failures and adds no new handling decision.
- Convert, log, retry, or suppress an error at the lowest boundary that has enough context to make the complete policy decision.
- Add a new error type only when it introduces meaningful semantics.

Do not force every module to have its own error enum. Do not erase useful device, protocol, validation, or boundary failures into strings for superficial uniformity.

## Apply SOLID without ceremony

**Intent:** SOLID is the skeleton of balance — it guards how responsibility is distributed, and must never manufacture structure that no current code needs.

Use SOLID as questions, not quotas:

- **Single responsibility:** Does this module have one cohesive reason to change? A responsibility may contain several collaborating types and files.
- **Open/closed:** Is there a real, current axis of variation? Add an extension point only when evidence exists.
- **Liskov substitution:** Are contracts truthful and substitutable? Prefer composition over a misleading hierarchy.
- **Interface segregation:** Does each consumer see only the capability it needs? Use module boundaries and narrow exports before inventing interfaces.
- **Dependency inversion:** Do high-level policies depend on domain capabilities rather than low-level mechanics? Concrete dependencies are acceptable inside small modules and composition roots.

Prefer a direct concrete dependency over a one-implementation abstraction. Introduce an interface or trait when there are multiple implementations, an actual replacement boundary, or a test seam that cannot be achieved more simply.

## Apply YAGNI without damaging structure

**Intent:** YAGNI protects flow — it removes imagined variation, but must never buy simplicity by erasing a boundary that carries present-day meaning.

Implement the smallest complete design for the current behavior.

Avoid speculative:

- generic frameworks;
- plugin systems;
- registries and factories;
- configuration switches;
- inheritance or trait hierarchies;
- compatibility paths;
- repositories, services, or DTOs with no current consumer.

YAGNI removes imagined variation, not present-day meaning. Keep a module boundary when it contains real vocabulary, ownership, invariants, or internal collaboration even if it currently has one implementation.

## Implement in a readable order

**Intent:** An order that surfaces each value while it is still cheap to fix — boundaries before code, symmetry before handoff.

1. Identify the affected semantic subtree and its public boundary.
2. Decide which module owns each new behavior and dependency.
3. Decide the nearest common ancestor for each shared declaration.
4. Decide where construction belongs and keep the upper composition simple.
5. Implement the smallest coherent vertical path.
6. Compare the result with sibling modules and restore useful symmetry.
7. Remove speculative abstractions and accidental visibility.
8. Verify behavior and report remaining uncertainty.

## Build a proportional verification harness

**Intent:** Proportionality — this section exists to stop reflexive over-verification; run the cheapest evidence that could disprove the change, expand only as risk demands, and stop once the risk is answered.

Do not wait for the user to prescribe tests. Discover available scripts, existing test conventions, and the cheapest evidence that could disprove the implementation.

Use risk to choose the harness:

- For a pure rule or bug, add a focused unit or regression test.
- For a module boundary, test through its public surface rather than private helpers.
- For serialization, protocols, generated configuration, or infrastructure, verify representative boundary artifacts.
- For UI behavior, verify the rendered interaction when practical; do not rely only on type checking.
- For hardware or unavailable external systems, isolate and test pure transformations, then clearly state what requires real-device verification.

Run the narrowest relevant check first, then expand in proportion to risk: formatter, static analysis, type checking, unit tests, integration tests, build, and runtime smoke checks as applicable.

Do not create elaborate mocks, generic test frameworks, or dependency abstractions for a single trivial assertion. Do not skip verification merely because the repository lacks a ready-made command. State exactly which checks ran and which did not.

Once the identified risk is answered, stop. Do not re-run suites that already passed, re-verify behavior the change cannot have touched, or add checks for their own sake.

## Final audit

Before handing off, confirm:

- The top-level flow is readable without opening leaf modules.
- Each changed module owns a coherent concept.
- Public declarations stop at the lowest honest boundary.
- Parent modules assemble direct children, not grandchildren.
- Similar siblings have similar shapes.
- Types and error distinctions earn their existence.
- SOLID improved the design without adding ceremony.
- YAGNI removed speculation without flattening meaning.
- Verification is proportional to the risk of the change.
- The source shape survives the formatter without awkward wraps.
