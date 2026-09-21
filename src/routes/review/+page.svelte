<script lang="ts">
  import { onMount } from "svelte";
  import DiffPanel from "./components/DiffPanel.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Topbar from "./components/Topbar.svelte";
  import QueueDrawer from "./components/QueueDrawer.svelte";
  import QueueFab from "./components/QueueFab.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import Button from "$lib/components/Button.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import { reviewState } from "./state.svelte";
  import { commentQueue } from "./comment-queue.svelte";
  import { selection } from "./selection.svelte";
  import { formatQueueForAgent, pruneToChanged } from "./helpers";
  import { copyAllShortcut } from "./shortcuts";
  import { toast } from "$lib/toast/state.svelte";
  import { DIFF_FONT_SIZE_PX, settings, updateSettings } from "$lib/settings/state.svelte";
  import type { DiffMode, Repo } from "./types";

  onMount(() => {
    reviewState.loadRepos();
  });

  let settingsOpen = $state(false);
  let removeTarget = $state<Repo | null>(null);

  let changedTree = $derived(pruneToChanged(reviewState.tree));
  // Nothing has been added on this machine yet, so the reviewer's next step is the
  // folder picker rather than the Repo dropdown.
  let noRepos = $derived(reviewState.repos.length === 0);
  let loadingFiles = $derived(reviewState.loadingRepos || reviewState.loadingBranches || reviewState.loadingTree);

  // A queue belongs to one Repo (docs/decisions/0009), so everything read out of it
  // here is scoped to the Repo being reviewed.
  let queueItems = $derived(commentQueue.itemsFor(reviewState.repoId));

  // The shortcut lives here rather than in the drawer, which is only rendered while it
  // is open — the button printing this shortcut would otherwise promise keys that stop
  // working the moment the drawer is closed.
  $effect(() => {
    function onKey(e: KeyboardEvent) {
      // Nothing to copy means nothing to swallow the keystroke for.
      if (!copyAllShortcut.matches(e) || queueItems.length === 0) return;
      e.preventDefault();
      copyAllForAgent();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  // Fired from the drawer's button and from the shortcut, which can run with the drawer
  // closed — so the toast is the only feedback that anything happened.
  async function copyAllForAgent() {
    const prefix = settings.usePrefixPrompt ? settings.prefixPrompt.trim() : "";
    try {
      await navigator.clipboard.writeText(formatQueueForAgent(queueItems, prefix));
    } catch (e) {
      toast(`複製到剪貼簿失敗：${e}`, { variant: "danger" });
      return;
    }
    toast(`已複製 ${queueItems.length} 則 comment`, { variant: "success" });
  }

  // Anything that replaces the diff on screen drops the selection with it: the lines it
  // named are no longer the lines on screen.
  function addRepo() {
    selection.close();
    reviewState.addRepo();
  }
  // Removing a Repo discards its Comment Queue (docs/decisions/0009), so it asks first
  // — but only when there is something to lose.
  function requestRemoveRepo(id: string) {
    const repo = reviewState.repos.find((r) => r.id === id);
    if (!repo) return;
    if (commentQueue.itemsFor(id).length === 0) {
      removeRepo(repo.id);
      return;
    }
    removeTarget = repo;
  }
  function removeRepo(id: string) {
    removeTarget = null;
    selection.close();
    reviewState.removeRepo(id);
  }
  function selectRepo(id: string) {
    selection.close();
    reviewState.selectRepo(id);
  }
  function setBaseBranch(name: string) {
    selection.close();
    reviewState.setBaseBranch(name);
  }
  function setDiffMode(mode: string) {
    selection.close();
    reviewState.setDiffMode(mode as DiffMode);
  }
  function selectFile(path: string) {
    selection.close();
    reviewState.selectFile(path);
  }
</script>

<div class="app" style="--diff-font-size:{DIFF_FONT_SIZE_PX[settings.diffFontSize]}">
  <Topbar
    repos={reviewState.repos}
    repoId={reviewState.repoId}
    branch={reviewState.branch}
    detachedHead={reviewState.detachedHead}
    branches={reviewState.branches}
    baseBranch={reviewState.baseBranch}
    diffMode={reviewState.diffMode}
    fetching={reviewState.fetching}
    lastFetched={reviewState.lastFetched}
    onSelectRepo={selectRepo}
    onAddRepo={addRepo}
    onRemoveRepo={requestRemoveRepo}
    onSetBaseBranch={setBaseBranch}
    onFetch={() => reviewState.fetchRemoteBranch()}
  />

  <div class="body">
    <Sidebar
      tree={reviewState.tree}
      {changedTree}
      diffMode={reviewState.diffMode}
      repoId={reviewState.repoId}
      detachedHead={reviewState.detachedHead}
      selectedFile={reviewState.selectedFile}
      loading={loadingFiles}
      {noRepos}
      onSetDiffMode={setDiffMode}
      onSelectFile={selectFile}
      onOpenSettings={() => (settingsOpen = true)}
    />

    <DiffPanel
      repoId={reviewState.repoId}
      selectedFile={reviewState.selectedFile}
      selectedRenamedFrom={reviewState.selectedRenamedFrom}
      selectedView={reviewState.selectedView}
      view={reviewState.view}
      onViewChange={(v) => (reviewState.view = v)}
      diffHunks={reviewState.diffHunks}
      diffBinary={reviewState.diffBinary}
      loadingDiff={reviewState.loadingDiff}
      fileLines={reviewState.fileLines}
      fileBinary={reviewState.fileBinary}
      loadingFile={reviewState.loadingFile}
      loading={loadingFiles}
      {noRepos}
      hasChanges={changedTree.length > 0}
    />

    {#if commentQueue.open}
      <QueueDrawer
        items={queueItems}
        repoName={reviewState.repo?.name}
        onRemove={(id) => commentQueue.remove(id)}
        onCopyAll={copyAllForAgent}
        onClose={() => (commentQueue.open = false)}
      />
    {/if}
  </div>

  <QueueFab count={queueItems.length} open={commentQueue.open} onclick={() => (commentQueue.open = !commentQueue.open)} />
  <SettingsModal open={settingsOpen} onClose={() => (settingsOpen = false)} {settings} onChange={updateSettings} />

  <Modal open={removeTarget !== null} onClose={() => (removeTarget = null)} title="移除 repo？" width={400}>
    {#if removeTarget}
      <p class="modal-text">
        <strong>{removeTarget.name}</strong> 會從 Ziff 的清單移除，資料夾本身不會被刪除，之後可以再加回來。
      </p>
      <p class="modal-text modal-warning">
        這會一併丟掉 {commentQueue.itemsFor(removeTarget.id).length} 則還沒交出去的 comment。
      </p>
    {/if}
    {#snippet footer()}
      <Button size="sm" onclick={() => (removeTarget = null)}>取消</Button>
      <Button variant="danger" size="sm" onclick={() => removeTarget && removeRepo(removeTarget.id)}>移除</Button>
    {/snippet}
  </Modal>
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

  .modal-text {
    margin: 0;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    line-height: 1.6;
    color: var(--text-secondary);
  }
  .modal-warning {
    margin-top: 10px;
    color: var(--danger-emphasis);
  }
  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
</style>
