import { toast } from '$lib/toast/state.svelte';
import {
  addRepo as addRepoCommand,
  fetchRemote,
  getFileContent,
  getFileDiff,
  getFileTree,
  listBranches,
  listRepos,
  pickRepoFolder,
} from './api';
import { findFileNode, firstFilePath, pruneToChanged } from './helpers';
import type { Branch, DiffHunk, DiffMode, DiffSpec, Repo, TreeNode } from './types';

export type ViewMode = 'unified' | 'split';

class ReviewState {
  repos = $state<Repo[]>([]);
  repoId = $state<string | null>(null);
  branches = $state<Branch[]>([]);
  branch = $state<string | null>(null);
  // What a Branch-mode diff compares against (CONTEXT.md: Base Branch). Starts at the
  // Repo's default branch, and the reviewer can pick any other branch instead.
  baseBranch = $state<string | null>(null);
  diffMode = $state<DiffMode>('unstaged');
  view = $state<ViewMode>('unified');

  tree = $state<TreeNode[]>([]);
  selectedFile = $state<string | null>(null);
  // Which view the selected file opens in: a changed file shows its diff, an unchanged
  // one shows File View — its full worktree content (CONTEXT.md: File View).
  selectedView = $state<'diff' | 'file' | null>(null);
  diffHunks = $state<DiffHunk[]>([]);
  fileLines = $state<string[]>([]);

  loadingRepos = $state(false);
  loadingBranches = $state(false);
  loadingTree = $state(false);
  loadingDiff = $state(false);
  loadingFile = $state(false);
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
    if (this.diffMode === 'branch' && this.baseBranch) spec.baseBranch = this.baseBranch;
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

  // Picks a folder, registers it as a Repo, and opens it for review. A folder that's
  // already registered just gets selected again.
  async addRepo() {
    let path: string | null;
    try {
      path = await pickRepoFolder();
    } catch (e) {
      toast(`開啟資料夾選擇器失敗：${e}`, { variant: 'danger' });
      return;
    }
    if (!path) return;
    try {
      const repo = await addRepoCommand(path);
      if (!this.repos.some((r) => r.id === repo.id)) this.repos = [...this.repos, repo];
      await this.selectRepo(repo.id);
    } catch (e) {
      toast(`新增 repo 失敗：${e}`, { variant: 'danger' });
    }
  }

  async selectRepo(id: string) {
    this.repoId = id;
    this.branch = null;
    this.branches = [];
    this.tree = [];
    // Each Repo brings its own default branch, so the previous Repo's base branch
    // doesn't carry over.
    this.baseBranch = this.repo?.defaultBranch ?? null;
    this.#clearSelection();
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

  async setBaseBranch(name: string) {
    this.baseBranch = name;
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
    this.#clearSelection();
    this.loadingTree = true;
    try {
      const tree = await getFileTree(spec);
      if (seq !== this.#treeSeq) return;
      this.tree = tree;
      // Opens on the first changed file — the reviewer came here for the diff, even
      // when the sidebar is also listing unchanged files.
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
    const spec = this.spec;
    if (!spec) return;
    const node = findFileNode(this.tree, path);
    // Only a changed file has a diff to show; anything else opens in File View.
    const view = node?.type === 'file' && node.changes ? 'diff' : 'file';
    this.selectedFile = path;
    this.selectedView = view;
    this.diffHunks = [];
    this.fileLines = [];
    if (view === 'diff') await this.#loadDiff(spec, path, seq);
    else await this.#loadFileContent(spec.repoId, path, seq);
  }

  async #loadDiff(spec: DiffSpec, path: string, seq: number) {
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

  async #loadFileContent(repoId: string, path: string, seq: number) {
    this.loadingFile = true;
    try {
      const content = await getFileContent(repoId, path);
      if (seq !== this.#diffSeq) return;
      this.fileLines = content.lines;
    } catch (e) {
      if (seq !== this.#diffSeq) return;
      toast(`載入檔案內容失敗：${e}`, { variant: 'danger' });
    } finally {
      if (seq === this.#diffSeq) this.loadingFile = false;
    }
  }

  #clearSelection() {
    this.selectedFile = null;
    this.selectedView = null;
    this.diffHunks = [];
    this.fileLines = [];
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
