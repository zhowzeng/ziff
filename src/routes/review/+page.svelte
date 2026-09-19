<script lang="ts">
  import { onMount } from "svelte";
  import FileTree from "./components/FileTree.svelte";
  import DiffHunk from "./components/DiffHunk.svelte";
  import DiffLine from "./components/DiffLine.svelte";
  import DiffLineSplit from "./components/DiffLineSplit.svelte";
  import CommentThread from "./components/CommentThread.svelte";
  import Avatar from "$lib/components/Avatar.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import Input from "$lib/components/Input.svelte";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import FetchButton from "$lib/components/FetchButton.svelte";
  import FileHeader from "./components/FileHeader.svelte";
  import QueueDrawer from "./components/QueueDrawer.svelte";
  import QueueFab from "./components/QueueFab.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import SettingsModal, { DEFAULT_SETTINGS } from "$lib/components/SettingsModal.svelte";
  import { toast } from "$lib/toast/state.svelte";
  import { reviewState } from "./state.svelte";
  import { commentQueue } from "./comment-queue.svelte";
  import {
    allDirPaths,
    branchMeta,
    flattenHunks,
    formatTime,
    lineRange,
    pairHunkLines,
    pruneByName,
    pruneToChanged,
    type FlatLine,
    type IndexedLine,
  } from "./helpers";
  import type { DiffLine as DiffLineData, DiffMode } from "./types";

  const DIFF_MODES = [
    { value: "unstaged", label: "Unstaged" },
    { value: "branch", label: "Branch" },
    { value: "staged", label: "Staged" },
  ];
  const VIEW_MODES = [
    { value: "unified", label: "Unified" },
    { value: "split", label: "Split" },
  ];

  onMount(() => {
    reviewState.loadRepos();
  });

  let showAllFiles = $state(false);
  let fileFilter = $state("");
  let settingsOpen = $state(false);
  let settings = $state({ ...DEFAULT_SETTINGS });

  let repoOptions = $derived(reviewState.repos.map((r) => ({ value: r.id, label: r.name, meta: r.path })));
  let branchOptions = $derived(reviewState.branches.map((b) => ({ value: b.name, label: b.name, meta: branchMeta(b) })));

  let changedTree = $derived(pruneToChanged(reviewState.tree));
  let baseTree = $derived(showAllFiles ? reviewState.tree : changedTree);
  let shownTree = $derived(fileFilter.trim() ? pruneByName(baseTree, fileFilter.trim().toLowerCase()) : baseTree);
  // "No diff" is about what the sidebar would actually list: the changed files, or
  // the whole tree when the reviewer asked to see every file.
  let sidebarEmptyState = $derived.by(() => {
    if (baseTree.length === 0) return "no-diff";
    if (fileFilter.trim() && shownTree.length === 0) return "no-match";
    return null;
  });
  let loadingFiles = $derived(reviewState.loadingRepos || reviewState.loadingBranches || reviewState.loadingTree);

  let openThread = $state<{ lo: number; hi: number } | null>(null);
  let replyValue = $state("");
  let dragging = $state<{ start: number; end: number } | null>(null);

  $effect(() => {
    function onUp() {
      if (dragging) {
        const lo = Math.min(dragging.start, dragging.end);
        const hi = Math.max(dragging.start, dragging.end);
        dragging = null;
        openThread = { lo, hi };
        replyValue = "";
      }
    }
    window.addEventListener("mouseup", onUp);
    return () => window.removeEventListener("mouseup", onUp);
  });

  function gutterDown(i: number) {
    dragging = { start: i, end: i };
  }
  function gutterEnter(i: number) {
    if (!dragging) return;
    // A selection stays inside one hunk: line numbers aren't contiguous across hunk
    // boundaries, so a range spanning two hunks names lines that aren't in the diff.
    if (flatLines[i]?.hunk !== flatLines[dragging.start]?.hunk) return;
    dragging = { ...dragging, end: i };
  }
  function isSelected(i: number) {
    if (dragging) {
      const lo = Math.min(dragging.start, dragging.end);
      const hi = Math.max(dragging.start, dragging.end);
      return i >= lo && i <= hi;
    }
    if (openThread) return i >= openThread.lo && i <= openThread.hi;
    return false;
  }
  function openSingleThread(idx: number) {
    openThread = { lo: idx, hi: idx };
    replyValue = "";
  }
  function closeThread() {
    openThread = null;
    replyValue = "";
    dragging = null;
  }

  let flatLines = $derived(flattenHunks(reviewState.diffHunks));
  let hunkGroups = $derived.by(() => {
    const groups = reviewState.diffHunks.map((h) => ({ header: h.header, lines: [] as FlatLine[] }));
    for (const f of flatLines) groups[f.hunk].lines.push(f);
    return groups;
  });
  let threadRange = $derived(openThread ? lineRange(flatLines, openThread.lo, openThread.hi) : null);

  // The thread renders straight out of the Comment Queue rather than keeping its own
  // copy, so an edit here reaches the text that gets handed to the CLI agent, and
  // reopening a commented range edits that comment instead of starting a second one.
  let threadItem = $derived(
    threadRange && reviewState.selectedFile ? commentQueue.find(reviewState.selectedFile, threadRange) : null,
  );
  let threadComments = $derived.by(() => {
    const item = threadItem;
    if (!item) return [];
    return [
      {
        time: formatTime(item.createdAt),
        text: item.text,
        editable: true,
        onEdit: (text: string) => commentQueue.update(item.id, text),
      },
    ];
  });

  let commentedKeys = $derived(commentQueue.lineKeys(reviewState.selectedFile ?? ""));

  function isCommented(line: DiffLineData) {
    return (
      (line.newNo !== null && commentedKeys.has(`new:${line.newNo}`)) ||
      (line.oldNo !== null && commentedKeys.has(`old:${line.oldNo}`))
    );
  }

  function submitComment() {
    if (!replyValue.trim() || !threadRange || !reviewState.selectedFile) return;
    commentQueue.add({ file: reviewState.selectedFile, ...threadRange, text: replyValue.trim() });
    replyValue = "";
  }

  function selectRepo(id: string) {
    closeThread();
    reviewState.selectRepo(id);
  }
  function selectBranch(name: string) {
    closeThread();
    reviewState.selectBranch(name);
  }
  function setDiffMode(mode: string) {
    closeThread();
    reviewState.setDiffMode(mode as DiffMode);
  }
  function selectFile(path: string) {
    closeThread();
    reviewState.selectFile(path);
  }

  type Row = { kind: "header"; label: string } | { kind: "line"; idx: number; line: DiffLineData };
  let rows = $derived.by<Row[]>(() => {
    const out: Row[] = [];
    for (const g of hunkGroups) {
      out.push({ kind: "header", label: g.header });
      for (const f of g.lines) out.push({ kind: "line", idx: f.idx, line: f.line });
    }
    return out;
  });

  type SplitPair = ReturnType<typeof pairHunkLines>[number];
  type SplitRow = { kind: "header"; label: string } | ({ kind: "row" } & SplitPair);
  let splitRows = $derived.by<SplitRow[]>(() => {
    const out: SplitRow[] = [];
    for (const g of hunkGroups) {
      out.push({ kind: "header", label: g.header });
      const numbered: IndexedLine[] = g.lines.map((f) => ({ ...f.line, idx: f.idx }));
      for (const pair of pairHunkLines(numbered)) {
        out.push({ kind: "row", ...pair });
      }
    }
    return out;
  });

  function splitRowThreadEndIdx(row: SplitPair) {
    if (row.right) return row.right.idx;
    if (row.left) return row.left.idx;
    return null;
  }

  // A del line paired with an add sits on the left of its row, so matching only the
  // right side would drop the thread when the reviewer switches to Split view.
  function splitRowHasIdx(row: SplitPair, idx: number) {
    return row.left?.idx === idx || row.right?.idx === idx;
  }
</script>

{#snippet commentBlock()}
  <div class="thread-anchor">
    <div class="thread-avatar">
      <Avatar name="Yu-Chen" size={24} />
    </div>
    <div class="thread-body">
      <CommentThread
        file={reviewState.selectedFile}
        lineStart={threadRange?.lineStart}
        lineEnd={threadRange?.lineEnd}
        comments={threadComments}
        bind:replyValue
        onComment={submitComment}
        onClose={closeThread}
      />
    </div>
  </div>
{/snippet}

<div class="app">
  <header class="topbar">
    <div class="brand">
      <Icon name="git-pull-request" size={18} color="var(--accent)" />
      <span class="wordmark">Ziff</span>
    </div>
    <Dropdown
      icon="folder"
      label="Repo"
      options={repoOptions}
      value={reviewState.repoId ?? ""}
      onChange={selectRepo}
      onAddNew={() => toast("新增 repo 的資料夾選擇器尚未接上")}
      addNewLabel="Add repo…"
      width={260}
    />
    <Icon name="chevron-right" size={12} color="var(--border-default)" />
    <Dropdown icon="git-branch" label="Branch" options={branchOptions} value={reviewState.branch ?? ""} onChange={selectBranch} width={280} />
    <div class="topbar-spacer"></div>
    <FetchButton fetching={reviewState.fetching} lastFetched={reviewState.lastFetched ?? undefined} onFetch={() => reviewState.fetchRemoteBranch()} />
  </header>

  <div class="body">
    <aside class="sidebar">
      <div class="sidebar-toolbar">
        <Segmented value={reviewState.diffMode} onChange={setDiffMode} options={DIFF_MODES} />
      </div>
      <div class="sidebar-filter">
        <div class="filter-input-wrap">
          <Icon name="search" size={13} color="var(--text-tertiary)" class="filter-icon" />
          <Input placeholder="Filter files…" size="sm" bind:value={fileFilter} disabled={!reviewState.repoId} style="padding-left:26px" />
        </div>
        <label class="show-all-label">
          <input type="checkbox" bind:checked={showAllFiles} disabled={!reviewState.repoId} />
          顯示所有檔案
        </label>
      </div>
      <div class="sidebar-tree">
        {#if loadingFiles}
          <EmptyState size="sm" icon="loader" title="載入中…" />
        {:else if !reviewState.repoId}
          <EmptyState size="sm" icon="folder-git-2" title="尚未選擇 repo" hint="從上方選擇 repo 與 branch 後，這裡會顯示變更的檔案。" />
        {:else if sidebarEmptyState === "no-diff"}
          <EmptyState size="sm" icon="git-compare" title="此分支沒有變更" hint="切換到有變更的分支，或勾選「顯示所有檔案」瀏覽整個專案。" />
        {:else if sidebarEmptyState === "no-match"}
          <EmptyState size="sm" icon="search-x" title="找不到符合的檔案" hint={`沒有檔案名稱包含「${fileFilter.trim()}」`} />
        {:else}
          {#key reviewState.tree}
            <FileTree tree={shownTree} selected={reviewState.selectedFile ?? undefined} onSelect={selectFile} defaultExpanded={allDirPaths(changedTree)} />
          {/key}
        {/if}
      </div>
      <button class="settings-entry" onclick={() => (settingsOpen = true)}>
        <Icon name="settings" size={14} color="var(--text-tertiary)" />
        Settings
      </button>
    </aside>

    {#if loadingFiles}
      <main class="diff-panel diff-panel-empty">
        <EmptyState size="md" icon="loader" title="載入中…" />
      </main>
    {:else if !reviewState.repoId}
      <main class="diff-panel diff-panel-empty">
        <EmptyState size="md" icon="folder-git-2" title="選擇一個 repo 開始" hint="從左上角選擇 repo 與 branch，即可檢視變更並開始留言。" />
      </main>
    {:else if !reviewState.selectedFile && changedTree.length === 0}
      <main class="diff-panel diff-panel-empty">
        <EmptyState size="md" icon="git-compare" title="這個分支目前沒有變更" hint="切換到有 commit 差異的分支，或建立新的變更後再回來查看。" />
      </main>
    {:else if !reviewState.selectedFile}
      <main class="diff-panel diff-panel-empty">
        <EmptyState size="md" icon="file-code" title="選擇一個檔案查看 diff" hint="從左側的檔案清單選擇一個變更的檔案。" />
      </main>
    {:else}
      <main class="diff-panel">
        <div class="diff-panel-header">
          <div class="file-header-wrap"><FileHeader path={reviewState.selectedFile} /></div>
          <div class="view-toggle-wrap"><Segmented value={reviewState.view} onChange={(v) => (reviewState.view = v as typeof reviewState.view)} options={VIEW_MODES} /></div>
        </div>

        {#if reviewState.loadingDiff}
          <div class="diff-body-empty"><EmptyState size="md" icon="loader" title="載入 diff…" /></div>
        {:else if reviewState.diffHunks.length === 0}
          <div class="diff-body-empty">
            <EmptyState size="md" icon="file-check" title="這個檔案沒有變更" hint="選擇左側標示變更行數的檔案，才會顯示 diff。" />
          </div>
        {:else if reviewState.view === "unified"}
          {#each rows as row, i (i)}
            {#if row.kind === "header"}
              <DiffHunk label={row.label} />
            {:else}
              <DiffLine
                kind={row.line.kind}
                oldNo={row.line.oldNo}
                newNo={row.line.newNo}
                index={row.idx}
                selected={isSelected(row.idx)}
                commented={isCommented(row.line)}
                onGutterDown={gutterDown}
                onGutterEnter={gutterEnter}
              >
                {row.line.content}
              </DiffLine>
              {#if openThread && row.idx === openThread.hi}
                {@render commentBlock()}
              {/if}
            {/if}
          {/each}
        {:else}
          {#each splitRows as row, i (i)}
            {#if row.kind === "header"}
              <DiffHunk label={row.label} />
            {:else}
              <DiffLineSplit
                left={row.left}
                right={row.right}
                leftCommented={commentedKeys.has(`old:${row.left?.no}`)}
                rightCommented={commentedKeys.has(`new:${row.right?.no}`)}
                onAddComment={() => {
                  const idx = splitRowThreadEndIdx(row);
                  if (idx !== null) openSingleThread(idx);
                }}
              />
              {#if openThread && splitRowHasIdx(row, openThread.hi)}
                {@render commentBlock()}
              {/if}
            {/if}
          {/each}
        {/if}
      </main>
    {/if}

    {#if commentQueue.open}
      <QueueDrawer items={commentQueue.items} onRemove={(id) => commentQueue.remove(id)} onClose={() => (commentQueue.open = false)} />
    {/if}
  </div>

  <QueueFab count={commentQueue.items.length} open={commentQueue.open} onclick={() => (commentQueue.open = !commentQueue.open)} />
  <SettingsModal open={settingsOpen} onClose={() => (settingsOpen = false)} {settings} onChange={(s) => (settings = s)} />
</div>

<style>
  :global(html, body) {
    height: 100%;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    font-family: var(--font-sans);
  }

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

  .topbar-spacer {
    flex: 1;
    min-width: 8px;
  }

  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

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

  .diff-panel {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    background: var(--bg-canvas);
  }

  .diff-panel-empty {
    display: flex;
  }

  .diff-body-empty {
    display: flex;
    min-height: 240px;
  }

  .diff-panel-header {
    display: flex;
    align-items: stretch;
    background: var(--bg-subtle);
    border-bottom: 1px solid var(--border-default);
    position: sticky;
    top: 0;
  }

  .file-header-wrap {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .view-toggle-wrap {
    padding: 0 10px;
    display: flex;
    align-items: center;
  }

  .thread-anchor {
    display: flex;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-subtle);
    border-bottom: 1px solid var(--border-muted);
  }

  .thread-avatar {
    flex-shrink: 0;
    padding-top: 2px;
  }

  .thread-body {
    flex: 1;
    min-width: 0;
  }
</style>
