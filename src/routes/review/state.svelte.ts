import { toast } from '$lib/toast/state.svelte';
import { fetchRemote, getFileDiff, getFileTree, listBranches, listRepos } from './api';
import { firstFilePath, pruneToChanged } from './helpers';
import type { Branch, DiffHunk, DiffMode, DiffSpec, Repo, TreeNode } from './types';

export type ViewMode = 'unified' | 'split';

class ReviewState {
  repos = $state<Repo[]>([]);
  repoId = $state<string | null>(null);
  branches = $state<Branch[]>([]);
  branch = $state<string | null>(null);
  diffMode = $state<DiffMode>('unstaged');
  view = $state<ViewMode>('unified');

  tree = $state<TreeNode[]>([]);
  selectedFile = $state<string | null>(null);
  diffHunks = $state<DiffHunk[]>([]);

  loadingRepos = $state(false);
  loadingBranches = $state(false);
  loadingTree = $state(false);
  loadingDiff = $state(false);
  fetching = $state(false);
  lastFetched = $state<string | null>(null);

  // Bumped on every load. A response only lands if its sequence number is still
  // the current one, so a slow reply for a repo/branch/file the reviewer has
  // already switched away from can't overwrite the newer state.
  #branchSeq = 0;
  #treeSeq = 0;
  #diffSeq = 0;

  repo = $derived(this.repos.find((r) => r.id === this.repoId) ?? null);

  spec = $derived.by<DiffSpec | null>(() => {
    if (!this.repoId || !this.branch) return null;
    const spec: DiffSpec = { repoId: this.repoId, branch: this.branch, diffMode: this.diffMode };
    // Branch mode needs something to compare against. Until there's a base-branch
    // picker it's the repo's default branch (CONTEXT.md: Base Branch).
    if (this.diffMode === 'branch' && this.repo) spec.baseBranch = this.repo.defaultBranch;
    return spec;
  });

  async loadRepos() {
    this.loadingRepos = true;
    try {
      this.repos = await listRepos();
      if (this.repos.length) await this.selectRepo(this.repos[0].id);
    } catch (e) {
      toast(`載入 repo 清單失敗：${e}`, { variant: 'danger' });
    } finally {
      this.loadingRepos = false;
    }
  }

  async selectRepo(id: string) {
    this.repoId = id;
    this.branch = null;
    this.branches = [];
    this.tree = [];
    this.selectedFile = null;
    this.diffHunks = [];
    this.#treeSeq++;
    this.#diffSeq++;

    const seq = ++this.#branchSeq;
    this.loadingBranches = true;
    try {
      const branches = await listBranches(id);
      if (seq !== this.#branchSeq) return;
      this.branches = branches;
      const current = branches.find((b) => b.isCurrent) ?? branches[0];
      if (current) await this.selectBranch(current.name);
    } catch (e) {
      if (seq !== this.#branchSeq) return;
      toast(`載入分支清單失敗：${e}`, { variant: 'danger' });
    } finally {
      if (seq === this.#branchSeq) this.loadingBranches = false;
    }
  }

  async selectBranch(name: string) {
    this.branch = name;
    await this.reloadTree();
  }

  async setDiffMode(mode: DiffMode) {
    this.diffMode = mode;
    await this.reloadTree();
  }

  async reloadTree() {
    const spec = this.spec;
    if (!spec) return;
    const seq = ++this.#treeSeq;
    this.#diffSeq++;
    this.selectedFile = null;
    this.diffHunks = [];
    this.loadingTree = true;
    try {
      const tree = await getFileTree(spec);
      if (seq !== this.#treeSeq) return;
      this.tree = tree;
      // Only changed files are selectable — the rest of the tree has no diff to show.
      const firstPath = firstFilePath(pruneToChanged(tree));
      if (firstPath) await this.selectFile(firstPath);
    } catch (e) {
      if (seq !== this.#treeSeq) return;
      toast(`載入變更檔案清單失敗：${e}`, { variant: 'danger' });
    } finally {
      if (seq === this.#treeSeq) this.loadingTree = false;
    }
  }

  async selectFile(path: string) {
    const seq = ++this.#diffSeq;
    this.selectedFile = path;
    this.diffHunks = [];
    const spec = this.spec;
    if (!spec) return;
    this.loadingDiff = true;
    try {
      const hunks = await getFileDiff(spec, path);
      if (seq !== this.#diffSeq) return;
      this.diffHunks = hunks;
    } catch (e) {
      if (seq !== this.#diffSeq) return;
      toast(`載入 diff 失敗：${e}`, { variant: 'danger' });
    } finally {
      if (seq === this.#diffSeq) this.loadingDiff = false;
    }
  }

  async fetchRemoteBranch() {
    if (!this.repoId || this.fetching) return;
    this.fetching = true;
    try {
      const result = await fetchRemote(this.repoId);
      if (!result.success) {
        toast(result.message, { variant: 'danger' });
        return;
      }
      this.lastFetched = new Date().toLocaleString('zh-TW', { hour12: false });
      toast(result.message, { variant: 'success' });
      await this.#reloadBranches();
      await this.reloadTree();
    } catch (e) {
      toast(`Fetch 失敗：${e}`, { variant: 'danger' });
    } finally {
      this.fetching = false;
    }
  }

  // Refreshes ahead/behind counts after a fetch, keeping the current selection.
  async #reloadBranches() {
    const id = this.repoId;
    if (!id) return;
    const seq = ++this.#branchSeq;
    try {
      const branches = await listBranches(id);
      if (seq === this.#branchSeq) this.branches = branches;
    } catch (e) {
      toast(`載入分支清單失敗：${e}`, { variant: 'danger' });
    }
  }
}

export const reviewState = new ReviewState();
