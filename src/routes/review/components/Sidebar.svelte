<script lang="ts">
  import FileTree from "./FileTree.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import Input from "$lib/components/Input.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import { allDirPaths, pruneByName } from "../helpers";
  import type { DiffMode, TreeNode } from "../types";

  const DIFF_MODES = [
    { value: "unstaged", label: "Unstaged" },
    { value: "branch", label: "Branch" },
    { value: "staged", label: "Staged" },
  ];

  interface Props {
    tree: TreeNode[];
    /** The changed-file subtree, so the "no diff" state and the default expansion agree
     *  with what the diff panel is showing. */
    changedTree: TreeNode[];
    diffMode: DiffMode;
    repoId: string | null;
    detachedHead: string | null;
    selectedFile: string | null;
    loading: boolean;
    /** Nothing has been added on this machine yet, so the reviewer's next step is the
     *  folder picker rather than the Repo dropdown. */
    noRepos: boolean;
    onSetDiffMode: (mode: string) => void;
    onSelectFile: (path: string) => void;
    onOpenSettings: () => void;
  }
  let {
    tree,
    changedTree,
    diffMode,
    repoId,
    detachedHead,
    selectedFile,
    loading,
    noRepos,
    onSetDiffMode,
    onSelectFile,
    onOpenSettings,
  }: Props = $props();

  // Both only ever narrow what this sidebar lists, so they live here rather than in the
  // page.
  let showAllFiles = $state(false);
  let fileFilter = $state("");

  let baseTree = $derived(showAllFiles ? tree : changedTree);
  let shownTree = $derived(fileFilter.trim() ? pruneByName(baseTree, fileFilter.trim().toLowerCase()) : baseTree);
  // "No diff" is about what the sidebar would actually list: the changed files, or
  // the whole tree when the reviewer asked to see every file.
  let emptyState = $derived.by(() => {
    if (baseTree.length === 0) return "no-diff";
    if (fileFilter.trim() && shownTree.length === 0) return "no-match";
    return null;
  });
</script>

<aside class="sidebar">
  <div class="sidebar-toolbar">
    <Segmented value={diffMode} onChange={onSetDiffMode} options={DIFF_MODES} />
  </div>
  <div class="sidebar-filter">
    <div class="filter-input-wrap">
      <Icon name="search" size={13} color="var(--text-tertiary)" class="filter-icon" />
      <Input placeholder="Filter files…" size="sm" bind:value={fileFilter} disabled={!repoId} style="padding-left:26px" />
    </div>
    <label class="show-all-label">
      <input type="checkbox" bind:checked={showAllFiles} disabled={!repoId} />
      顯示所有檔案
    </label>
  </div>
  <div class="sidebar-tree">
    {#if loading}
      <EmptyState size="sm" icon="loader" title="載入中…" />
    {:else if !repoId}
      <EmptyState
        size="sm"
        icon="folder-git-2"
        title={noRepos ? "尚未加入 repo" : "尚未選擇 repo"}
        hint={noRepos
          ? "從上方 Repo 選單的「Add repo…」加入本機 git repo。"
          : "從上方選擇 repo 與 branch 後，這裡會顯示變更的檔案。"}
      />
    {:else if detachedHead}
      <EmptyState
        size="sm"
        icon="git-commit-horizontal"
        title="HEAD 沒有指向分支"
        hint={`目前停在 ${detachedHead}。Ziff review 的是已 checkout 的分支，先 checkout 一個分支再回來。`}
      />
    {:else if emptyState === "no-diff"}
      <EmptyState size="sm" icon="git-compare" title="此分支沒有變更" hint="切換到有變更的分支，或勾選「顯示所有檔案」瀏覽整個專案。" />
    {:else if emptyState === "no-match"}
      <EmptyState size="sm" icon="search-x" title="找不到符合的檔案" hint={`沒有檔案名稱包含「${fileFilter.trim()}」`} />
    {:else}
      {#key tree}
        <FileTree tree={shownTree} selected={selectedFile ?? undefined} onSelect={onSelectFile} defaultExpanded={allDirPaths(changedTree)} />
      {/key}
    {/if}
  </div>
  <button class="settings-entry" onclick={onOpenSettings}>
    <Icon name="settings" size={14} color="var(--text-tertiary)" />
    Settings
  </button>
</aside>

<style>
  .sidebar {
    width: 240px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-default);
    background: var(--bg-surface);
    min-height: 0;
  }

  .sidebar-toolbar {
    padding: var(--space-2);
    border-bottom: 1px solid var(--border-muted);
    flex-shrink: 0;
  }

  .sidebar-filter {
    padding: var(--space-2);
    border-bottom: 1px solid var(--border-muted);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .filter-input-wrap {
    position: relative;
  }

  .filter-input-wrap :global(.filter-icon) {
    position: absolute;
    left: 8px;
    top: 8px;
    pointer-events: none;
  }

  .show-all-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .show-all-label input {
    margin: 0;
    accent-color: var(--accent-emphasis);
  }

  .sidebar-tree {
    flex: 1;
    min-height: 0;
    padding: var(--space-2);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .settings-entry {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 10px;
    border: none;
    border-top: 1px solid var(--border-default);
    background: var(--bg-subtle);
    cursor: pointer;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .settings-entry:hover {
    background: var(--bg-inset);
  }

</style>
