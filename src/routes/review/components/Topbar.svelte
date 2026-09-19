<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import FetchButton from "$lib/components/FetchButton.svelte";
  import { branchMeta } from "../helpers";
  import type { Branch, DiffMode, Repo } from "../types";

  interface Props {
    repos: Repo[];
    repoId: string | null;
    branch: string | null;
    detachedHead: string | null;
    branches: Branch[];
    baseBranch: string | null;
    diffMode: DiffMode;
    fetching: boolean;
    lastFetched: string | null;
    onSelectRepo: (id: string) => void;
    onAddRepo: () => void;
    onRemoveRepo: (id: string) => void;
    onSetBaseBranch: (name: string) => void;
    onFetch: () => void;
  }
  let {
    repos,
    repoId,
    branch,
    detachedHead,
    branches,
    baseBranch,
    diffMode,
    fetching,
    lastFetched,
    onSelectRepo,
    onAddRepo,
    onRemoveRepo,
    onSetBaseBranch,
    onFetch,
  }: Props = $props();

  let repoOptions = $derived(repos.map((r) => ({ value: r.id, label: r.name, meta: r.path })));
  // Only the Base Branch is pickable now, so the branch under review is left out of
  // its options — a branch diffed against itself is always empty.
  let baseBranchOptions = $derived(
    branches.filter((b) => b.name !== branch).map((b) => ({ value: b.name, label: b.name, meta: branchMeta(b) })),
  );
</script>

<header class="topbar">
  <div class="brand">
    <Icon name="git-pull-request" size={18} color="var(--accent)" />
    <span class="wordmark">Ziff</span>
  </div>
  <Dropdown
    icon="folder"
    label="Repo"
    options={repoOptions}
    value={repoId ?? ""}
    onChange={onSelectRepo}
    onAddNew={onAddRepo}
    addNewLabel="Add repo…"
    optionAction={{ icon: "trash-2", title: "從 Ziff 移除（不會刪除資料夾）", onAction: onRemoveRepo }}
    placeholder="Select repo…"
    width={260}
  />
  <Icon name="chevron-right" size={12} color="var(--border-default)" />
  <!-- Not a picker: a comment's path:L12 is read against the worktree, so the branch
       under review is always the checked-out one (docs/decisions/0010). -->
  <div class="topbar-branch" title={detachedHead ? "HEAD 沒有指向任何分支" : "目前 checkout 的分支"}>
    <Icon name={detachedHead ? "git-commit-horizontal" : "git-branch"} size={13} color="var(--text-tertiary)" />
    <span class="topbar-branch-name">
      {detachedHead ? `detached @ ${detachedHead}` : (branch ?? "")}
    </span>
  </div>
  {#if diffMode === "branch"}
    <!-- Only Branch mode compares against a Base Branch (CONTEXT.md: Diff Mode). -->
    <span class="topbar-vs">vs</span>
    <Dropdown
      icon="git-merge"
      label="Base Branch"
      sublabel="Base branch"
      options={baseBranchOptions}
      value={baseBranch ?? ""}
      onChange={onSetBaseBranch}
      width={280}
    />
  {/if}
  <div class="topbar-spacer"></div>
  <FetchButton {fetching} lastFetched={lastFetched ?? undefined} {onFetch} />
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 48px;
    padding: 0 var(--space-3);
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-surface);
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding-right: var(--space-2);
  }

  .wordmark {
    font-size: var(--text-base);
    font-weight: 700;
    color: var(--text-primary);
  }

  .topbar-branch {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 8px;
    min-width: 0;
    max-width: 220px;
  }
  .topbar-branch-name {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .topbar-vs {
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    flex-shrink: 0;
  }
  .topbar-spacer {
    flex: 1;
    min-width: 8px;
  }

</style>
