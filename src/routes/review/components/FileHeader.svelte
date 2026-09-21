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
</script>

<div style="height:38px;display:flex;align-items:center;gap:8px;padding:0 12px;background:var(--bg-subtle)">
  <Icon name="chevron-down" size={13} color="var(--text-tertiary)" />
  {#if renamedFrom}
    <!-- A moved file shows both ends: the diff below is against its own old content,
         not a whole-file delete and re-add. -->
    <span style="font-family:var(--font-mono);font-size:var(--text-sm);color:var(--text-tertiary);min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{renamedFrom}</span>
    <Icon name="arrow-right" size={12} color="var(--text-tertiary)" />
  {/if}
  <span style="font-family:var(--font-mono);font-size:var(--text-sm);color:var(--text-secondary);min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{path}</span>
  <div style="margin-left:auto;display:flex;gap:2px;flex-shrink:0">
    <IconButton icon="copy" title="Copy path" size={24} onclick={copyPath} />
    {#if onToggleHunks}
      <IconButton
        icon={allCollapsed ? 'chevrons-up-down' : 'chevrons-down-up'}
        title={allCollapsed ? 'Expand all hunks' : 'Collapse all hunks'}
        size={24}
        onclick={onToggleHunks}
      />
    {/if}
  </div>
</div>
