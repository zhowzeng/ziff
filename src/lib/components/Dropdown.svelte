<script>
  import Icon from './Icon.svelte';

  /**
   * @typedef {Object} DropdownOption
   * @property {string} value
   * @property {string} label
   * @property {string} [meta]
   */

  /**
   * @typedef {Object} OptionAction
   * @property {string} icon
   * @property {string} title
   * @property {(value: string) => void} onAction
   */

  /**
   * @typedef {Object} Props
   * @property {string} [icon]
   * @property {string} [label]
   * @property {string} [sublabel]
   * @property {DropdownOption[]} options
   * @property {string} value
   * @property {(value: string) => void} onChange
   * @property {number} [width]
   * @property {string} [placeholder]
   * @property {OptionAction} [optionAction]
   * @property {() => void} [onAddNew]
   * @property {string} [addNewLabel]
   */

  /** @type {Props} */
  let { icon, label, sublabel, options, value, onChange, width = 240, onAddNew, addNewLabel = 'Add new…', placeholder = '', optionAction } = $props();

  let open = $state(false);
  let root = $state();

  $effect(() => {
    if (!open) return;
    /** @param {MouseEvent} e */
    function onDown(e) {
      if (root && !root.contains(/** @type {Node} */ (e.target))) open = false;
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
      {current ? current.label : placeholder}
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
          role="button"
          tabindex="0"
          onclick={() => {
            onChange(o.value);
            open = false;
          }}
          onkeydown={(e) => {
            if (e.key === ' ' || e.key === 'Enter') {
              e.preventDefault();
              onChange(o.value);
              open = false;
            }
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
          {#if optionAction}
            <!-- The caller says what this does; the dropdown only gives it a place to sit. -->
            <button
              class="option-action"
              title={optionAction.title}
              onclick={(e) => {
                e.stopPropagation();
                open = false;
                optionAction.onAction(o.value);
              }}
              style="margin-left:auto;flex-shrink:0;display:flex;align-items:center;justify-content:center;width:20px;height:20px;padding:0;border:none;background:transparent;border-radius:var(--radius-sm);cursor:pointer"
            >
              <Icon name={optionAction.icon} size={13} color="var(--text-tertiary)" />
            </button>
          {/if}
        </div>
      {/each}
      {#if onAddNew}
        <div class="divider"></div>
        <div
          class="option add-new"
          role="button"
          tabindex="0"
          onclick={() => {
            open = false;
            onAddNew();
          }}
          onkeydown={(e) => {
            if (e.key === ' ' || e.key === 'Enter') {
              e.preventDefault();
              open = false;
              onAddNew();
            }
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
  .option-action {
    opacity: 0;
  }
  .option:hover .option-action,
  .option-action:focus-visible {
    opacity: 1;
  }
  .option-action:hover {
    background: var(--border-default);
  }
  .divider {
    height: 1px;
    background: var(--border-default);
    margin: 4px 2px;
  }
</style>
