# State modules are tested through the Svelte compiler

The logic that goes wrong quietly — the Comment Queue's Repo isolation (ADR 0009), `reviewState`'s
sequence guards, `sanitize()` on stored settings — all lives in `.svelte.ts` modules, whose `$state`
means nothing until the Svelte compiler has been over it. So `vitest.config.js` mounts the SvelteKit
plugin and the tests import those modules as they are, rather than testing extracted copies of the
pure parts. Node has no `localStorage`, which the settings module reads at import time, so
`vitest.setup.ts` supplies an in-memory one.

## Considered Options

- **Extract the rune-free logic and test that instead**: rejected — the bugs worth catching are in
  the containers. A missing `repoId` comparison in `itemsFor()` or a missing sequence check in
  `#loadDiff()` is invisible to a test of `sameAnchor()` alone, and those are exactly the failures
  that show a wrong Repo's content without looking broken.
- **Add jsdom for `localStorage`**: rejected — a whole DOM to stand in for four methods, on modules
  that touch nothing else in the browser. The day a test needs real DOM, that is the day to add it.
