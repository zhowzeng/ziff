# Route-local code colocates with the route; `lib/` only for cross-route sharing

Component, state, helper, type, and API-wrapper code that serves a single route lives inside that route's own folder under `src/routes/`, added and removed alongside the route. Code only moves to `src/lib/<feature>/` once a second route or surface actually needs it — we default to colocation and lift on confirmed reuse, not in anticipation of it. Svelte 5 state modules (in either location) use the rune-based `state.svelte.ts` naming convention; see the `ziff-frontend` skill for the state-module patterns.

## Considered Options

- **Feature-first `lib/` structure from day one** (put everything under `src/lib/<feature>/` even for single-route code): rejected because it forces a shared/not-shared judgment before a second consumer exists, and creates premature abstractions for routes that end up staying single-purpose.
