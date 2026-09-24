# Syntax highlighting uses Shiki in the frontend, over whole files

The diff and File View are coloured by Shiki (`src/routes/review/highlight.ts`), which tokenizes
**each whole file**, not the lines a hunk shows: a hunk that starts inside a block comment or a
template string can't be highlighted from its own lines, because the state that says "this is a
comment" was set above it. So `get_file_diff` returns both sides in full (`oldText`, `newText`) along
with the hunks. Rust already reads both to diff them, so this adds no reads. It only returns text
that used to be dropped after diffing (ADR 0005 still holds). File View highlights the lines it
already has. A del line takes its colours from the old side and every other line from the new, both
looked up by line number.

Colours come from Shiki's CSS-variables theme, so every token is a `var(--syntax-*)` reference and the
palette lives in `colors.css` with the other design tokens (GitHub Light values for now). The syntax
colours and the within-line change marks (`inlineSegments`) are laid over each other in `paintLine`,
which cuts a line wherever either one changes.

Highlighting lands after the plain lines are on screen, so a grammar loading for the first time
never holds up the diff. A file with no grammar, over 500,000 characters, or failing to highlight
stays plain text. Grammars are split into separate chunks and loaded on first use. The build output
grows by about 8 MB of grammar chunks, and the review page's own chunk by about 200 KB (Shiki core
and the JS regex engine).

## Considered Options

- **syntect (+ two-face) in Rust**: rejected. It would also highlight whole files, and it is what bat
  and delta use. But Shiki's TextMate grammars are the ones VS Code ships, so code looks the way the
  reviewer's editor shows it. It also hands back per-line tokens that merge directly with the
  frontend's within-line diff, and it keeps the Rust binary from growing by a syntax set.
- **Highlight each hunk's lines on their own**: rejected. It needs no extra IPC, but multi-line
  comments and strings come out wrong exactly where a hunk begins mid-construct.
- **Shiki's Oniguruma WASM engine**: rejected for the JS regex engine (`forgiving`). No WASM to load
  in the webview. The few patterns the JS engine can't compile go unmatched rather than failing the
  file.
- **A ready-made theme (e.g. `github-light`) instead of CSS variables**: rejected. Hex colours baked
  into tokens would sit outside the design tokens, so a later dark theme would mean a second
  tokenization instead of a second set of variables.
