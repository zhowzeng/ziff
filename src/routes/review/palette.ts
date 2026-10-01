// What the Command Palette lists: the changed files, narrowed by what the reviewer typed.

import type { TreeNode } from './types';

export interface PaletteEntry {
  path: string;
  name: string;
  /** Everything up to and including the last `/`, or empty for a top-level file. */
  dir: string;
  add?: number;
  del?: number;
}

export function flattenFiles(nodes: TreeNode[]): PaletteEntry[] {
  return nodes.flatMap((n) => {
    if (n.type === 'dir') return flattenFiles(n.children);
    const slash = n.path.lastIndexOf('/');
    return [
      {
        path: n.path,
        name: n.name,
        dir: slash >= 0 ? n.path.slice(0, slash + 1) : '',
        add: n.changes?.add,
        del: n.changes?.del,
      },
    ];
  });
}

// Every typed character has to appear in the path, in order — a light subsequence match.
// A hit inside the file name outranks one that only matches along the folders, and among
// equals the shorter path comes first. An empty query keeps the sidebar's order.
export function filterFiles(entries: PaletteEntry[], query: string): PaletteEntry[] {
  const q = query.trim().toLowerCase();
  if (!q) return entries;
  const scored: [PaletteEntry, number][] = [];
  for (const e of entries) {
    const path = e.path.toLowerCase();
    let i = 0;
    for (const ch of path) if (ch === q[i]) i++;
    if (i < q.length) continue;
    scored.push([e, (e.name.toLowerCase().includes(q) ? 0 : 10) + path.length / 100]);
  }
  return scored.sort((a, b) => a[1] - b[1]).map(([e]) => e);
}
