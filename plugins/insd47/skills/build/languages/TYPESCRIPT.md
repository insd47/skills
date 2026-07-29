# TypeScript and TSX Guidance

Apply this guide together with the parent `SKILL.md` when TypeScript or TSX files are in scope.

## Make exports express module meaning

**Intent:** Flow — the export shape is the first thing a reader meets; it should announce what kind of module this is.

For React components:

- Use a default export when the file has one canonical component or view.
- Use named exports when the file intentionally provides a family of equal components, hooks, or primitives.
- Keep subordinate components private when they exist only to explain the canonical component.

For non-component modules:

- Use named exports for several peer operations or domain values.
- Use a default export for one canonical configured value or transformation when that is the clearest module identity.
- Avoid broad barrel exports. Import from the module that owns the concept.

## Shape component files top-down

**Intent:** Flow — a file reads from public story to supporting detail, so the principal component is understood before its plumbing.

Declare the canonical component as `export default function Name()` whenever the component earns its own declared `Props` interface. When a file provides a family of simple peer components whose props are fully expressed by `ComponentProps<'tag'>` or plain `PropsWithChildren` — markdown node renderers, typography primitives — named `export function` declarations are the natural shape. Do not use `React.FC` or arrow-function constants for components. Keep subordinate local components as plain non-exported `function Name()` declarations below the principal export.

Order a component file so it reads from public story to supporting detail:

1. `'use client'` when needed, then one blank line;
2. all imports as a single block;
3. the default or principal component;
4. subordinate components and local helpers;
5. module-level constants, module state, and `metadata` exports;
6. `Props` and other local types at the bottom.

Local types, constants, and module singletons belong below the code that uses them. Never hoist a `Props` interface above its component.

## Name Props by the file's export shape

**Intent:** Uniformity — `interface Props` is a project-wide signal meaning "one canonical component lives here"; keep that signal truthful.

- A file whose identity is one default-exported component declares exactly `interface Props`, below the component — never `ComponentNameProps`.
- A subordinate local component does not earn a declared interface; type its props inline in the signature: `function Link({ href = '/', ...props }: ComponentProps<'a'>)`.
- Only when a file intentionally exports several peer components that each need a declared contract, name the interfaces `ComponentNameProps`. Avoid designing files into this situation: prefer one canonical component per file, or inline the types.

Within `Props`:

- extend the wrapped element with `ComponentProps<'tag'>` and forward `...props`, rest parameter last;
- destructure in the signature with defaults inline (`{ name = 'INSD', asChild = false, ...props }: Props`);
- type children with `PropsWithChildren` for children-only wrappers, `ReactNode` for slot-like props, and a specific `ReactElement<XProps>` only when the contract genuinely requires one;
- use `interface` for props and object shapes; reserve `type` for unions, tuples, and function aliases (`type Status = 'loading' | 'ready' | 'error'`).

## Shape library modules top-down

**Intent:** Flow — a published module reads from public story to supporting detail, exactly like a component file.

Order a library entry module so the package's reason to exist leads: injected or configured constants first; then the principal export (the class or function the package is named for); then subordinate helpers; then options and contract interfaces below the code that uses them — declaration hoisting makes this legal, usage-first makes it readable; type re-exports last.

In signatures, destructure only what the function transforms; reach untouched fields through the named options object (`options.endpoint`) so provenance stays visible at the use site.

## Build folder-level semantic boundaries

**Intent:** Balance — a feature owns its private views and mechanics; promotion into shared space is an earned event, not a default.

Use folders when a feature develops several cooperating components, hooks, models, or transformations. Let the folder's primary file or index present the canonical capability and keep implementation files directly addressable only where useful.

Name files as lowercase single words (`image.tsx`, `navigation.tsx`, `frame.ts`); component names stay PascalCase. A multi-file component folder exposes its canonical component from `index.tsx`. Keep route-private view folders under a leading underscore (`_views`, `_opengraph`) inside their route.

Co-locate route-specific or feature-private views beneath their owner rather than promoting them into a global component directory. Promote a component only when it becomes a genuine cross-feature concept.

Keep pages and top-level views declarative. They should visibly assemble sections, providers, and domain operations in execution or rendering order. Move detailed data acquisition, parsing, interaction mechanics, and reusable visual primitives into their owning modules.

## Prefer inference and derived contracts

**Intent:** Flow — every contract has one source of truth, and annotations appear only where meaning crosses a boundary.

Let TypeScript infer local variables, callback returns, and implementation details.

Create explicit types for:

- public domain models;
- shared state/action contracts;
- wire, storage, validation, or infrastructure boundaries;
- component props that add domain fields to native props;
- module outputs depended on by sibling composition.

Derive rather than duplicate when a source of truth exists:

- use `ComponentProps` for native or wrapped component props;
- derive schema outputs from the validation schema (`interface Post extends z.infer<typeof scheme>`);
- use `ReturnType` for a stable module output when sibling infrastructure or composition consumes it;
- use `Parameters`, discriminated unions, and literal inference where they accurately preserve an existing contract.

Do not add return annotations or standalone interfaces merely to restate obvious inference. Do not export a type unless another module needs the contract.

Keep local `Props`, `State`, and `Actions` declarations at the bottom of the file, below everything that uses them. Place a large public domain contract in a dedicated module when it becomes an independent concept.

When types are the module's subject — a domain-model or wire-contract file — order them top-down instead: the root contract leads, and its constituent types follow in the order they are first referenced, root to leaf. Both placements serve the same principle: the file's main story comes first, and supporting detail appears where the reader meets it.

## Type parameters with the ecosystem's contracts

**Intent:** Uniformity — a concept the ecosystem already speaks keeps the ecosystem's vocabulary, so consumer boilerplate transfers verbatim.

When a parameter concept has a de-facto ecosystem contract — AWS credentials, `fetch`, `AbortSignal` — type it with the ecosystem's published types and forward values by spread; declare no parallel local shape. The test: a consumer's existing boilerplate for that concept must transfer verbatim (`credentials: fromNodeProviderChain()`). A type-only ecosystem package is a legitimate runtime dependency when its types appear in the public surface.

## Compose React instead of abstracting prematurely

**Intent:** Balance — composition keeps each piece's responsibility visible in the JSX; premature abstraction hides it behind configuration.

Prefer ordinary component composition, children, and narrow context providers over configuration-heavy component factories.

- Keep providers responsible for one cohesive shared state or capability.
- Split state and actions when it materially narrows consumer updates or contracts.
- Extract a hook when it owns a reusable lifecycle or interaction mechanism.
- Keep a local component as a function when it has no independent reuse or domain identity.
- Avoid generic `Base`, `Manager`, or render-prop abstractions for a single current use.

Idioms:

- Return a labeled `as const` tuple from a hook that pairs an action with state: `return [action, feedback] as const;`.
- Guard context invariants with a plain throw naming the required provider (`throw new Error('... must be used within a HeaderProvider')`).
- Use `&&` for single-branch conditional JSX, a ternary for two-way values and attributes (`loading={seen ? 'eager' : 'lazy'}`), and `??` for nullish defaults.

Preserve symmetry among peer cards, sections, routes, metadata builders, and content loaders. Some repeated JSX or mapping code is acceptable when it keeps parallel structures obvious.

## Keep asynchronous composition readable

**Intent:** Flow — asynchronous structure should narrate the phases of one story, even at a small measured cost.

Group independent asynchronous work when it represents one logical operation. `Promise.all` may serve semantic grouping and visual organization in addition to concurrency.

Convert an expected load failure into a domain decision at the call site — `.catch(() => null)` followed by `notFound()` — rather than spreading try/catch through the page.

When an environment prevents top-level static imports, group dynamic imports at the composition root, then perform construction in a separate readable phase:

```ts
const [$alpha, $beta] = await Promise.all([
  import('./alpha'),
  import('./beta'),
]);

const alpha = $alpha.run();
const beta = $beta.run(alpha);
```

Accept a small performance cost for a clearer top-level narrative when the cost is irrelevant to requirements. Do not claim a performance motivation without measurement.

## Keep infrastructure modules symmetric

**Intent:** Uniformity — modules in equivalent roles share one shape, so reading one teaches all of them.

Give sibling infrastructure modules the same public shape when they play equivalent roles, for example one `run` function plus a derived result type:

```ts
export function run() {
  // Declare one coherent resource group.
  return { resource };
}

export type Module = ReturnType<typeof run>;
```

Let the top-level configuration import modules, construct independent groups, then pass their outputs into dependent groups. Keep provider details and resource declarations in the child that owns them. If one category grows too large, add an intermediate composition module instead of overloading the root.

Use this pattern only for real sibling modules. Do not manufacture `run` functions or result aliases for isolated files that have no composition role.

## Keep errors proportional

**Intent:** Balance — failure detail exists only where a caller decides differently because of it.

Use framework-native exceptions and validation failures when callers only need success or failure. Introduce discriminated errors or result unions when callers make different decisions for different failures.

Parse, don't validate: consume unknown external data through a schema named `scheme` at the boundary, and keep internal values strongly inferred from that validation — past the boundary, the type system carries the proof. Include the offending value in boundary error messages. Do not spread `unknown`, casts, or duplicated DTO types through the application.

## Follow the naming register

**Intent:** Uniformity — one vocabulary across the project, so a name read anywhere means the same thing everywhere.

- Declare local module constants freely per ordinary JavaScript convention, but name exported module constants in camelCase (`formatter`, `config`, `base`) — no SCREAMING_SNAKE.
- Name a local handler identically to the prop it is passed into (`function onLoad()` handed to `onLoad={onLoad}`); use a bare verb (`scrollTo`) when it is not bound to a prop. Do not introduce a parallel `handleX` vocabulary.
- Inside a component, prefer booleans that read as bare adjectives or participles (`active`, `copied`, `mounted`, `external`), and reserve the `is` prefix for type guards (`isPromiseLike`). This is a soft preference, not a hard rule.
- Generic parameters are `T` or `T`-prefixed (`TCallback`).
- `??` is the default spelling of "not provided"; write `||` only when collapsing `''` or `0` is a deliberate, stated decision.
- Import Node builtins with the `node:` prefix.

## Preserve readable source shape

**Intent:** Beauty — vertical rhythm and wrap-avoidance keep similar code looking similar after the formatter runs.

Follow the repository formatter and lint rules. Within that style, apply the vertical rhythm from `SKILL.md` concretely:

- pack consecutive one-line statements, declarations, and hook calls with no blank lines between them — three `useState` lines stay glued;
- put a blank line above and below every multi-line construct: an `if`/`else`, a loop, a multi-line call or literal, a long method chain;
- put a blank line after an early-return guard before the main work;
- when one-liners accumulate into a long run, split them into logically grouped clusters separated by blank lines;
- keep interface members packed; separate sibling type declarations with one blank line;
- in JSX, separate multi-line sibling elements with a blank line, and keep tightly paired one-line siblings glued — repeated pairs read as grouped stanzas.

Prevent ugly automatic wrapping by restructuring the line instead of accepting the wrap:

- extract an intermediate constant with a domain name before the return (`const external = href?.startsWith('http') ?? false;`);
- split long class lists into one semantic group per `cn()` string argument, incoming `className` passed last, trailing comma on the multi-line call;
- give a multi-attribute JSX element one attribute per line;
- never inline a complex expression into JSX when a named constant reads better.

Keep all imports in one block with no blank lines between groups. Inline the `type` modifier when mixing value and type imports from one module (`import { use, type ComponentProps } from 'react'`); use `import type` for type-only modules.

Write almost no inline comments. Give exported library functions, hooks, and non-obvious components a short JSDoc with the prose in Korean and technical nouns in English. In JSDoc, document a destructured option as `@param property`, never `@param param0.property` — functions take at most one options object, so the prefix adds nothing. Never leave commented-out code.

Favor early returns for invalid or exceptional cases. Do not reorder code mechanically if the existing order tells a clearer story.

## Verify TypeScript and UI changes

**Intent:** Proportionality — one focused check that exercises the changed behavior beats a battery of ceremonial ones; stop once the risk is answered.

Run the relevant formatter, linter, type checker, focused tests, and build scripts discovered in the repository.

For React behavior, verify the rendered state or interaction when practical. Add focused tests for state transitions, parsing, validation, or regressions. Do not create a generic test harness or mock architecture solely to test one small component; test through the smallest stable public surface.
