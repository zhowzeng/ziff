<script lang="ts">
  import FileTree from "$lib/components/FileTree.svelte";
  import DiffHunk from "$lib/components/DiffHunk.svelte";
  import DiffLine from "$lib/components/DiffLine.svelte";
  import DiffLineSplit from "$lib/components/DiffLineSplit.svelte";
  import CommentThread from "$lib/components/CommentThread.svelte";
  import Avatar from "$lib/components/Avatar.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import Dropdown from "$lib/components/Dropdown.svelte";
  import Segmented from "$lib/components/Segmented.svelte";
  import FetchButton from "$lib/components/FetchButton.svelte";
  import FileHeader from "$lib/components/FileHeader.svelte";
  import ContextDrawer from "$lib/components/ContextDrawer.svelte";
  import ContextFab from "$lib/components/ContextFab.svelte";

  const PROJECTS = [
    { value: "goose", label: "goose", meta: "~/dev/goose" },
    { value: "goose-mcp-extensions", label: "goose-mcp-extensions", meta: "~/dev/goose-mcp-extensions" },
    { value: "block-design-system", label: "block-design-system", meta: "~/dev/block-design-system" },
  ];
  const BRANCHES = [
    { value: "feature/xlsx-api-upgrade", label: "feature/xlsx-api-upgrade", meta: "目前分支 · 領先 main 3 個 commit" },
    { value: "main", label: "main", meta: "落後 2 個 commit" },
    { value: "develop", label: "develop" },
    { value: "feature/mcp-context-drawer", label: "feature/mcp-context-drawer" },
  ];
  const DIFF_MODES = [
    { value: "unstaged", label: "Unstaged" },
    { value: "branch", label: "Branch" },
    { value: "staged", label: "Staged" },
  ];
  const VIEW_MODES = [
    { value: "unified", label: "Unified" },
    { value: "split", label: "Split" },
  ];

  const tree = [
    {
      name: "src",
      path: "src",
      type: "dir",
      children: [
        { name: "xlsx_tool.rs", path: "src/xlsx_tool.rs", type: "file", changes: { add: 12, del: 4 } },
        { name: "main.rs", path: "src/main.rs", type: "file" },
      ],
    },
    { name: "Cargo.toml", path: "Cargo.toml", type: "file", changes: { add: 2, del: 1 } },
  ];

  let project = $state("goose");
  let branch = $state("feature/xlsx-api-upgrade");
  let diffMode = $state("unstaged");
  let view = $state<"unified" | "split">("unified");
  let selected = $state("src/xlsx_tool.rs");
  let replyValue = $state("");
  let contextOpen = $state(false);
  let savedContext = $state<{ id: string; file: string; lineStart: number; lineEnd?: number; text: string }[]>([]);

  type DiffLineData = {
    kind: "context" | "add" | "del";
    oldNo: number | null;
    newNo: number | null;
    text: string;
    // commentHere: this line already carries the seeded demo thread.
    // commentable: in split view (no gutter-drag there) it's the only affordance to reopen that thread.
    commentHere?: boolean;
    commentable?: boolean;
  };

  const diffLines: DiffLineData[] = [
    { kind: "context", oldNo: 10, newNo: 10, text: "use calamine::{open_workbook, Reader, Xlsx};" },
    { kind: "del", oldNo: 11, newNo: null, text: 'calamine = "0.22"' },
    { kind: "add", oldNo: null, newNo: 11, text: 'calamine = "0.24"' },
    { kind: "context", oldNo: 12, newNo: 12, text: "" },
    { kind: "del", oldNo: 13, newNo: null, text: "let mut wb: Xlsx<_> = open_workbook(path)?;" },
    { kind: "add", oldNo: null, newNo: 13, text: "let mut wb: Xlsx<_> = open_workbook_auto(path)?;", commentHere: true, commentable: true },
    { kind: "context", oldNo: 14, newNo: 14, text: 'let sheet = wb.worksheet_range("Sheet1")?;' },
  ];

  const commentHereIndex = diffLines.findIndex((l) => l.commentHere);

  type Comment = { time: string; text: string; editable: boolean; onEdit: (text: string) => void };

  let openLineThread = $state(true);
  let fixedComment = $state<Comment>({
    time: "2m ago",
    text: "跳大版有改 api",
    editable: true,
    onEdit: (text: string) => {
      fixedComment.text = text;
    },
  });

  let dragging = $state<{ start: number; end: number } | null>(null);
  let range = $state<{ lo: number; hi: number } | null>(null);
  let rangeComments = $state<Comment[]>([]);

  $effect(() => {
    function onUp() {
      if (dragging) {
        const lo = Math.min(dragging.start, dragging.end);
        const hi = Math.max(dragging.start, dragging.end);
        range = { lo, hi };
        dragging = null;
      }
    }
    window.addEventListener("mouseup", onUp);
    return () => window.removeEventListener("mouseup", onUp);
  });

  function gutterDown(i: number) {
    dragging = { start: i, end: i };
  }
  function gutterEnter(i: number) {
    if (dragging) dragging = { ...dragging, end: i };
  }
  function isSelected(i: number) {
    if (dragging) {
      const lo = Math.min(dragging.start, dragging.end);
      const hi = Math.max(dragging.start, dragging.end);
      return i >= lo && i <= hi;
    }
    if (range) return i >= range.lo && i <= range.hi;
    return false;
  }
  function rangeLineNo(i: number) {
    const l = diffLines[i];
    return l.newNo ?? l.oldNo ?? 0;
  }

  function addToContext(item: { file: string; lineStart: number; lineEnd?: number; text: string }) {
    savedContext = [...savedContext, { id: crypto.randomUUID(), ...item }];
    contextOpen = true;
  }
  function removeFromContext(id: string) {
    savedContext = savedContext.filter((i) => i.id !== id);
  }

  function submitFixedComment() {
    if (!replyValue.trim()) return;
    const text = replyValue.trim();
    fixedComment = {
      time: "now",
      text,
      editable: true,
      onEdit: (t: string) => {
        fixedComment.text = t;
      },
    };
    addToContext({ file: selected, lineStart: rangeLineNo(commentHereIndex), text });
    replyValue = "";
  }

  function submitRangeComment() {
    if (!replyValue.trim() || !range) return;
    const text = replyValue.trim();
    const lineStart = rangeLineNo(range.lo);
    const lineEnd = rangeLineNo(range.hi);
    rangeComments = [
      {
        time: "now",
        text,
        editable: true,
        onEdit: (t: string) => {
          rangeComments[0].text = t;
        },
      },
    ];
    addToContext({ file: selected, lineStart, lineEnd, text });
    replyValue = "";
  }

  function selectFile(path: string) {
    selected = path;
    openLineThread = false;
    range = null;
  }

  type SplitHalf = { kind: "context" | "add" | "del"; no: number | null; text: string; commentable?: boolean; _srcIndex?: number } | null;

  function pairLines(lines: DiffLineData[]): { left: SplitHalf; right: SplitHalf }[] {
    const result: { left: SplitHalf; right: SplitHalf }[] = [];
    let i = 0;
    while (i < lines.length) {
      const l = lines[i];
      if (l.kind === "context") {
        result.push({ left: { kind: "context", no: l.oldNo, text: l.text }, right: { kind: "context", no: l.newNo, text: l.text } });
        i++;
        continue;
      }
      const dels: DiffLineData[] = [];
      while (i < lines.length && lines[i].kind === "del") {
        dels.push(lines[i]);
        i++;
      }
      const adds: DiffLineData[] = [];
      while (i < lines.length && lines[i].kind === "add") {
        adds.push(lines[i]);
        i++;
      }
      const max = Math.max(dels.length, adds.length);
      for (let j = 0; j < max; j++) {
        const d = dels[j];
        const a = adds[j];
        result.push({
          left: d ? { kind: "del", no: d.oldNo, text: d.text } : null,
          right: a ? { kind: "add", no: a.newNo, text: a.text, commentable: a.commentable, _srcIndex: lines.indexOf(a) } : null,
        });
      }
    }
    return result;
  }

  let splitRows = $derived(pairLines(diffLines));
</script>

{#snippet commentBlock(lineStart: number, lineEnd: number | undefined, comments: Comment[], onComment: () => void, onClose: () => void)}
  <div class="thread-anchor">
    <div class="thread-avatar">
      <Avatar name="Yu-Chen" size={24} />
    </div>
    <div class="thread-body">
      <CommentThread file={selected} {lineStart} {lineEnd} {comments} bind:replyValue {onComment} {onClose} />
    </div>
  </div>
{/snippet}

<div class="app">
  <header class="topbar">
    <div class="brand">
      <Icon name="git-pull-request" size={18} color="var(--accent)" />
      <span class="wordmark">Ziff</span>
    </div>
    <Dropdown icon="folder" label="Project" options={PROJECTS} value={project} onChange={(v) => (project = v)} width={260} />
    <Icon name="chevron-right" size={12} color="var(--border-default)" />
    <Dropdown icon="git-branch" label="Branch" options={BRANCHES} value={branch} onChange={(v) => (branch = v)} width={280} />
    <div class="topbar-spacer"></div>
    <FetchButton />
  </header>

  <div class="body">
    <aside class="sidebar">
      <div class="sidebar-toolbar">
        <Segmented value={diffMode} onChange={(v) => (diffMode = v)} options={DIFF_MODES} />
      </div>
      <div class="sidebar-tree">
        <FileTree {tree} {selected} onSelect={selectFile} />
      </div>
    </aside>

    <main class="diff-panel">
      <div class="diff-panel-header">
        <div class="file-header-wrap"><FileHeader path={selected} /></div>
        <div class="view-toggle-wrap"><Segmented value={view} onChange={(v) => (view = v)} options={VIEW_MODES} /></div>
      </div>

      <DiffHunk label="@@ -10,5 +10,5 @@ fn load_workbook(path: &Path) -> Result<Xlsx<...>>" />

      {#if view === "unified"}
        {#each diffLines as line, i (i)}
          <DiffLine
            kind={line.kind}
            oldNo={line.oldNo}
            newNo={line.newNo}
            commentable={line.commentable}
            onAddComment={() => (openLineThread = true)}
            index={i}
            selected={isSelected(i)}
            onGutterDown={gutterDown}
            onGutterEnter={gutterEnter}
          >
            {line.text}
          </DiffLine>
          {#if line.commentHere && openLineThread}
            {@render commentBlock(rangeLineNo(commentHereIndex), undefined, [fixedComment], submitFixedComment, () => (openLineThread = false))}
          {/if}
          {#if range && i === range.hi}
            {@render commentBlock(rangeLineNo(range.lo), rangeLineNo(range.hi), rangeComments, submitRangeComment, () => {
              range = null;
              rangeComments = [];
            })}
          {/if}
        {/each}
      {:else}
        {#each splitRows as row, i (i)}
          <DiffLineSplit left={row.left} right={row.right} onAddComment={() => (openLineThread = true)} />
          {#if row.right?._srcIndex !== undefined && diffLines[row.right._srcIndex]?.commentHere && openLineThread}
            {@render commentBlock(rangeLineNo(commentHereIndex), undefined, [fixedComment], submitFixedComment, () => (openLineThread = false))}
          {/if}
        {/each}
      {/if}
    </main>

    {#if contextOpen}
      <ContextDrawer items={savedContext} onRemove={removeFromContext} onClose={() => (contextOpen = false)} />
    {/if}
  </div>

  <ContextFab count={savedContext.length} open={contextOpen} onclick={() => (contextOpen = !contextOpen)} />
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

  .sidebar-tree {
    flex: 1;
    min-height: 0;
    padding: var(--space-2);
    overflow-y: auto;
  }

  .diff-panel {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    background: var(--bg-canvas);
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
