import type { Branch, DiffHunk, DiffLine, TreeNode } from './types';

export function pruneToChanged(nodes: TreeNode[]): TreeNode[] {
  return nodes.reduce<TreeNode[]>((acc, n) => {
    if (n.type === 'file') {
      if (n.changes) acc.push(n);
      return acc;
    }
    const children = pruneToChanged(n.children);
    if (children.length) acc.push({ ...n, children });
    return acc;
  }, []);
}

export function pruneByName(nodes: TreeNode[], query: string): TreeNode[] {
  return nodes.reduce<TreeNode[]>((acc, n) => {
    if (n.type === 'file') {
      if (n.name.toLowerCase().includes(query)) acc.push(n);
      return acc;
    }
    const children = pruneByName(n.children, query);
    if (children.length) acc.push({ ...n, children });
    return acc;
  }, []);
}

export function firstFilePath(nodes: TreeNode[]): string | null {
  for (const n of nodes) {
    if (n.type === 'file') return n.path;
    const found = firstFilePath(n.children);
    if (found) return found;
  }
  return null;
}

export function allDirPaths(nodes: TreeNode[]): string[] {
  const paths: string[] = [];
  const walk = (ns: TreeNode[]) =>
    ns.forEach((n) => {
      if (n.type === 'dir') {
        paths.push(n.path);
        walk(n.children);
      }
    });
  walk(nodes);
  return paths;
}

export function branchMeta(b: Branch): string | undefined {
  if (b.isCurrent) return `目前分支${b.ahead ? ` · 領先 ${b.ahead} 個 commit` : ''}`;
  if (b.behind) return `落後 ${b.behind} 個 commit`;
  return undefined;
}

export function rangeLabel(lineStart: number, lineEnd?: number) {
  return lineEnd && lineEnd !== lineStart ? `L${lineStart}–L${lineEnd}` : `L${lineStart}`;
}

// A line with its position among all lines of the file being reviewed (flattened
// across hunks), used to anchor comment threads and gutter drag-selection.
export type IndexedLine = DiffLine & { idx: number };

// Same, but keeping the owning hunk: line numbers are only contiguous within a
// hunk, so a selection may not run across a hunk boundary.
export type FlatLine = { line: DiffLine; idx: number; hunk: number };

export function flattenHunks(hunks: DiffHunk[]): FlatLine[] {
  const out: FlatLine[] = [];
  let idx = 0;
  hunks.forEach((h, hunk) => {
    for (const line of h.lines) out.push({ line, idx: idx++, hunk });
  });
  return out;
}

// Line numbers for a selected idx range. Both ends are read off the same side of
// the diff — the new side when the selection touches it at all, the old side for a
// pure-deletion selection — so the range can never come out reversed.
export function lineRange(lines: FlatLine[], lo: number, hi: number): { lineStart: number; lineEnd?: number } {
  const selected = lines.slice(lo, hi + 1).map((f) => f.line);
  const isNo = (n: number | null): n is number => n !== null;
  const newNos = selected.map((l) => l.newNo).filter(isNo);
  const nos = newNos.length ? newNos : selected.map((l) => l.oldNo).filter(isNo);
  const lineStart = nos[0] ?? 0;
  const lineEnd = nos[nos.length - 1] ?? lineStart;
  return { lineStart, lineEnd: lineEnd === lineStart ? undefined : lineEnd };
}

export type SplitSide = { kind: DiffLine['kind']; no: number | null; text: string; idx: number; commentable?: boolean } | null;

// Only the right (new-side) column exposes the "add comment" affordance, matching
// DiffLineSplit's single onAddComment callback — there's no per-side target to
// route a left-side click to.
export function pairHunkLines(lines: IndexedLine[]): { left: SplitSide; right: SplitSide }[] {
  const result: { left: SplitSide; right: SplitSide }[] = [];
  let i = 0;
  while (i < lines.length) {
    const l = lines[i];
    if (l.kind === 'context') {
      result.push({
        left: { kind: 'context', no: l.oldNo, text: l.content, idx: l.idx },
        right: { kind: 'context', no: l.newNo, text: l.content, idx: l.idx, commentable: true },
      });
      i++;
      continue;
    }
    const dels: IndexedLine[] = [];
    while (i < lines.length && lines[i].kind === 'del') {
      dels.push(lines[i]);
      i++;
    }
    const adds: IndexedLine[] = [];
    while (i < lines.length && lines[i].kind === 'add') {
      adds.push(lines[i]);
      i++;
    }
    const max = Math.max(dels.length, adds.length);
    for (let j = 0; j < max; j++) {
      const d = dels[j];
      const a = adds[j];
      result.push({
        left: d ? { kind: 'del', no: d.oldNo, text: d.content, idx: d.idx } : null,
        right: a ? { kind: 'add', no: a.newNo, text: a.content, idx: a.idx, commentable: true } : null,
      });
    }
  }
  return result;
}
