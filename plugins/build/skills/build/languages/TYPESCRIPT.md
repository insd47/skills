# TypeScript and TSX Guidance

Apply this guide together with the parent `SKILL.md` when TypeScript or TSX files are in scope.

## Make exports express module meaning

For React components:

- Use a default export when the file has one canonical component or view.
- Use named exports when the file intentionally provides a family of equal components, hooks, or primitives.
- Keep subordinate components private when they exist only to explain the canonical component.

For non-component modules:

- Use named exports for several peer operations or domain values.
- Use a default export for one canonical configured value or transformation when that is the clearest module identity.
- Avoid broad barrel exports. Import from the module that owns the concept.

Place the principal export near the top so the file reads from public story to implementation detail. Put local helpers, constants, module state, and private types below it when doing so keeps the main flow visible.

## Build folder-level semantic boundaries

Use folders when a feature develops several cooperating components, hooks, models, or transformations. Let the folder's primary file or index present the canonical capability and keep implementation files directly addressable only where useful.

Co-locate route-specific or feature-private views beneath their owner rather than promoting them into a global component directory. Promote a component only when it becomes a genuine cross-feature concept.

Keep pages and top-level views declarative. They should visibly assemble sections, providers, and domain operations in execution or rendering order. Move detailed data acquisition, parsing, interaction mechanics, and reusable visual primitives into their owning modules.

## Prefer inference and derived contracts

Let TypeScript infer local variables, callback returns, and implementation details.

Create explicit types for:

- public domain models;
- shared state/action contracts;
- wire, storage, validation, or infrastructure boundaries;
- component props that add domain fields to native props;
- module outputs depended on by sibling composition.

Derive rather than duplicate when a source of truth exists:

- use `ComponentProps` for native or wrapped component props;
- derive schema outputs from the validation schema;
- use `ReturnType` for a stable module output when sibling infrastructure or composition consumes it;
- use `Parameters`, discriminated unions, and literal inference where they accurately preserve an existing contract.

Do not add return annotations or standalone interfaces merely to restate obvious inference. Do not export a type unless another module needs the contract.

Keep short local `Props`, `State`, and `Actions` declarations near the bottom when that makes the principal component or operation immediately readable. Place a large public domain contract in a dedicated module when it becomes an independent concept.

## Compose React instead of abstracting prematurely

Prefer ordinary component composition, children, and narrow context providers over configuration-heavy component factories.

- Keep providers responsible for one cohesive shared state or capability.
- Split state and actions when it materially narrows consumer updates or contracts.
- Extract a hook when it owns a reusable lifecycle or interaction mechanism.
- Keep a local component as a function when it has no independent reuse or domain identity.
- Avoid generic `Base`, `Manager`, or render-prop abstractions for a single current use.

Preserve symmetry among peer cards, sections, routes, metadata builders, and content loaders. Some repeated JSX or mapping code is acceptable when it keeps parallel structures obvious.

## Keep asynchronous composition readable

Group independent asynchronous work when it represents one logical operation. `Promise.all` may serve semantic grouping and visual organization in addition to concurrency.

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

Use framework-native exceptions and validation failures when callers only need success or failure. Introduce discriminated errors or result unions when callers make different decisions for different failures.

Validate unknown external data at the boundary. Keep internal values strongly inferred from that validation. Do not spread `unknown`, casts, or duplicated DTO types through the application.

## Preserve readable source shape

Follow the repository formatter and lint rules. Within that style:

- group imports and statements by semantic purpose;
- use blank lines to separate setup, action, and result;
- favor early returns for invalid or exceptional cases;
- keep the main component or operation above its helpers;
- keep similar sibling files visually parallel;
- use direct names whose context is supplied by the module.

Do not reorder code mechanically if the existing order tells a clearer story.

## Verify TypeScript and UI changes

Run the relevant formatter, linter, type checker, focused tests, and build scripts discovered in the repository.

For React behavior, verify the rendered state or interaction when practical. Add focused tests for state transitions, parsing, validation, or regressions. Do not create a generic test harness or mock architecture solely to test one small component; test through the smallest stable public surface.
