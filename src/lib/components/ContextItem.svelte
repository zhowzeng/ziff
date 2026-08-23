<script>
  import Icon from './Icon.svelte';
  import IconButton from './IconButton.svelte';

  let { item, onRemove, onCopy } = $props();

  let range = $derived(
    item.lineEnd && item.lineEnd !== item.lineStart ? `L${item.lineStart}–L${item.lineEnd}` : `L${item.lineStart}`
  );
</script>

<div style="border:1px solid var(--border-default);border-radius:var(--radius-md);background:var(--gray-0);overflow:hidden">
  <div style="display:flex;align-items:center;gap:6px;padding:6px 10px;background:var(--bg-subtle);border-bottom:1px solid var(--border-muted)">
    <Icon name="file-text" size={12} color="var(--text-tertiary)" />
    <span style="font-family:var(--font-mono);font-size:var(--text-xs);color:var(--text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{item.file}</span>
    <span style="font-family:var(--font-mono);font-size:var(--text-xs);color:var(--text-tertiary)">{range}</span>
    <div style="margin-left:auto;display:flex;gap:2px">
      <IconButton icon="copy" title="Copy" size={20} onclick={() => onCopy(item)} />
      <IconButton icon="x" title="Remove" size={20} onclick={() => onRemove(item.id)} />
    </div>
  </div>
  <div style="padding:8px 10px;font-family:var(--font-sans);font-size:var(--text-sm);color:var(--text-primary);line-height:var(--leading-normal)">{item.text}</div>
</div>
