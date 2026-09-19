<script>
  import Icon from '$lib/components/Icon.svelte';
  import IconButton from '$lib/components/IconButton.svelte';

  let { path, renamedFrom = null } = $props();
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
    <IconButton icon="copy" title="Copy path" size={24} />
    <IconButton icon="chevrons-up-down" title="Expand all" size={24} />
  </div>
</div>
