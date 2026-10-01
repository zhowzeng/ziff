<script>
  import Icon from '$lib/components/Icon.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import { toast } from '$lib/toast/state.svelte';

  /**
   * @typedef {Object} Props
   * @property {string} path
   * @property {string|null} [renamedFrom]
   * @property {boolean} [allCollapsed]
   * @property {(() => void)|null} [onToggleHunks] File View has no hunks, so it passes null and the toggle stays off screen.
   */

  /** @type {Props} */
  let { path, renamedFrom = null, allCollapsed = false, onToggleHunks = null } = $props();

  // The path is what a comment for the CLI agent is anchored to, so it's copied
  // repo-relative, exactly as it reads in the comment.
  async function copyPath() {
    try {
      await navigator.clipboard.writeText(path);
    } catch (e) {
      toast(`複製到剪貼簿失敗：${e}`, { variant: 'danger' });
      return;
    }
    toast('已複製檔案路徑', { variant: 'success' });
  }

  // The file name carries the weight; its folders recede, like a breadcrumb.
  let slash = $derived(path.lastIndexOf('/'));
  let dir = $derived(slash >= 0 ? path.slice(0, slash + 1) : '');
  let base = $derived(slash >= 0 ? path.slice(slash + 1) : path);
</script>

<div class="header">
  {#if renamedFrom}
    <!-- A moved file shows both ends: the diff below is against its own old content,
         not a whole-file delete and re-add. -->
    <span class="old">{renamedFrom}</span>
    <Icon name="arrow-right" size={12} color="var(--text-tertiary)" />
  {/if}
  <span class="path"><span class="dir">{dir}</span><span class="base">{base}</span></span>
  <div class="actions">
    <IconButton icon="copy" title="Copy path" size={26} onclick={copyPath} />
    {#if onToggleHunks}
      <IconButton
        icon={allCollapsed ? 'chevrons-up-down' : 'chevrons-down-up'}
        title={allCollapsed ? 'Expand all hunks' : 'Collapse all hunks'}
        size={26}
        onclick={onToggleHunks}
      />
    {/if}
  </div>
</div>

<style>
  .header {
    height: 44px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px 0 16px;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    min-width: 0;
  }
  .old,
  .dir {
    color: var(--text-tertiary);
  }
  /* Long paths give up their folders first: the file name is what the reviewer is
     looking for, so it keeps its width and only the folder part is cut short. */
  .path {
    display: flex;
    min-width: 0;
    white-space: nowrap;
  }
  .old,
  .dir {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .base {
    flex-shrink: 0;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-primary);
    font-weight: 600;
  }
  .actions {
    margin-left: auto;
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }
</style>
