import { describe, expect, it } from 'vitest';
import { filterFiles, flattenFiles } from './palette';
import type { TreeNode } from './types';

const file = (path: string, add = 1, del = 0): TreeNode => ({
  type: 'file',
  name: path.slice(path.lastIndexOf('/') + 1),
  path,
  changes: { add, del },
  renamedFrom: null,
});

const tree: TreeNode[] = [
  {
    type: 'dir',
    name: 'src',
    path: 'src',
    children: [
      { type: 'dir', name: 'auth', path: 'src/auth', children: [file('src/auth/session.ts', 42, 7)] },
      file('src/router.ts'),
    ],
  },
  file('README.md'),
];

describe('flattenFiles', () => {
  it('lists files in tree order, splitting the folder from the name', () => {
    expect(flattenFiles(tree).map((e) => [e.dir, e.name])).toEqual([
      ['src/auth/', 'session.ts'],
      ['src/', 'router.ts'],
      ['', 'README.md'],
    ]);
  });

  it('carries the change counts', () => {
    expect(flattenFiles(tree)[0]).toMatchObject({ add: 42, del: 7 });
  });
});

describe('filterFiles', () => {
  const entries = flattenFiles(tree);

  it('keeps the sidebar order for an empty or blank query', () => {
    expect(filterFiles(entries, '')).toEqual(entries);
    expect(filterFiles(entries, '   ')).toEqual(entries);
  });

  it('matches characters in order, ignoring case', () => {
    expect(filterFiles(entries, 'SSN').map((e) => e.path)).toEqual(['src/auth/session.ts']);
  });

  it('drops files that do not contain every character in order', () => {
    expect(filterFiles(entries, 'zzz')).toEqual([]);
    expect(filterFiles(entries, 'ts.r')).toEqual([]);
  });

  it('ranks a file-name hit above a folder-only hit', () => {
    const list = flattenFiles([file('auth/x.ts'), file('lib/auth.ts')]);
    expect(filterFiles(list, 'auth').map((e) => e.path)).toEqual(['lib/auth.ts', 'auth/x.ts']);
  });
});
