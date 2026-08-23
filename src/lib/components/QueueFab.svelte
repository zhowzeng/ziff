<script>
  import Icon from './Icon.svelte';

  let { count = 0, open = false, onclick } = $props();
</script>

<button
  {onclick}
  title="Comment Queue"
  aria-label="Comment Queue"
  style={`position:fixed;right:20px;bottom:20px;width:52px;height:52px;border-radius:50%;
    background:${open ? 'var(--accent-emphasis)' : 'var(--gray-0)'};box-shadow:var(--shadow-lg);
    border:${open ? 'none' : '1px solid var(--border-default)'};
    display:flex;align-items:center;justify-content:center;cursor:pointer;z-index:20;padding:0`}
>
  <Icon name="message-square" size={20} color={open ? 'var(--gray-0)' : 'var(--text-secondary)'} />
  {#if count > 0}
    <div
      style="position:absolute;top:-2px;right:-2px;min-width:20px;height:20px;border-radius:10px;background:var(--danger-emphasis);
        color:var(--gray-0);font-family:var(--font-sans);font-size:11px;font-weight:700;line-height:20px;text-align:center;padding:0 5px;
        border:2px solid var(--bg-canvas)"
    >{count}</div>
  {/if}
</button>
