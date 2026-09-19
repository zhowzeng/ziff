<script lang="ts">
  import DiffHunk from "./DiffHunk.svelte";
  import DiffLine from "./DiffLine.svelte";
  import DiffLineSplit from "./DiffLineSplit.svelte";
  import FileHeader from "./FileHeader.svelte";
  import FileLine from "./FileLine.svelte";
  import CommentThread from "./CommentThread.svelte";
  import Avatar from "$lib/components/Avatar.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import { toast } from "$lib/toast/state.svelte";
  import { commentQueue } from "../comment-queue.svelte";
  import { selection } from "../selection.svelte";
  import {
    fileLineRange,
    flattenHunks,
    formatForAgent,
    formatTime,
    lineRange,
    pairHunkLines,
    type FlatLine,
    type IndexedLine,
  } from "../helpers";
  import type { ViewMode } from "../state.svelte";
  import type { DiffHunk as DiffHunkData, DiffLine as DiffLineData } from "../types";

  const VIEW_MODES = [
    { value: "unified", label: "Unified" },
    { value: "split", label: "Split" },
  ];

  interface Props {
    repoId: string | null;
    selectedFile: string | null;
    selectedRenamedFrom: string | null;
    /** Which view the selected file opens in — an unchanged file has no diff, so it
     *  opens in File View instead (CONTEXT.md). */
    selectedView: "diff" | "file" | null;
    view: ViewMode;
    onViewChange: (view: ViewMode) => void;
    diffHunks: DiffHunkData[];
    diffBinary: boolean;
    loadingDiff: boolean;
    fileLines: string[];
    fileBinary: boolean;
    loadingFile: boolean;
    /** The repo/branch/tree load, which has to finish before any of this means anything. */
    loading: boolean;
    noRepos: boolean;
    hasChanges: boolean;
  }
  let {
    repoId,
    selectedFile,
    selectedRenamedFrom,
    selectedView,
    view,
    onViewChange,
    diffHunks,
    diffBinary,
    loadingDiff,
    fileLines,
    fileBinary,
    loadingFile,
    loading,
    noRepos,
    hasChanges,
  }: Props = $props();

  let isFileView = $derived(selectedView === "file");

  $effect(() => {
    function onUp() {
      selection.commitDrag();
    }
    window.addEventListener("mouseup", onUp);
    return () => window.removeEventListener("mouseup", onUp);
  });

  function gutterDown(i: number) {
    selection.startDrag(i);
  }
  function gutterEnter(i: number) {
    const start = selection.dragStart;
    if (start === null) return;
    // A selection stays inside one hunk: line numbers aren't contiguous across hunk
    // boundaries, so a range spanning two hunks names lines that aren't in the diff.
    // File View has no hunks — it's one continuous file — so nothing to stay inside of.
    if (!isFileView && flatLines[i]?.hunk !== flatLines[start]?.hunk) return;
    selection.extendDrag(i);
  }

  let flatLines = $derived(flattenHunks(diffHunks));
  let hunkGroups = $derived.by(() => {
    const groups = diffHunks.map((h) => ({ header: h.header, lines: [] as FlatLine[] }));
    for (const f of flatLines) groups[f.hunk].lines.push(f);
    return groups;
  });
  // File View's lines are indexed straight off the file, so its anchors come from the
  // line indexes themselves rather than from the diff's two numbering spaces.
  let threadRange = $derived.by(() => {
    const range = selection.range;
    if (!range) return null;
    if (isFileView) return fileLineRange(range.lo, range.hi);
    return lineRange(flatLines, range.lo, range.hi);
  });

  // The thread renders straight out of the Comment Queue rather than keeping its own
  // copy, so an edit here reaches the text that gets handed to the CLI agent, and
  // reopening a commented range edits that comment instead of starting a second one.
  let threadItem = $derived(
    threadRange && selectedFile ? commentQueue.find(repoId, selectedFile, threadRange) : null,
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

  // A queue belongs to one Repo (docs/decisions/0009), so everything read out of it
  // here is scoped to the Repo being reviewed.
  let commentedKeys = $derived(commentQueue.lineKeys(repoId, selectedFile ?? ""));

  function isCommented(line: DiffLineData) {
    return line.newNo !== null && commentedKeys.has(`new:${line.newNo}`);
  }

  function addToQueue() {
    if (!selection.draft.trim() || !threadRange || !selectedFile || !repoId) return;
    commentQueue.add({
      repoId,
      file: selectedFile,
      ...threadRange,
      text: selection.draft.trim(),
    });
    selection.draft = "";
  }

  // Copy Now is used-once (docs/decisions/0001): it never touches the Comment Queue, so
  // the thread closes instead of reopening onto a saved comment there is none of.
  async function copyNow() {
    if (!selection.draft.trim() || !threadRange || !selectedFile) return;
    const text = formatForAgent({ file: selectedFile, ...threadRange, text: selection.draft.trim() });
    try {
      await navigator.clipboard.writeText(text);
    } catch (e) {
      toast(`複製到剪貼簿失敗：${e}`, { variant: "danger" });
      return;
    }
    selection.close();
    toast("已複製這則評論", { variant: "success" });
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
        file={selectedFile}
        lineStart={threadRange?.lineStart}
        lineEnd={threadRange?.lineEnd}
        comments={threadComments}
        bind:replyValue={selection.draft}
        onAddToQueue={addToQueue}
        onCopyNow={copyNow}
        onClose={() => selection.close()}
      />
    </div>
  </div>
{/snippet}

{#if loading}
  <main class="diff-panel diff-panel-empty">
    <EmptyState size="md" icon="loader" title="載入中…" />
  </main>
{:else if !repoId}
  <main class="diff-panel diff-panel-empty">
    <EmptyState
      size="md"
      icon="folder-git-2"
      title={noRepos ? "加入一個 repo 開始" : "選擇一個 repo 開始"}
      hint={noRepos
        ? "從左上角 Repo 選單的「Add repo…」選擇本機 git repo 資料夾。"
        : "從左上角選擇 repo 與 branch，即可檢視變更並開始留言。"}
    />
  </main>
{:else if !selectedFile && !hasChanges}
  <main class="diff-panel diff-panel-empty">
    <EmptyState size="md" icon="git-compare" title="這個分支目前沒有變更" hint="切換到有 commit 差異的分支，或建立新的變更後再回來查看。" />
  </main>
{:else if !selectedFile}
  <main class="diff-panel diff-panel-empty">
    <EmptyState size="md" icon="file-code" title="選擇一個檔案查看 diff" hint="從左側的檔案清單選擇一個變更的檔案。" />
  </main>
{:else}
  <main class="diff-panel">
    <div class="diff-panel-header">
      <div class="file-header-wrap"><FileHeader path={selectedFile} renamedFrom={selectedRenamedFrom} /></div>
      {#if !isFileView}
        <div class="view-toggle-wrap"><Segmented value={view} onChange={(v) => onViewChange(v as ViewMode)} options={VIEW_MODES} /></div>
      {/if}
    </div>

    {#if isFileView}
      {#if loadingFile}
        <div class="diff-body-empty"><EmptyState size="md" icon="loader" title="載入檔案…" /></div>
      {:else if fileBinary}
        <div class="diff-body-empty">
          <EmptyState size="md" icon="binary" title="二進位檔" hint="這個檔案不是文字檔，無法逐行顯示，也無法留言。" />
        </div>
      {:else if fileLines.length === 0}
        <div class="diff-body-empty">
          <EmptyState size="md" icon="file" title="這個檔案是空的" hint="檔案沒有任何內容可以顯示。" />
        </div>
      {:else}
        {#each fileLines as content, i (i)}
          <FileLine
            lineNo={i + 1}
            {content}
            index={i}
            selected={selection.includes(i)}
            commented={commentedKeys.has(`file:${i + 1}`)}
            onGutterDown={gutterDown}
            onGutterEnter={gutterEnter}
          />
          {#if selection.range && i === selection.range.hi}
            {@render commentBlock()}
          {/if}
        {/each}
      {/if}
    {:else if loadingDiff}
      <div class="diff-body-empty"><EmptyState size="md" icon="loader" title="載入 diff…" /></div>
    {:else if diffBinary}
      <div class="diff-body-empty">
        <EmptyState size="md" icon="binary" title="二進位檔" hint="這個檔案有變更，但不是文字檔，無法逐行顯示 diff，也無法留言。" />
      </div>
    {:else if diffHunks.length === 0}
      <div class="diff-body-empty">
        <EmptyState size="md" icon="file-check" title="這個檔案沒有變更" hint="選擇左側標示變更行數的檔案，才會顯示 diff。" />
      </div>
    {:else if view === "unified"}
      {#each rows as row, i (i)}
        {#if row.kind === "header"}
          <DiffHunk label={row.label} />
        {:else}
          <DiffLine
            kind={row.line.kind}
            oldNo={row.line.oldNo}
            newNo={row.line.newNo}
            index={row.idx}
            selected={selection.includes(row.idx)}
            commented={isCommented(row.line)}
            onGutterDown={gutterDown}
            onGutterEnter={gutterEnter}
          >
            {row.line.content}
          </DiffLine>
          {#if selection.range && row.idx === selection.range.hi}
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
              if (idx !== null) selection.openAt(idx);
            }}
          />
          {#if selection.range && splitRowHasIdx(row, selection.range.hi)}
            {@render commentBlock()}
          {/if}
        {/if}
      {/each}
    {/if}
  </main>
{/if}

<style>
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
