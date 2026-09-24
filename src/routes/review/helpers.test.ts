import { describe, expect, it } from 'vitest';
import {
  adjacentChangedFile,
  fileAfterReload,
  flattenHunks,
  formatForAgent,
  formatQueueForAgent,
  lineRange,
  inlineDiff,
  inlineSegments,
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

describe('fileAfterReload', () => {
  const tree = [
    dir('src', [file('a.ts', { add: 1, del: 0 }), file('b.ts', { add: 2, del: 0 })]),
    file('README.md'),
  ];

  it('keeps the reviewer on the file they were reading', () => {
    expect(fileAfterReload(tree, 'b.ts')).toBe('b.ts');
  });

  it('keeps it even when the agent reverted it and it has no changes left', () => {
    expect(fileAfterReload([file('a.ts'), file('b.ts', { add: 1, del: 0 })], 'a.ts')).toBe('a.ts');
  });

  it('falls back to the first changed file once that file is gone from the tree', () => {
    expect(fileAfterReload(tree, 'deleted.ts')).toBe('a.ts');
  });

  it('opens on the first changed file, not the first file, with nothing to keep', () => {
    expect(fileAfterReload(tree, null)).toBe('a.ts');
  });

  it('has nothing to open when the tree has no changed file left', () => {
    expect(fileAfterReload([file('README.md')], 'deleted.ts')).toBeNull();
  });
});

describe('adjacentChangedFile', () => {
  const tree = [
    dir('src', [file('a.ts', { add: 1, del: 0 }), file('b.ts'), file('c.ts', { add: 2, del: 0 })]),
    file('d.ts', { add: 1, del: 1 }),
  ];

  it('moves through changed files in sidebar order, skipping unchanged ones', () => {
    expect(adjacentChangedFile(tree, 'a.ts', 1)).toBe('c.ts');
    expect(adjacentChangedFile(tree, 'c.ts', 1)).toBe('d.ts');
    expect(adjacentChangedFile(tree, 'c.ts', -1)).toBe('a.ts');
  });

  it('stops at either end instead of wrapping', () => {
    expect(adjacentChangedFile(tree, 'd.ts', 1)).toBeNull();
    expect(adjacentChangedFile(tree, 'a.ts', -1)).toBeNull();
  });

  it('starts from the end it is heading away from when no changed file is open', () => {
    expect(adjacentChangedFile(tree, null, 1)).toBe('a.ts');
    expect(adjacentChangedFile(tree, null, -1)).toBe('d.ts');
    expect(adjacentChangedFile(tree, 'b.ts', 1)).toBe('a.ts');
  });

  it('has nowhere to go without changed files', () => {
    expect(adjacentChangedFile([file('README.md')], null, 1)).toBeNull();
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

describe('formatQueueForAgent', () => {
  const items = [
    { file: 'src/a.ts', lineStart: 3, text: 'rename this' },
    { file: 'src/b.ts', lineStart: 10, lineEnd: 12, text: 'extract a helper' },
  ];

  it('joins every comment in the queue with a blank line between them', () => {
    expect(formatQueueForAgent(items)).toBe(
      'src/a.ts:L3\nrename this\n\nsrc/b.ts:L10-L12\nextract a helper'
    );
  });

  it('puts the prefix in front once, not once per comment', () => {
    expect(formatQueueForAgent(items, 'Apply these:')).toBe(
      'Apply these:\n\nsrc/a.ts:L3\nrename this\n\nsrc/b.ts:L10-L12\nextract a helper'
    );
  });

  it('leaves out the prefix when there is none', () => {
    expect(formatQueueForAgent([items[0]], '')).toBe('src/a.ts:L3\nrename this');
  });
});

describe('formatForAgent', () => {
  const orphaned = {
    file: 'src/foo.rs',
    lineStart: 12,
    text: '這裡應該用 open_workbook_auto',
    orphaned: { anchorText: ['let x = foo()?;'], staged: false },
  };

  it('quotes the lines instead of a line number once they are gone', () => {
    expect(formatForAgent(orphaned)).toBe(
      'src/foo.rs (commented at L12; that code is no longer in the file)\n' +
        '> let x = foo()?;\n這裡應該用 open_workbook_auto'
    );
  });

  it('says the working tree moved when the comment was written against the index', () => {
    expect(formatForAgent({ ...orphaned, orphaned: { ...orphaned.orphaned, staged: true } })).toBe(
      'src/foo.rs (commented at L12 as staged; the working tree has since changed)\n' +
        '> let x = foo()?;\n這裡應該用 open_workbook_auto'
    );
  });

  it('prefixes every quoted line, so a multi-line comment stays apart from the code', () => {
    const text = formatForAgent({
      ...orphaned,
      lineEnd: 13,
      text: 'first line\nsecond line',
      orphaned: { anchorText: ['let x = foo()?;', 'let y = bar()?;'], staged: false },
    });
    expect(text).toBe(
      'src/foo.rs (commented at L12-L13; that code is no longer in the file)\n' +
        '> let x = foo()?;\n> let y = bar()?;\nfirst line\nsecond line'
    );
  });
});

const marked = (segs: { text: string; changed: boolean }[]) => segs.filter((s) => s.changed).map((s) => s.text);

describe('inlineDiff', () => {
  it('marks only the words that changed on each side', () => {
    const diff = inlineDiff('const x = foo(bar);', 'const x = foo(baz);');
    expect(diff && marked(diff.old)).toEqual(['bar']);
    expect(diff && marked(diff.new)).toEqual(['baz']);
  });

  it('keeps the whole line text across its segments', () => {
    const diff = inlineDiff('a b c', 'a x c');
    expect(diff?.old.map((s) => s.text).join('')).toBe('a b c');
    expect(diff?.new.map((s) => s.text).join('')).toBe('a x c');
  });

  it('marks the space between two changed words with them', () => {
    const diff = inlineDiff('available: list existing files', 'available: only the files');
    expect(diff && marked(diff.new)).toEqual(['only the']);
  });

  it('marks nothing when the lines share no word', () => {
    expect(inlineDiff('alpha beta', 'gamma delta')).toBeNull();
  });
});

describe('inlineSegments', () => {
  it('compares the n-th del of a run with the n-th add after it', () => {
    const segs = inlineSegments(numbered([ctx(1, 1), del(2, 'let a = 1;'), del(3, 'x'), add(2, 'let a = 2;')]));
    expect(marked(segs.get(1)!)).toEqual(['1']);
    expect(marked(segs.get(3)!)).toEqual(['2']);
    // The unpaired del and the context line have nothing to compare against.
    expect(segs.has(0)).toBe(false);
    expect(segs.has(2)).toBe(false);
  });
});
