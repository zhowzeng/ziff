<script>
  import IconButton from './IconButton.svelte';

  let { open = false, onClose, title, width = 420, children, footer } = $props();

  $effect(() => {
    if (!open) return;
    function onKey(e) {
      if (e.key === 'Escape') onClose?.();
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });
</script>

{#if open}
  <div
    role="presentation"
    onmousedown={(e) => { if (e.target === e.currentTarget) onClose?.(); }}
    style="position:fixed;inset:0;background:var(--bg-scrim);z-index:80;display:flex;align-items:center;justify-content:center"
  >
    <div
      role="dialog"
      aria-modal="true"
      aria-label={title}
      style={`width:${width}px;max-width:calc(100vw - 32px);max-height:calc(100vh - 64px);display:flex;flex-direction:column;
        background:var(--gray-0);border:1px solid var(--border-default);border-radius:var(--radius-lg);box-shadow:var(--shadow-lg)`}
    >
      {#if title}
        <div style="display:flex;align-items:center;gap:8px;padding:14px 16px;border-bottom:1px solid var(--border-default)">
          <span style="flex:1;font-family:var(--font-sans);font-weight:600;font-size:var(--text-base);color:var(--text-primary)">{title}</span>
          <IconButton icon="x" title="Close" size={24} onclick={onClose} />
        </div>
      {/if}
      <div style="padding:16px;overflow-y:auto">
        {@render children?.()}
      </div>
      {#if footer}
        <div style="display:flex;justify-content:flex-end;gap:8px;padding:12px 16px;border-top:1px solid var(--border-default)">
          {@render footer?.()}
        </div>
      {/if}
    </div>
  </div>
{/if}
