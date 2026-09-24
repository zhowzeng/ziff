// Syntax highlighting for the diff and File View (docs/decisions/0017). Shiki runs in
// a worker (highlight.worker.ts), so nothing of it is loaded on the main thread.

import type { InlineSegment } from './helpers';
import type { HighlightReply, HighlightRequest } from './highlight.worker';

// A run of one line's text in one colour. The colour is a `var(--syntax-*)` reference,
// so the palette lives with the other design tokens in colors.css.
export type SyntaxToken = { text: string; color?: string };

// Past this a file is plain text. Off the main thread it no longer freezes the window,
// but a multi-megabyte lockfile or bundle would still keep the worker busy for tens of
// seconds, and every file opened after it would wait that long for its colours.
const MAX_HIGHLIGHT_CHARS = 500_000;

let worker: Worker | null = null;
let nextId = 0;
const pending = new Map<number, { resolve: (tokens: SyntaxToken[][] | null) => void; reject: (e: Error) => void }>();

function getWorker(): Worker {
  if (worker) return worker;
  worker = new Worker(new URL('./highlight.worker.ts', import.meta.url), { type: 'module' });
  worker.onmessage = (e: MessageEvent<HighlightReply>) => {
    const reply = e.data;
    const request = pending.get(reply.id);
    pending.delete(reply.id);
    if ('error' in reply) request?.reject(new Error(reply.error));
    else request?.resolve(reply.tokens);
  };
  // A worker that failed to load answers nothing, so everything waiting on it ends
  // here as plain text rather than never.
  worker.onerror = () => {
    for (const request of pending.values()) request.reject(new Error('highlight worker failed'));
    pending.clear();
  };
  return worker;
}

// Each line of `text` as coloured tokens, indexed by line number - 1. Null when the
// path has no known grammar or the file is too big to be worth it.
export function highlightLines(path: string, text: string): Promise<SyntaxToken[][] | null> {
  if (text.length > MAX_HIGHLIGHT_CHARS) return Promise.resolve(null);
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    getWorker().postMessage({ id, path, text } satisfies HighlightRequest);
  });
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
