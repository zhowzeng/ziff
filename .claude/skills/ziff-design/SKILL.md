---
name: ziff-design
description: Use this skill to generate well-branded interfaces and assets for Ziff, either for production or throwaway prototypes/mocks/etc. Contains essential design guidelines, colors, type, fonts, assets, and UI kit components for prototyping.
user-invocable: true
---

Read readme.md within this skill, and explore the other available files.

This is a **reference mirror** of the Ziff design system published at claude.ai/design (project id `3f869967-6ee9-4c7d-aa75-52bb0577bd5b`). It holds the visual guidelines (`guidelines/*.html`) as browsable documentation for color, type, spacing, and radii.

**The tokens themselves are NOT duplicated here.** The single source of truth for `colors.css` / `typography.css` / `spacing.css` is `src/lib/styles/tokens/` in the app — that's what components actually import, and the `guidelines/*.html` preview pages here link straight to that same file (`../../../../src/lib/styles/tokens/index.css`), so there's nothing to keep in sync by hand. To change a token, edit it once in `src/lib/styles/tokens/`; every component and every guideline preview picks it up automatically.

The actual production Svelte components live in `src/lib/components/` in this repo (Button, IconButton, Icon, Input, Textarea, Badge so far). When building new UI:
- Prefer reusing an existing component from `src/lib/components/` over writing new inline styles.
- When a component doesn't exist yet, use `src/lib/styles/tokens/` plus the original React reference in the claude.ai/design project (fetch via the DesignSync tool, `components/**`) as the spec, and port it to Svelte 5 (runes) following the pattern of the existing components.
- If creating throwaway prototypes/mocks, static HTML files against `src/lib/styles/tokens/index.css` are fine.

If the remote claude.ai/design project changes (a re-sync), write the new token values only into `src/lib/styles/tokens/` — do not recreate a copy here.
