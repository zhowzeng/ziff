<script>
  import Icon from './Icon.svelte';

  let { icon, title, hint = undefined, size = 'md' } = $props();

  let iconSize = $derived(size === 'md' ? 28 : 22);
  let titleSize = $derived(size === 'md' ? 'var(--text-md)' : 'var(--text-sm)');
  let titleWeight = $derived(size === 'md' ? 600 : 500);
  let hintSize = $derived(size === 'md' ? 'var(--text-sm)' : 'var(--text-xs)');
  let hintWidth = $derived(size === 'md' ? 320 : 200);
  let gap = $derived(size === 'md' ? 10 : 8);
</script>

<div style={`flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:${gap}px;padding:24px 16px;text-align:center`}>
  <Icon name={icon} size={iconSize} color="var(--text-tertiary)" />
  <div style={`font-family:var(--font-sans);font-size:${titleSize};font-weight:${titleWeight};color:var(--text-secondary)`}>{title}</div>
  {#if hint}
    <div style={`font-family:var(--font-sans);font-size:${hintSize};color:var(--text-tertiary);max-width:${hintWidth}px`}>{hint}</div>
  {/if}
</div>
