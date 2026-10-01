# Ziff Design System

## Company & product

Ziff is a desktop app for code review. It connects to GitHub, syncs pull requests and review comments, and gives a clean, friendly diff-reading interface — so reviewers can take notes quickly while reading code, organize their feedback, and copy review context to CLI coding agents (Codex, Claude, etc.).

**Sources provided:** one product screenshot showing the diff review screen — a file tree, a unified diff view, and an inline comment thread. No codebase, Figma file, or brand guide was attached. The user's one stated preference: they like GitHub's light theme for readability, and find its comment-leaving flow intuitive.

Because the only source is a single screenshot, this design system is a first pass: a light, GitHub-inspired visual system and component set built to match that one screen, extended with the standard primitives (buttons, inputs, badges) a desktop review app needs. The neutral/success/danger/warning color scale and the UI/mono type stack are pulled directly from GitHub's own open-source design tokens (`github/primer/primitives`) and typeface (`github/mona-sans`) — see "Fonts" and "Palette source" below.

## Components

Source of truth: `src/lib/components/` for components shared across routes; review-only
components colocate at `src/routes/review/components/` instead (see
`docs/decisions/0004-route-colocation-over-early-lib-sharing.md`). Both are Svelte 5, runes.

- Core (`lib/components/`) — Button, IconButton, Icon
- Forms (`lib/components/`) — Input, Textarea
- Feedback (`lib/components/`) — Badge
- Comments (`routes/review/components/`) — Avatar*, CommentThread (+ CommentItem)
- Diff (`routes/review/components/`) — FileTree (+ FileTreeRow), DiffLine (+ DiffHunk, DiffLineSplit)
- Desktop-app chrome — QueueDrawer (+ QueueItem), QueueFab, FileHeader are in
  `routes/review/components/`; Dropdown, FetchButton, SettingsModal, EmptyState stay in
  `lib/components/` (plausibly reusable by a future non-review route)
- Other primitives (`lib/components/`) — Modal, Toast/ToastStack, Checkbox, Switch, Tooltip, Segmented

\* `Avatar` is grouped with Comments by usage but lives in `lib/components/` — it's a generic
identity primitive, not review-specific.

`Segmented` is one reusable component covering what would otherwise be two near-identical toggles (the topbar's diff-mode toggle, the diff panel's unified/split view toggle).

Worth knowing: `CommentThread`'s `comments` items use a plain `text: string` field rather than a rich content node — keeps the prop shape simple since there's no need for arbitrary child content there.

## Reference implementation

`src/routes/review/+page.svelte` is the full reference screen: topbar with repo/branch `Dropdown`s, a diff-mode `Segmented` toggle, and a `FetchButton`; a diff panel with a `FileHeader`, a Unified/Split `Segmented` view toggle, and gutter drag-to-select-range commenting (mouse down on a line's gutter, drag across lines, release to open a comment anchored to that range); a `QueueDrawer` that lists comments saved to the Comment Queue for a CLI agent; and a `QueueFab` that toggles it and shows the saved count. Submitting any comment saves it to the queue and opens the drawer automatically.

Both views comment the same way: the gutter drag handle appears on the new side only — the right column in Split view, a line with a new-side number in Unified — because a deleted line is gone from the worktree and can't be anchored to (docs/decisions/0010). Esc closes the open comment thread. A hunk header collapses its own hunk, and the `FileHeader` toggle collapses or expands them all; collapsing the hunk an open thread sits in closes the thread with it.

Settings and empty states: `src/lib/components/SettingsModal.svelte` (Modal + Segmented + Dropdown + Switch + Textarea) covers Appearance (theme, language), View (zoom, diff font size, default diff mode), and Copy to Agent (optional prefix prompt), wired to a "Settings" entry at the bottom of the sidebar. `src/lib/components/EmptyState.svelte` (`size="md"`/`"sm"`) covers: no project/branch selected, no diff on the current branch, no file matches the sidebar filter. The review page also has a small fixed "DEMO STATE" switcher (normal/no-project/no-diff, top-right) for previewing the empty states — a prototype-only aid, not meant to ship.

## Guidelines

- `guidelines/colors-*.html` — neutral ramp, accent, semantic, diff colors
- `guidelines/type-*.html` — sans and mono type scales
- `guidelines/spacing-scale.html`, `guidelines/radius-elevation.html`
- `guidelines/brand-wordmark.html` — the type-only "Ziff" wordmark placeholder

Open any of these directly in a browser to preview a token category — they link straight to the app's real tokens (`../../../../src/lib/styles/tokens/index.css`), not a local copy.

## Tokens

Single source of truth: `src/lib/styles/tokens/{colors,typography,spacing}.css` in the app (imported as `index.css`). This skill folder does **not** keep its own copy — edit the app's tokens and both the components and the `guidelines/*.html` previews here update automatically.

## CONTENT FUNDAMENTALS

- **Language**: sample content is Traditional Chinese, suggesting primary users write in Chinese day to day, while code, file paths, and commit copy stay in English. UI chrome defaults to English but the app must render CJK text natively and comfortably.
- **Tone**: terse, functional, developer-to-developer. No exclamation points, no marketing voice.
- **Casing**: sentence case throughout ("Resolve comment", not "RESOLVE COMMENT").
- **Voice**: direct, second-person where the app addresses the user ("Write a reply"), neutral/observational for system state ("Synced 2m ago").
- **Emoji**: none as copy. The only "reaction" affordance is a smile icon (add-emoji-reaction button).

## VISUAL FOUNDATIONS

- **Palette source:** the neutral, success (green), danger (red), and warning (yellow) scales, plus the semantic aliases (`bgColor.*`, `fgColor.*`, `borderColor.*`) and the diff component tokens (`diffBlob.*`), are taken from `github/primer/primitives` — exact, current GitHub values. Notable real detail: GitHub's diff **hunk header** background is a subtle **blue** tint (`bgColor.accent.muted`, `#ddf4ff`), not gray — `--diff-hunk-bg` matches this.
- **Palette**: *porcelain* in light (warm tinted canvas `--bg-canvas` with white cards) and *midnight ink* in dark (Settings → Theme: System / Light / Dark; tokens redefined under `:root[data-theme="dark"]` and the OS preference in `colors.css`). One brand accent, indigo-violet (`--accent`, with `--accent-gradient` violet→pink for the logo and hero), used for primary actions, the active file row, the branch pill, comment affordances and focus rings. Diff add/remove stay green/red. Components must use semantic tokens (`--bg-raised`, `--bg-active`, `--on-emphasis`, `--text-*`), never raw `--gray-*`/hex, so both themes work. The earlier GitHub/Primer-derived look is no longer a constraint.
- **Type**: **Geist** for UI, **Geist Mono** for code, **Instrument Serif** (display, italic for emphasis) for editorial headlines such as the Welcome screen. All three are bundled via `@fontsource` — no network needed at runtime; CJK falls through to the OS font.
- **Spacing**: 4px base unit, scale at 4/8/12/16/20/24/32/48/64.
- **Backgrounds**: flat and functional in the working UI. The one expressive moment is the first-run `Welcome` screen (soft accent aura, animated mini diff, gradient headline); the Z `Logo` mark carries a teal gradient.
- **Animation**: restrained and purposeful, driven by `--ease-out` / `--ease-spring` and `--dur-*` tokens (all collapse to 0 under `prefers-reduced-motion`): sliding pill in `Segmented`, chevron rotation in the tree and hunks, fade-up for empty states / threads, drawer slide-in, modal pop, shimmer skeleton while the tree loads.
- **Hover states**: subtle background shifts (`--bg-subtle` on rows/buttons), never color inversion or scale changes.
- **Press states**: slightly darker shade of the hover background, no shrink/scale.
- **Borders**: 1px hairlines (`--border-default: #d1d9e0`, real Primer `borderColor.default`) are the dominant structural device, more than shadow. `--border-muted` aliases `--border-default` — accurate to real GitHub.
- **Shadows**: minimal. A comment thread card gets a small elevation (`--shadow-md`); everything else is flat with borders only.
- **Corner radii**: 5–8px on buttons/inputs, 12–16px on cards, modals and the comment thread. Pills only for status badges, counts and the branch indicator.
- **Layout**: the sidebar, diff panel and queue drawer float as rounded cards (`--radius-xl`) on the tinted canvas with a 10px gap; the topbar sits directly on the canvas. `⌘K` / `Ctrl+K` opens a command palette to jump to a changed file.
- **Transparency/blur**: only on floating chrome — the topbar, the sticky file header (`--bg-glass` + backdrop blur) and the modal scrim.
- **Imagery**: none — this is a text/code-first tool.

## ICONOGRAPHY

Small monochrome line icons (chevrons, copy, emoji-reaction) — a stroke-based icon set. Uses **Lucide** (lucide.dev, MIT-licensed). In this repo, use the `lucide-svelte` package via `src/lib/components/Icon.svelte`. This is a **substitution, flagged for the user**: if Ziff's real app uses a different icon set, swap it out.

## Fonts

Bundled with the app through `@fontsource-variable/geist`, `@fontsource-variable/geist-mono` and `@fontsource/instrument-serif` (all SIL OFL), imported in `tokens/typography.css`. Geist has no CJK glyphs, so Traditional Chinese renders in the platform font (PingFang TC / Noto Sans TC / Microsoft JhengHei).

## No logo

No logo or brand mark provided. The wordmark "Ziff" is set in plain type. Hold off until real brand assets arrive.

## Open questions / next steps

Only genuine design questions live here — things that need a person to decide or an example to point to. Implementation-only follow-ups (add a library, wire up real state) have been moved out: they don't need another pass through this design system, and listing them here just clutters the one list that's supposed to say what still needs design input.

1. Real brand assets: **logo** (open item — see "No logo" above) and confirmation of the accent color (teal is a placeholder, deliberately kept distinct from GitHub's blue even after aligning the rest of the palette to real GitHub tokens).
2. Confirmation of tone/voice guidelines — there's now a real body of applied copy to point to (`SettingsModal`, the empty states, comment threads), all following the same terse dev-to-dev voice, but nobody's signed off that it's actually right for Ziff.

**Resolved (kept for history, not action items):** additional screens — PR list/inbox (ruled out of scope for v1: local repos/diffs only), settings, empty states, the copy-to-agent flow (turned out to be plain copy-to-clipboard), and the Project dropdown's "add new" entry are all designed and shipped in `src/lib/components/` and `review/+page.svelte`.

**Not design questions, intentionally not tracked here:** the "add project" folder picker (needs a native Tauri dialog + a persisted project list) and wiring the fetch-failure `Toast` to a real `git fetch` call are pure implementation work with no open design decision — track them wherever the app's engineering work is tracked, not in this doc.
