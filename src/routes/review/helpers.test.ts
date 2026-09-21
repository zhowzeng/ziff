import { describe, expect, it } from 'vitest';
import {
  flattenHunks,
  lineRange,
  pairHunkLines,
  pruneByName,
  pruneToChanged,
  type IndexedLine,
} from './helpers';
import type { DiffHunk, DiffLine, TreeNode } from './types';

const ctx = (oldNo: number, newNo: number, content = 'ctx'): DiffLine => ({
  kind: 'context',
  oldNo,
  newNo,
  content,
});
const del = (oldNo: number, content = 'del'): DiffLine => ({
  kind: 'del',
  oldNo,
  newNo: null,
  content,
});
const add = (newNo: number, content = 'add'): DiffLine => ({
  kind: 'add',
  oldNo: null,
  newNo,
  content,
});

const hunk = (lines: DiffLine[], header = '@@'): DiffHunk => ({ header, lines });

// pairHunkLines takes the lines of one hunk already carrying their flat idx.
const numbered = (lines: DiffLine[]): IndexedLine[] => lines.map((line, idx) => ({ ...line, idx }));

const file = (name: string, changes?: { add: number; del: number }): TreeNode => ({
  type: 'file',
  name,
  path: name,
  changes,
  renamedFrom: null,
});
const dir = (name: string, children: TreeNode[]): TreeNode => ({
  type: 'dir',
  name,
  path: name,
  children,
});

describe('pairHunkLines', () => {
  it('puts a context line on both sides, commentable only on the right', () => {
    const [row] = pairHunkLines(numbered([ctx(10, 12)]));
    expect(row.left).toEqual({ kind: 'context', no: 10, text: 'ctx', idx: 0 });
    expect(row.right).toEqual({ kind: 'context', no: 12, text: 'ctx', idx: 0, commentable: true });
  });

  it('pairs a del run with the add run that follows it', () => {
    const rows = pairHunkLines(numbered([del(5), del(6), add(5), add(6)]));
    expect(rows).toHaveLength(2);
    expect(rows.map((r) => [r.left?.no, r.right?.no])).toEqual([
      [5, 5],
      [6, 6],
    ]);
  });

  it('leaves the right side empty for the dels a shorter add run cannot cover', () => {
    const rows = pairHunkLines(numbered([del(5), del(6), del(7), add(5)]));
    expect(rows).toHaveLength(3);
    expect(rows[0].right?.no).toBe(5);
    expect(rows[1].right).toBeNull();
    expect(rows[2].right).toBeNull();
    expect(rows.map((r) => r.left?.no)).toEqual([5, 6, 7]);
  });

  it('leaves the left side empty for the adds a shorter del run cannot cover', () => {
    const rows = pairHunkLines(numbered([del(5), add(5), add(6), add(7)]));
    expect(rows).toHaveLength(3);
    expect(rows[0].left?.no).toBe(5);
    expect(rows[1].left).toBeNull();
    expect(rows[2].left).toBeNull();
    expect(rows.map((r) => r.right?.no)).toEqual([5, 6, 7]);
  });

  it('renders a pure add section with nothing on the left', () => {
    const rows = pairHunkLines(numbered([add(1), add(2)]));
    expect(rows.map((r) => r.left)).toEqual([null, null]);
    expect(rows.map((r) => r.right?.commentable)).toEqual([true, true]);
  });

  it('renders a pure del section with nothing on the right', () => {
    const rows = pairHunkLines(numbered([del(1), del(2)]));
    expect(rows.map((r) => r.right)).toEqual([null, null]);
    expect(rows.map((r) => r.left?.no)).toEqual([1, 2]);
  });

  it('starts a new pairing when adds come before dels', () => {
    const rows = pairHunkLines(numbered([add(1), del(1)]));
    expect(rows).toHaveLength(2);
    expect(rows[0]).toEqual({ left: null, right: { kind: 'add', no: 1, text: 'add', idx: 0, commentable: true } });
    expect(rows[1]).toEqual({ left: { kind: 'del', no: 1, text: 'del', idx: 1 }, right: null });
  });

  it('keeps the flat idx of every line it pairs', () => {
    const rows = pairHunkLines(numbered([ctx(1, 1), del(2), add(2), ctx(3, 3)]));
    expect(rows.flatMap((r) => [r.left?.idx, r.right?.idx].filter((i) => i !== undefined))).toEqual([
      0, 0, 1, 2, 3, 3,
    ]);
  });
});

describe('lineRange', () => {
  it('drops lineEnd for a single-line selection', () => {
    const lines = flattenHunks([hunk([ctx(10, 12)])]);
    expect(lineRange(lines, 0, 0)).toEqual({ lineStart: 12, lineEnd: undefined, side: 'new' });
  });

  it('reads both ends off the new side, so a swept del cannot reverse the range', () => {
    // ctx 12 / del (old side only) / add 13 — the del has no new-side number to read.
    const lines = flattenHunks([hunk([ctx(11, 12), del(12), add(13)])]);
    expect(lineRange(lines, 0, 2)).toEqual({ lineStart: 12, lineEnd: 13, side: 'new' });
  });

  it('drops lineEnd when the swept lines all land on one new-side number', () => {
    const lines = flattenHunks([hunk([add(13), del(12), del(13)])]);
    expect(lineRange(lines, 0, 2)).toEqual({ lineStart: 13, lineEnd: undefined, side: 'new' });
  });
});

describe('flattenHunks', () => {
  it('numbers lines continuously across hunks while keeping the owning hunk', () => {
    const flat = flattenHunks([hunk([ctx(1, 1), add(2)]), hunk([ctx(40, 41)])]);
    expect(flat.map((f) => f.idx)).toEqual([0, 1, 2]);
    // The hunk id is what the diff panel compares to keep a drag inside one hunk.
    expect(flat.map((f) => f.hunk)).toEqual([0, 0, 1]);
  });

  it('returns nothing for a file with no hunks', () => {
    expect(flattenHunks([])).toEqual([]);
  });
});

describe('pruneToChanged', () => {
  it('keeps only files that changed, and the dirs leading to them', () => {
    const tree = [
      dir('src', [
        dir('routes', [file('+page.svelte', { add: 3, del: 1 }), file('types.ts')]),
        dir('lib', [file('untouched.ts')]),
      ]),
      file('README.md'),
    ];
    expect(pruneToChanged(tree)).toEqual([
      dir('src', [dir('routes', [file('+page.svelte', { add: 3, del: 1 })])]),
    ]);
  });

  it('leaves the tree it was given alone', () => {
    const tree = [dir('src', [file('a.ts', { add: 1, del: 0 }), file('b.ts')])];
    pruneToChanged(tree);
    expect(tree[0].type === 'dir' && tree[0].children).toHaveLength(2);
  });
});

describe('pruneByName', () => {
  it('matches file names case-insensitively and keeps the dirs above them', () => {
    const tree = [dir('src', [dir('routes', [file('Topbar.svelte'), file('state.svelte.ts')])])];
    expect(pruneByName(tree, 'topbar')).toEqual([dir('src', [dir('routes', [file('Topbar.svelte')])])]);
  });

  it('drops a dir whose name matches but whose files do not', () => {
    const tree = [dir('review', [file('a.ts')])];
    expect(pruneByName(tree, 'review')).toEqual([]);
  });
});
