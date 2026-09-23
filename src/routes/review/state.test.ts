// reviewState answers every load against a sequence number and drops the response when
// that number has moved on (state.svelte.ts:56-61). Each of those guards protects the
// same failure: the reviewer switches away, the slow reply for what they left lands on
// top of what they are looking at now, and the screen shows the previous Repo's content
// while claiming to show this one. Nothing about that looks broken, so it needs a test.
//
// Every test here resolves the stale load first, while the current one is still in
// flight — that is the order the guards exist for, and the one where a dropped guard
// also takes the spinner down or raises a toast about a Repo nobody is looking at.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { BranchList, FileContent, FileDiff, TreeNode } from './types';

vi.mock('./api', () => ({
  listRepos: vi.fn(),
  addRepo: vi.fn(),
  removeRepo: vi.fn(),
  pickRepoFolder: vi.fn(),
  listBranches: vi.fn(),
  getFileTree: vi.fn(),
  getFileDiff: vi.fn(),
  getFileContent: vi.fn(),
  fetchRemote: vi.fn(),
}));

vi.mock('$lib/toast/state.svelte', () => ({ toast: vi.fn() }));

type State = typeof import('./state.svelte').reviewState;
type Api = { [K in keyof typeof import('./api')]: ReturnType<typeof vi.fn> };

let reviewState: State;
let api: Api;
let toast: ReturnType<typeof vi.fn>;

beforeEach(async () => {
  vi.resetModules();
  vi.clearAllMocks();
  api = (await import('./api')) as unknown as Api;
  ({ toast } = (await import('$lib/toast/state.svelte')) as unknown as {
    toast: ReturnType<typeof vi.fn>;
  });
  ({ reviewState } = await import('./state.svelte'));
});

/** A promise the test resolves by hand, so two loads can be left in flight at once. */
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const branchList = (name: string, detachedHead: string | null = null): BranchList => ({
  branches: [{ name, isCurrent: true, ahead: null, behind: null }],
  detachedHead,
});

const fileNode = (path: string, changed = true): TreeNode => ({
  type: 'file',
  name: path,
  path,
  ...(changed ? { changes: { add: 1, del: 0 } } : {}),
  renamedFrom: null,
});

const diff = (header: string): FileDiff => ({ hunks: [{ header, lines: [] }], binary: false });

const content = (line: string): FileContent => ({ lines: [line], binary: false });

/** Gets to the state the guards matter in: a Repo open on its checked-out branch. */
async function openRepo(id: string, tree: TreeNode[] = []) {
  api.listBranches.mockResolvedValueOnce(branchList('main'));
  api.getFileTree.mockResolvedValueOnce(tree);
  api.getFileDiff.mockResolvedValue(diff('@@ initial @@'));
  api.getFileContent.mockResolvedValue(content('initial'));
  await reviewState.selectRepo(id);
  vi.clearAllMocks();
}

describe('branch load', () => {
  it('drops the branches of a Repo the reviewer has already left', async () => {
    const stale = deferred<BranchList>();
    const current = deferred<BranchList>();
    api.listBranches.mockReturnValueOnce(stale.promise).mockReturnValueOnce(current.promise);
    api.getFileTree.mockResolvedValue([]);

    const left = reviewState.selectRepo('a');
    const stayed = reviewState.selectRepo('b');

    stale.resolve(branchList('a-branch', 'a-head'));
    await left;

    expect(reviewState.branches).toEqual([]);
    expect(reviewState.detachedHead).toBeNull();
    expect(reviewState.loadingBranches).toBe(true);

    current.resolve(branchList('b-branch', 'b-head'));
    await stayed;

    expect(reviewState.branches.map((b) => b.name)).toEqual(['b-branch']);
    expect(reviewState.detachedHead).toBe('b-head');
    expect(reviewState.loadingBranches).toBe(false);
  });

  it('says nothing about a Repo the reviewer has already left failing to load', async () => {
    const stale = deferred<BranchList>();
    const current = deferred<BranchList>();
    api.listBranches.mockReturnValueOnce(stale.promise).mockReturnValueOnce(current.promise);
    api.getFileTree.mockResolvedValue([]);

    const left = reviewState.selectRepo('a');
    const stayed = reviewState.selectRepo('b');

    stale.reject(new Error('repo a is gone'));
    await left;

    expect(toast).not.toHaveBeenCalled();
    expect(reviewState.loadingBranches).toBe(true);

    current.resolve(branchList('b-branch'));
    await stayed;

    expect(reviewState.branch).toBe('b-branch');
  });
});

describe('tree load', () => {
  it('drops the tree of a Diff Mode the reviewer has already left', async () => {
    await openRepo('a');
    const stale = deferred<TreeNode[]>();
    const current = deferred<TreeNode[]>();
    api.getFileTree.mockReturnValueOnce(stale.promise).mockReturnValueOnce(current.promise);

    const left = reviewState.reloadTree();
    const stayed = reviewState.setDiffMode('staged');

    stale.resolve([fileNode('stale.ts')]);
    await left;

    expect(reviewState.tree).toEqual([]);
    expect(reviewState.loadingTree).toBe(true);

    current.resolve([fileNode('current.ts')]);
    await stayed;

    expect(reviewState.tree.map((n) => n.path)).toEqual(['current.ts']);
    expect(reviewState.selectedFile).toBe('current.ts');
    expect(reviewState.loadingTree).toBe(false);
  });

  it('drops the tree of a Repo that was removed while it was loading', async () => {
    await openRepo('a');
    const stale = deferred<TreeNode[]>();
    api.getFileTree.mockReturnValueOnce(stale.promise);
    api.removeRepo.mockResolvedValue(undefined);

    const left = reviewState.reloadTree();
    await reviewState.removeRepo('a');

    stale.resolve([fileNode('stale.ts')]);
    await left;

    expect(reviewState.tree).toEqual([]);
    expect(reviewState.selectedFile).toBeNull();
  });

  it('says nothing about a tree the reviewer has already left failing to load', async () => {
    await openRepo('a');
    const stale = deferred<TreeNode[]>();
    const current = deferred<TreeNode[]>();
    api.getFileTree.mockReturnValueOnce(stale.promise).mockReturnValueOnce(current.promise);

    const left = reviewState.reloadTree();
    const stayed = reviewState.setDiffMode('staged');

    stale.reject(new Error('repo a is gone'));
    await left;

    expect(toast).not.toHaveBeenCalled();
    expect(reviewState.loadingTree).toBe(true);

    current.resolve([]);
    await stayed;
  });

  it('drops a tree still loading when the reviewer switches Repo', async () => {
    await openRepo('a');
    const stale = deferred<TreeNode[]>();
    api.getFileTree.mockReturnValueOnce(stale.promise);
    const left = reviewState.reloadTree();

    // The Repo switched to is on a detached HEAD, so it has no branch under review and
    // loads no tree of its own (docs/decisions/0010) — selectRepo moving the sequence
    // on the way in is the only thing keeping the old Repo's tree out.
    api.listBranches.mockResolvedValueOnce({ branches: [], detachedHead: 'c0ffee' });
    await reviewState.selectRepo('b');

    stale.resolve([fileNode('stale.ts')]);
    await left;

    expect(reviewState.tree).toEqual([]);
  });
});

describe('file load', () => {
  it('drops the diff of a file the reviewer has already left', async () => {
    await openRepo('a', [fileNode('one.ts'), fileNode('two.ts')]);
    const stale = deferred<FileDiff>();
    const current = deferred<FileDiff>();
    api.getFileDiff.mockReturnValueOnce(stale.promise).mockReturnValueOnce(current.promise);

    const left = reviewState.selectFile('one.ts');
    const stayed = reviewState.selectFile('two.ts');

    stale.resolve(diff('@@ one @@'));
    await left;

    expect(reviewState.diffHunks).toEqual([]);
    expect(reviewState.loadingDiff).toBe(true);

    current.resolve(diff('@@ two @@'));
    await stayed;

    expect(reviewState.selectedFile).toBe('two.ts');
    expect(reviewState.diffHunks.map((h) => h.header)).toEqual(['@@ two @@']);
    expect(reviewState.loadingDiff).toBe(false);
  });

  it('drops the File View content of a file the reviewer has already left', async () => {
    await openRepo('a', [fileNode('one.ts', false), fileNode('two.ts', false)]);
    const stale = deferred<FileContent>();
    const current = deferred<FileContent>();
    api.getFileContent.mockReturnValueOnce(stale.promise).mockReturnValueOnce(current.promise);

    const left = reviewState.selectFile('one.ts');
    const stayed = reviewState.selectFile('two.ts');

    stale.resolve(content('one'));
    await left;

    expect(reviewState.fileLines).toEqual([]);
    expect(reviewState.loadingFile).toBe(true);

    current.resolve(content('two'));
    await stayed;

    expect(reviewState.selectedFile).toBe('two.ts');
    expect(reviewState.fileLines).toEqual(['two']);
    expect(reviewState.loadingFile).toBe(false);
  });

  it('says nothing about a diff the reviewer has already left failing to load', async () => {
    await openRepo('a', [fileNode('one.ts'), fileNode('two.ts')]);
    const stale = deferred<FileDiff>();
    const current = deferred<FileDiff>();
    api.getFileDiff.mockReturnValueOnce(stale.promise).mockReturnValueOnce(current.promise);

    const left = reviewState.selectFile('one.ts');
    const stayed = reviewState.selectFile('two.ts');

    stale.reject(new Error('one.ts is gone'));
    await left;

    expect(toast).not.toHaveBeenCalled();
    expect(reviewState.loadingDiff).toBe(true);

    current.resolve(diff('@@ two @@'));
    await stayed;
  });

  it('says nothing about File View content the reviewer has already left failing to load', async () => {
    await openRepo('a', [fileNode('one.ts', false), fileNode('two.ts', false)]);
    const stale = deferred<FileContent>();
    const current = deferred<FileContent>();
    api.getFileContent.mockReturnValueOnce(stale.promise).mockReturnValueOnce(current.promise);

    const left = reviewState.selectFile('one.ts');
    const stayed = reviewState.selectFile('two.ts');

    stale.reject(new Error('one.ts is gone'));
    await left;

    expect(toast).not.toHaveBeenCalled();
    expect(reviewState.loadingFile).toBe(true);

    current.resolve(content('two'));
    await stayed;
  });

  it('drops a diff still loading when the file leaves the tree', async () => {
    await openRepo('a', [fileNode('one.ts')]);
    const stale = deferred<FileDiff>();
    api.getFileDiff.mockReturnValueOnce(stale.promise);
    const left = reviewState.selectFile('one.ts');

    // The agent finished and one.ts no longer differs, so Refresh has nothing to keep
    // the reviewer on. Nothing selects a file on the way out, so the reload moving the
    // sequence itself is the only thing keeping the diff of a file that is gone from
    // landing on an empty selection.
    api.listBranches.mockResolvedValueOnce(branchList('main'));
    api.getFileTree.mockResolvedValueOnce([]);
    await reviewState.refresh();

    stale.resolve(diff('@@ stale @@'));
    await left;

    expect(reviewState.selectedFile).toBeNull();
    expect(reviewState.diffHunks).toEqual([]);
  });
});

describe('refresh', () => {
  it('follows a branch checked out in the terminal since the last load', async () => {
    await openRepo('a', [fileNode('one.ts')]);
    api.listBranches.mockResolvedValueOnce(branchList('feature'));
    api.getFileTree.mockResolvedValueOnce([fileNode('two.ts')]);

    await reviewState.refresh();

    expect(reviewState.branch).toBe('feature');
    expect(api.getFileTree).toHaveBeenCalledWith(expect.objectContaining({ branch: 'feature' }));
    expect(reviewState.tree.map((n) => n.path)).toEqual(['two.ts']);
  });

  it('drops the tree once the Repo is on a detached HEAD', async () => {
    await openRepo('a', [fileNode('one.ts')]);
    api.listBranches.mockResolvedValueOnce({ branches: [], detachedHead: 'c0ffee' });

    await reviewState.refresh();

    expect(reviewState.branch).toBeNull();
    expect(reviewState.detachedHead).toBe('c0ffee');
    expect(reviewState.tree).toEqual([]);
    expect(reviewState.selectedFile).toBeNull();
    expect(api.getFileTree).not.toHaveBeenCalled();
  });

  it('moves the Base Branch back to the default once it is the one checked out', async () => {
    reviewState.repos = [{ id: 'a', name: 'a', path: '/a', defaultBranch: 'main' }];
    await openRepo('a');
    await reviewState.setDiffMode('branch');
    api.getFileTree.mockResolvedValueOnce([]);
    await reviewState.setBaseBranch('develop');
    api.listBranches.mockResolvedValueOnce(branchList('develop'));
    api.getFileTree.mockResolvedValueOnce([]);

    await reviewState.refresh();

    expect(reviewState.branch).toBe('develop');
    expect(reviewState.baseBranch).toBe('main');
  });

  it('keeps the default Base Branch when the default branch itself is checked out', async () => {
    reviewState.repos = [{ id: 'a', name: 'a', path: '/a', defaultBranch: 'main' }];
    api.listBranches.mockResolvedValueOnce(branchList('feature'));
    api.getFileTree.mockResolvedValueOnce([]);
    await reviewState.selectRepo('a');
    api.listBranches.mockResolvedValueOnce(branchList('main'));
    api.getFileTree.mockResolvedValueOnce([]);

    await reviewState.refresh();

    expect(reviewState.branch).toBe('main');
    expect(reviewState.baseBranch).toBe('main');
  });
});
