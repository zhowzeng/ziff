<script>
  import Icon from './Icon.svelte';

  let { icon, label, sublabel, options, value, onChange, width = 240, onAddNew, addNewLabel = 'Add new…' } = $props();

  let open = $state(false);
  let root = $state();

  $effect(() => {
    if (!open) return;
    function onDown(e) {
      if (root && !root.contains(e.target)) open = false;
    }
    window.addEventListener('mousedown', onDown);
    return () => window.removeEventListener('mousedown', onDown);
  });

  let current = $derived(options.find((o) => o.value === value));
</script>

<div bind:this={root} style="position:relative;min-width:0;flex-shrink:1" aria-label={label}>
  <button
    onclick={() => (open = !open)}
    style={`display:flex;align-items:center;gap:6px;height:28px;padding:0 8px;min-width:0;width:100%;
      background:${open ? 'var(--bg-subtle)' : 'transparent'};border:1px solid ${open ? 'var(--border-default)' : 'transparent'};
      border-radius:var(--radius-sm);cursor:pointer;font-family:var(--font-sans);max-width:220px`}
  >
    {#if icon}
      <Icon name={icon} size={13} color="var(--text-tertiary)" />
    {/if}
    <span style="font-size:var(--text-sm);font-weight:500;color:var(--text-primary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
      {current ? current.label : ''}
    </span>
    <Icon name="chevron-down" size={12} color="var(--text-tertiary)" />
  </button>

  {#if open}
    <div
      style={`position:absolute;top:100%;left:0;margin-top:4px;width:${width}px;z-index:30;
        background:var(--gray-0);border:1px solid var(--border-default);border-radius:var(--radius-md);
        box-shadow:var(--shadow-lg);padding:4px`}
    >
      {#if sublabel}
        <div style="padding:4px 8px;font-family:var(--font-sans);font-size:var(--text-xs);color:var(--text-tertiary)">{sublabel}</div>
      {/if}
      {#each options as o (o.value)}
        <div
          class="option"
          class:selected={o.value === value}
          onclick={() => {
            onChange(o.value);
            open = false;
          }}
          style="display:flex;align-items:center;gap:8px;padding:6px 8px;border-radius:var(--radius-sm);cursor:pointer"
        >
          <div style="width:14px;display:flex;justify-content:center;flex-shrink:0">
            {#if o.value === value}
              <Icon name="check" size={13} color="var(--accent-emphasis)" />
            {/if}
          </div>
          <div style="min-width:0">
            <div style="font-family:var(--font-sans);font-size:var(--text-sm);color:var(--text-primary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{o.label}</div>
            {#if o.meta}
              <div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary)">{o.meta}</div>
            {/if}
          </div>
        </div>
      {/each}
      {#if onAddNew}
        <div class="divider"></div>
        <div
          class="option add-new"
          onclick={() => {
            open = false;
            onAddNew();
          }}
          style="display:flex;align-items:center;gap:8px;padding:6px 8px;border-radius:var(--radius-sm);cursor:pointer"
        >
          <div style="width:14px;display:flex;justify-content:center;flex-shrink:0">
            <Icon name="folder-plus" size={13} color="var(--accent-emphasis)" />
          </div>
          <div style="font-family:var(--font-sans);font-size:var(--text-sm);font-weight:500;color:var(--accent-emphasis)">{addNewLabel}</div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .option {
    background: transparent;
  }
  .option:hover,
  .option.selected {
    background: var(--bg-subtle);
  }
  .divider {
    height: 1px;
    background: var(--border-default);
    margin: 4px 2px;
  }
</style>
