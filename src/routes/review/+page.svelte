<script lang="ts">
  import FileTree from "$lib/components/FileTree.svelte";
  import DiffHunk from "$lib/components/DiffHunk.svelte";
  import DiffLine from "$lib/components/DiffLine.svelte";
  import CommentThread from "$lib/components/CommentThread.svelte";
  import Avatar from "$lib/components/Avatar.svelte";
  import Badge from "$lib/components/Badge.svelte";
  import IconButton from "$lib/components/IconButton.svelte";
  import Icon from "$lib/components/Icon.svelte";

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

  let selected = $state("src/xlsx_tool.rs");
  let threadOpen = $state(true);
  let replyValue = $state("");

  const diffLines = [
    { kind: "context", oldNo: 10, newNo: 10, text: "use calamine::{open_workbook, Reader, Xlsx};" },
    { kind: "del", oldNo: 11, newNo: null, text: 'calamine = "0.22"' },
    { kind: "add", oldNo: null, newNo: 11, text: 'calamine = "0.24"' },
    { kind: "context", oldNo: 12, newNo: 12, text: "" },
    { kind: "del", oldNo: 13, newNo: null, text: "let mut wb: Xlsx<_> = open_workbook(path)?;" },
    { kind: "add", oldNo: null, newNo: 13, text: "let mut wb: Xlsx<_> = open_workbook_auto(path)?;", commentable: true },
    { kind: "context", oldNo: 14, newNo: 14, text: "let sheet = wb.worksheet_range(\"Sheet1\")?;" },
  ];
</script>

<div class="app">
  <header class="topbar">
    <div class="brand">
      <Icon name="git-pull-request" size={18} color="var(--accent)" />
      <span class="wordmark">Ziff</span>
    </div>
    <span class="pr-title">goose · feature/xlsx-api-upgrade</span>
    <Badge variant="accent">unstaged</Badge>
  </header>

  <div class="body">
    <aside class="sidebar">
      <FileTree {tree} {selected} onSelect={(path) => (selected = path)} />
    </aside>

    <main class="diff-panel">
      <div class="diff-header">
        <Icon name="file-code" size={14} color="var(--text-tertiary)" />
        <span class="file-path">{selected}</span>
        <div class="diff-header-actions">
          <IconButton icon="copy" title="Copy diff" />
        </div>
      </div>

      <DiffHunk label="@@ -10,5 +10,5 @@ fn load_workbook(path: &Path) -> Result<Xlsx<...>>" />

      {#each diffLines as line, i (i)}
        <DiffLine kind={line.kind} oldNo={line.oldNo} newNo={line.newNo} commentable={line.commentable} onAddComment={() => (threadOpen = true)}>
          {line.text}
        </DiffLine>
        {#if line.commentable && threadOpen}
          <div class="thread-anchor">
            <div class="thread-avatar">
              <Avatar name="Yu-Chen" size={24} />
            </div>
            <div class="thread-body">
              <CommentThread
                file={selected}
                lineStart={13}
                comments={[{ time: "2m ago", text: "跳大版有改 api", editable: true }]}
                bind:replyValue
                onClose={() => (threadOpen = false)}
              />
            </div>
          </div>
        {/if}
      {/each}
    </main>
  </div>
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
    gap: var(--space-3);
    height: 48px;
    padding: 0 var(--space-4);
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-surface);
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .wordmark {
    font-size: var(--text-base);
    font-weight: 700;
    color: var(--text-primary);
  }

  .pr-title {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .sidebar {
    width: 240px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-default);
    background: var(--bg-surface);
    padding: var(--space-2);
    overflow-y: auto;
  }

  .diff-panel {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    background: var(--bg-canvas);
  }

  .diff-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-surface);
    position: sticky;
    top: 0;
  }

  .file-path {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-primary);
  }

  .diff-header-actions {
    margin-left: auto;
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
