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

  spec = $derived.by<DiffSpec | null>(() => {
    if (!this.repoId || !this.branch) return null;
    return { repoId: this.repoId, branch: this.branch, diffMode: this.diffMode };
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

    this.loadingBranches = true;
    try {
      this.branches = await listBranches(id);
      const current = this.branches.find((b) => b.isCurrent) ?? this.branches[0];
      if (current) await this.selectBranch(current.name);
    } catch (e) {
      toast(`載入分支清單失敗：${e}`, { variant: 'danger' });
    } finally {
      this.loadingBranches = false;
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
    this.selectedFile = null;
    this.diffHunks = [];
    this.loadingTree = true;
    try {
      this.tree = await getFileTree(spec);
      const firstPath = firstFilePath(pruneToChanged(this.tree)) ?? firstFilePath(this.tree);
      if (firstPath) await this.selectFile(firstPath);
    } catch (e) {
      toast(`載入變更檔案清單失敗：${e}`, { variant: 'danger' });
    } finally {
      this.loadingTree = false;
    }
  }

  async selectFile(path: string) {
    this.selectedFile = path;
    const spec = this.spec;
    if (!spec) return;
    this.loadingDiff = true;
    try {
      this.diffHunks = await getFileDiff(spec, path);
    } catch (e) {
      toast(`載入 diff 失敗：${e}`, { variant: 'danger' });
    } finally {
      this.loadingDiff = false;
    }
  }

  async fetchRemoteBranch() {
    if (!this.repoId || this.fetching) return;
    this.fetching = true;
    try {
      const result = await fetchRemote(this.repoId);
      if (!result.success) toast(result.message, { variant: 'danger' });
    } catch (e) {
      toast(`Fetch 失敗：${e}`, { variant: 'danger' });
    } finally {
      this.fetching = false;
    }
  }
}

export const reviewState = new ReviewState();
