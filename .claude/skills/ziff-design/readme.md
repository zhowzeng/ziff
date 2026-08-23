# Ziff Design System

## Company & product

Ziff is a desktop app for code review. It connects to GitHub, syncs pull requests and review comments, and gives a clean, friendly diff-reading interface — so reviewers can take notes quickly while reading code, organize their feedback, and copy review context to CLI coding agents (Codex, Claude, etc.).

**Sources provided:** one product screenshot showing the diff review screen — a file tree, a unified diff view, and an inline comment thread. No codebase, Figma file, or brand guide was attached. The user's one stated preference: they like GitHub's light theme for readability, and find its comment-leaving flow intuitive.

Because the only source is a single screenshot, this design system is a first pass: a light, GitHub-inspired visual system and component set built to match that one screen, extended with the standard primitives (buttons, inputs, badges) a desktop review app needs. **Update (2026-08-23):** the neutral/success/danger/warning color scale and the UI/mono type stack are no longer approximations — they're pulled directly from GitHub's own open-source design tokens (`github/primer/primitives`) and typeface (`github/mona-sans`). See "Fonts" and "Palette source" below.

## Components (source of truth: claude.ai/design project)

- `components/core/` — Button, IconButton, Icon (Lucide via CDN)
- `components/forms/` — Input, Textarea
- `components/feedback/` — Badge
- `components/comments/` — Avatar, CommentThread
- `components/diff/` — FileTree, DiffLine, DiffHunk

These are authored as React (`.jsx`) in the claude.ai/design project. **In this repo (SvelteKit), they've been ported to Svelte 5** — see `src/lib/components/`. Ported: Button, IconButton, Icon, Input, Textarea, Badge, Avatar, CommentThread (+ CommentItem), FileTree (+ FileTreeRow), DiffLine (+ DiffHunk, DiffLineSplit). From the desktop-app UI kit: Dropdown, ContextDrawer (+ ContextItem), ContextFab, FileHeader, FetchButton. `Segmented` is a Svelte-only addition (not a separate file in the React source) — `TopBar.jsx`'s `DiffModeToggle` and `DiffPanel.jsx`'s `ViewToggle`/`Segmented` were two near-identical inline implementations in the reference; ported as one reusable component instead of duplicating. Nothing left unported from the core/forms/feedback/comments/diff sets or the desktop-app chrome.

**Reverse-synced from the app back into claude.ai/design (2026-08-23):** `Modal`, `Toast`/`ToastStack`, `Checkbox`, `Switch`, `Tooltip`, and `Segmented` were originally built Svelte-first (no React spec) to fill common gaps ahead of any screen needing them. The design system's own sync process (`github.md` in that project) pulled them back in as confirmed React components — they're no longer a Svelte-only deviation, both sides now agree. No visual changes came out of that sync, only new coverage.

Deviation from the React spec worth knowing: `CommentThread`'s `comments` items use a plain `text: string` field instead of React `children` (a JSX node) — Svelte props aren't JSX, so this is the natural adaptation. Everything else follows the `.d.ts` shapes as published.

## UI kits

- `ui_kits/desktop-app/` (in the claude.ai/design project) — the full reference: PR header, file tree sidebar (with file filter, show-all-files toggle, and a settings entry), diff panel, context drawer, comment thread with reply + resolve, project/branch dropdowns, a settings modal, and empty states for "no project selected" / "no diff on this branch" / "no files match filter".
- `src/routes/review/+page.svelte` in this repo — now a full port of `ui_kits/desktop-app/index.html`'s behavior: topbar with real project/branch `Dropdown`s, a diff-mode `Segmented` toggle, and a `FetchButton`; a diff panel with a `FileHeader`, a Unified/Split `Segmented` view toggle, and gutter drag-to-select-range commenting (mouse down on a line's gutter, drag across lines, release to open a comment thread anchored to that range) alongside the single fixed demo thread; a `ContextDrawer` that lists comments saved for a CLI agent; and a `ContextFab` that toggles it and shows the saved count. Submitting any comment (the fixed thread's reply, or a new range selection) saves it to context and opens the drawer automatically. One faithful-to-reference quirk carried over: in Split view there's no gutter-drag, so the only way to reopen the closed fixed thread there is the per-line hover "+" button — in Unified view that same hover button is superseded by the gutter drag handle and never renders (matches the original React reference's prop precedence).

**Settings + empty states (2026-08-23):** ported from the reference's newly-designed `SettingsModal.jsx` and inline `EmptyMain`/`EmptyPane` functions. `src/lib/components/SettingsModal.svelte` is a new component (Modal + Segmented + Dropdown + Switch + Textarea) with three sections — Appearance (theme, language), View (zoom, diff font size, default diff mode), Copy to Agent (optional prefix prompt) — wired to a "Settings" entry at the bottom of the sidebar in `review/+page.svelte`. `src/lib/components/EmptyState.svelte` is one reusable component (`size="md"`/`"sm"`) replacing the reference's two near-identical inline functions, covering: no project/branch selected, no diff on the current branch, and no file matches the sidebar filter. The review page also gained a small fixed "DEMO STATE" switcher (normal/no-project/no-diff, top-right) mirroring the reference's `DemoScenarioSwitch` — a prototype-only aid for previewing the empty states, not meant to ship.

## Guidelines

- `guidelines/colors-*.html` — neutral ramp, accent, semantic, diff colors
- `guidelines/type-*.html` — sans and mono type scales
- `guidelines/spacing-scale.html`, `guidelines/radius-elevation.html`
- `guidelines/brand-wordmark.html` — the type-only "Ziff" wordmark placeholder (synced 2026-08-23)

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

- **Palette source (2026-08-23):** the neutral, success (green), danger (red), and warning (yellow) scales, plus the semantic aliases (`bgColor.*`, `fgColor.*`, `borderColor.*`) and the diff component tokens (`diffBlob.*`), were fetched directly from `github/primer/primitives` (live `main` branch) — exact, current GitHub values. Notable real detail: GitHub's diff **hunk header** background is a subtle **blue** tint (`bgColor.accent.muted`, `#ddf4ff`), not gray — `--diff-hunk-bg` matches this.
- **Palette**: near-white canvas (`--bg-canvas: #fff`) with a warm-neutral gray scale for borders and secondary text. One brand accent, teal (`--accent`), used sparingly for primary actions, active file-tree rows, and focus rings — kept intentionally distinct from GitHub's blue.
- **Type**: **Mona Sans** — GitHub's own open-source UI typeface (`github/mona-sans`, SIL OFL-1.1, also on Google Fonts) — paired with the **native system monospace stack** for code/diffs/line numbers. Mirrors github.com exactly: GitHub ships Mona Sans for UI text but doesn't load a custom webfont for code.
- **Spacing**: 4px base unit, scale at 4/8/12/16/20/24/32/48/64.
- **Backgrounds**: flat and functional — no imagery, no gradients, no illustration, no texture.
- **Animation**: restrained, fast transitions only (150ms ease, opacity/background on hover).
- **Hover states**: subtle background shifts (`--bg-subtle` on rows/buttons), never color inversion or scale changes.
- **Press states**: slightly darker shade of the hover background, no shrink/scale.
- **Borders**: 1px hairlines (`--border-default: #d1d9e0`, real Primer `borderColor.default`) are the dominant structural device, more than shadow. `--border-muted` aliases `--border-default` — accurate to real GitHub.
- **Shadows**: minimal. A comment thread card gets a small elevation (`--shadow-md`); everything else is flat with borders only.
- **Corner radii**: small — 4–6px on buttons/inputs, 10px on the comment card. Nothing pill-shaped except status badges.
- **Cards**: white background, 1px border, small radius, small shadow only when floating above content.
- **Transparency/blur**: none used.
- **Imagery**: none — this is a text/code-first tool.

## ICONOGRAPHY

Small monochrome line icons (chevrons, copy, emoji-reaction) — a stroke-based icon set. Uses **Lucide** (lucide.dev, MIT-licensed). In this repo, use the `lucide-svelte` package (already installed) via `src/lib/components/Icon.svelte`, not the CDN. This is a **substitution, flagged for the user**: if Ziff's real app uses a different icon set, swap it out.

## Fonts

**UI sans — Mona Sans (real, not a substitution).** GitHub's own open-source variable typeface, from `github/mona-sans` (SIL OFL-1.1 license). GitHub's actual production UI font. Loaded via `fonts.googleapis.com` in `tokens/typography.css`.

**Code/mono — native system monospace stack (real, not a substitution).** `ui-monospace, SF Mono, SFMono-Regular, Menlo, Consolas, Liberation Mono, monospace` — literally what `primer/primitives` defines for `fontStack.monospace`. GitHub does not ship a custom webfont for code.

## No logo

No logo or brand mark provided. The wordmark "Ziff" is set in plain type. **Still pending as of 2026-08-23 — hold off until real brand assets arrive.**

## Open questions / next steps

1. The Ziff codebase or a Figma file, so components and screens can be built from real source rather than inference.
2. ~~Additional screens.~~ **All resolved as of 2026-08-23.** `PR list/inbox` — out of scope for v1 (local repos/diffs only, no GitHub PR sync). `Settings` — real screen shipped (`SettingsModal`, see "UI kits"). `Empty states` — designed and ported (`EmptyState`, no-project / no-diff / no-match). `Copy context to agent` — turned out to need no screen/flow, just copy-to-clipboard (already implemented). `Sync/connection states` — scoped down once GitHub PR sync was ruled out: the only "connection" left is a local `git fetch`, which already has a loading state (`FetchButton`'s spin animation); the one gap was fetch-failure feedback, and that's a `Toast` (danger variant, already built) wired to the real fetch call once it exists — an implementation task, not a design one.
3. Real brand assets: **logo** (open item) and confirmation of the accent color (teal is a placeholder, deliberately kept distinct from GitHub's blue even after aligning the rest of the palette to real GitHub tokens).
4. Confirmation of tone/voice guidelines beyond what one code comment can show.
5. **New (2026-08-23):** the Project dropdown (`TopBar.jsx`) should have an "add new" entry that lets the user pick a local folder to add as a project — not yet designed. Two separate things bundled in one idea: the dropdown UI treatment (an "+ Add project…" row, probably with a divider) and the actual folder-picker behavior (native OS dialog via Tauri) — the latter needs a new dependency (`@tauri-apps/plugin-dialog`) and real state for a persisted project list, neither of which exists yet in the prototype.
