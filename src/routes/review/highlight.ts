// Syntax highlighting for the diff and File View (docs/decisions/0017). Shiki
// tokenizes a whole file at once, so a line's colours follow from everything above
// it — a hunk that starts inside a block comment still reads as a comment.

import { createCssVariablesTheme, createHighlighterCore, type HighlighterCore } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import { bundledLanguages, type BundledLanguage } from 'shiki/langs';
import type { InlineSegment } from './helpers';

// A run of one line's text in one colour. The colour is a `var(--syntax-*)` reference,
// so the palette lives with the other design tokens in colors.css.
export type SyntaxToken = { text: string; color?: string };

const THEME = 'ziff';

// Past this a file is plain text: tokenizing runs on the main thread, and a
// multi-megabyte lockfile or bundle would freeze the window for seconds.
const MAX_HIGHLIGHT_CHARS = 500_000;

let highlighter: Promise<HighlighterCore> | null = null;

function getHighlighter(): Promise<HighlighterCore> {
  highlighter ??= createHighlighterCore({
    themes: [createCssVariablesTheme({ name: THEME, variablePrefix: '--syntax-', fontStyle: false })],
    langs: [],
    // Plain JS regexes, not the Oniguruma WASM build. `forgiving` leaves a pattern the
    // JS engine can't compile unmatched instead of failing the whole grammar.
    engine: createJavaScriptRegexEngine({ forgiving: true }),
  });
  return highlighter;
}

// The grammar for a path: its extension (`ts`, `rs`), or the whole name for files
// that have none (`Dockerfile`, `Makefile`). Grammars are loaded on first use.
export function languageOf(path: string): BundledLanguage | null {
  const name = path.slice(path.lastIndexOf('/') + 1).toLowerCase();
  const key = name.includes('.') ? name.slice(name.lastIndexOf('.') + 1) : name;
  return Object.hasOwn(bundledLanguages, key) ? (key as BundledLanguage) : null;
}

// Each line of `text` as coloured tokens, indexed by line number - 1. Null when the
// path has no known grammar or the file is too big to be worth it.
export async function highlightLines(path: string, text: string): Promise<SyntaxToken[][] | null> {
  const lang = languageOf(path);
  if (!lang || text.length > MAX_HIGHLIGHT_CHARS) return null;
  const hl = await getHighlighter();
  await hl.loadLanguage(bundledLanguages[lang]);
  return hl
    .codeToTokensBase(text, { lang, theme: THEME })
    .map((line) => line.map((t) => ({ text: t.content, color: t.color })));
}

// A run of a line as it is drawn: one colour, and whether it is part of what changed
// within the line.
export type LinePiece = { text: string; changed: boolean; color?: string };

// Lays a line's syntax colours and its within-line changes over each other, cutting
// wherever either one changes. Tokens that don't spell out `text` exactly — Shiki and
// git split lines differently on a lone `\r` — are dropped rather than drawn, since
// what they spell is not the line.
export function paintLine(text: string, tokens?: SyntaxToken[], segments?: InlineSegment[]): LinePiece[] {
  const colors = tokens && tokens.map((t) => t.text).join('') === text ? tokens : [{ text }];
  const marks = segments ?? [{ text, changed: false }];
  const out: LinePiece[] = [];
  let ci = 0;
  let mi = 0;
  let cOff = 0;
  let mOff = 0;
  while (ci < colors.length && mi < marks.length) {
    const c = colors[ci];
    const m = marks[mi];
    const n = Math.min(c.text.length - cOff, m.text.length - mOff);
    if (n > 0) out.push({ text: c.text.slice(cOff, cOff + n), changed: m.changed, color: c.color });
    cOff += n;
    mOff += n;
    if (cOff === c.text.length) {
      ci++;
      cOff = 0;
    }
    if (mOff === m.text.length) {
      mi++;
      mOff = 0;
    }
  }
  return out;
}
