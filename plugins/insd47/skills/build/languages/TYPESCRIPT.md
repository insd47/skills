# TypeScript and TSX

Apply with `SKILL.md` when TypeScript or TSX files are in scope.

## Exports

React components: default export for the file's one canonical component; named exports for an intentional family of equal peers (renderers, primitives); subordinate components stay private. Other modules: named exports for peer operations; default export for one canonical configured value. No broad barrels — import from the module that owns the concept.

## Component files

Declare the canonical component `export default function Name()` when it earns a declared `Props` interface; a family of simple peers whose props are fully expressed by `ComponentProps<'tag'>` or `PropsWithChildren` uses named `export function` declarations. No `React.FC`, no arrow-const components. Subordinate locals are plain non-exported `function Name()` declarations below the principal export.

File order: `'use client'` + blank line → one import block → principal component → subordinates and helpers → module constants, state, `metadata` → `Props` and local types last. Never hoist `Props` above its component.

## Props

- A file whose identity is one default-exported component declares exactly `interface Props`, below the component — never `ComponentNameProps`. That name appears only when a file intentionally exports several peer components each needing a declared contract; avoid designing files into that situation.
- Subordinate locals type props inline: `function Link({ href = '/', ...props }: ComponentProps<'a'>)`.
- Extend the wrapped element with `ComponentProps<'tag'>` and forward `...props` last; destructure with inline defaults in the signature.
- children: `PropsWithChildren` for children-only wrappers, `ReactNode` for slot props, `ReactElement<XProps>` only when the contract requires it.
- `interface` for props and object shapes; `type` for unions, tuples, function aliases.

## Library modules

Entry order: injected or configured constants → the principal export the package is named for → helpers → options and contract interfaces below the code that uses them → type re-exports last. In signatures, destructure only what the function transforms; reach untouched fields through the named options object (`options.endpoint`) so provenance stays visible.

## Folders

Create a folder when a feature grows several cooperating components, hooks, models, or transformations. Files are lowercase single words (`image.tsx`); components stay PascalCase; a multi-file component folder exposes its canonical component from `index.tsx`; route-private views live under underscore folders (`_views`) inside their route. Co-locate feature-private views under their owner; promote only genuine cross-feature concepts. Pages and top-level views stay declarative — visibly assembling sections, providers, and domain operations — with data acquisition, parsing, and interaction mechanics in their owning modules.

## Inference and derived contracts

Let TypeScript infer locals, callback returns, and implementation details. Declare explicit types where meaning crosses a boundary: public domain models, shared state/action contracts, wire/storage/validation shapes, props adding domain fields to native props, outputs consumed by sibling composition. Derive from the source of truth instead of duplicating: `ComponentProps` for wrapped components, `interface Post extends z.infer<typeof scheme>` for schema outputs, `ReturnType` for a stable module output consumed by siblings, `Parameters`/discriminated unions/literal inference where they preserve an existing contract. No annotations restating obvious inference; no exported type without an external consumer. Local `Props`/`State`/`Actions` sit at the bottom below their users; a domain-model or wire-contract file instead reads top-down, root contract first, constituents in first-reference order.

## Ecosystem contracts

Type a concept the ecosystem already speaks — AWS credentials, `fetch`, `AbortSignal` — with the ecosystem's published types and forward values by spread; no parallel local shape. The test: a consumer's existing boilerplate transfers verbatim (`credentials: fromNodeProviderChain()`). A type-only package is a legitimate dependency when its types appear in the public surface.

## React composition

Compose with children and narrow context providers instead of configuration-heavy factories: one cohesive concern per provider; split state and actions when it materially narrows consumer updates; extract a hook when it owns a reusable lifecycle or interaction; keep locals as plain functions; no `Base`/`Manager`/render-prop abstraction for a single use.

Idioms: `return [action, feedback] as const` from a hook pairing action with state; guard context invariants with a plain throw naming the required provider; `&&` for single-branch JSX, ternary for two-way values (`loading={seen ? 'eager' : 'lazy'}`), `??` for nullish defaults. Preserve symmetry among peer cards, sections, routes, and loaders — some repeated JSX is the price of parallel structure staying obvious.

## Async

Group independent awaits with `Promise.all` when they form one logical operation — semantic grouping counts as much as concurrency. Convert an expected load failure into a domain decision at the call site (`.catch(() => null)` then `notFound()`) rather than spreading try/catch. When static imports are unavailable, gather dynamic imports at the composition root, then construct in a separate readable phase:

```ts
const [$alpha, $beta] = await Promise.all([
  import('./alpha'),
  import('./beta'),
]);

const alpha = $alpha.run();
const beta = $beta.run(alpha);
```

Accept a small requirements-irrelevant cost for a clearer top-level narrative; never claim a performance motivation without measurement.

## Infrastructure symmetry

Sibling infrastructure modules in equivalent roles share one public shape:

```ts
export function run() {
  return { resource };
}

export type Module = ReturnType<typeof run>;
```

The top-level configuration imports modules, constructs independent groups, then passes outputs into dependent groups; provider details stay in the owning child; add an intermediate composition module when a category outgrows the root. Do not manufacture this shape for isolated files with no composition role.

## Errors

Framework-native exceptions and validation failures when callers only need success or failure; discriminated errors or result unions when callers decide differently per failure. Parse, don't validate: consume unknown external data through a schema named `scheme` at the boundary and keep internal values inferred from it — past the boundary the type system carries the proof. Include the offending value in boundary error messages. No `unknown`, casts, or duplicated DTOs inside the application.

## Naming register

- Exported module constants in camelCase (`formatter`, `config`) — no SCREAMING_SNAKE.
- A handler passed to a prop shares its name (`function onLoad()` → `onLoad={onLoad}`); an unbound handler takes a bare verb (`scrollTo`). No parallel `handleX` vocabulary.
- Component booleans read as bare adjectives or participles (`active`, `copied`, `mounted`); the `is` prefix is for type guards (soft preference).
- Generic parameters are `T` or `T`-prefixed (`TCallback`).
- `??` is the default "not provided"; write `||` only when collapsing `''`/`0` is a deliberate, stated decision.
- Node builtins use the `node:` prefix.

## Source shape

Follow the repository formatter; within it keep the vertical rhythm from `SKILL.md`:

- pack consecutive one-liners (three `useState` stay glued); blank line above and below every multi-line construct and after each early-return guard; split long one-liner runs into clustered stanzas; interface members packed, sibling type declarations separated; multi-line JSX siblings separated, tightly paired one-liners glued.
- restructure a line rather than accept an ugly wrap: extract a named intermediate (`const external = href?.startsWith('http') ?? false;`); one semantic group per `cn()` string argument with incoming `className` last and a trailing comma; one attribute per line on multi-attribute JSX; name a complex expression before it enters JSX.

One import block, no blank lines inside; inline the `type` modifier when mixing (`import { use, type ComponentProps } from 'react'`); `import type` for type-only modules.

Almost no inline comments. Exported functions, hooks, and non-obvious components get a short JSDoc in Korean with technical nouns in English; document a destructured option as `@param property`, never `@param param0.property`. No commented-out code. Favor early returns; keep an existing order that tells a clearer story.

## Verify

Run the repository's formatter, linter, type checker, focused tests, and build. Verify rendered state or interaction for React behavior when practical; add focused tests for state transitions, parsing, validation, and regressions. No generic harness or mock architecture for one small component — test through the smallest stable public surface.
