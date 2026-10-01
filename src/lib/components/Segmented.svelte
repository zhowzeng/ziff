<script>
  /**
   * @typedef {Object} SegmentedOption
   * @property {string} value
   * @property {string} label
   */

  /**
   * @typedef {Object} Props
   * @property {string} value
   * @property {(value: string) => void} onChange
   * @property {SegmentedOption[]} options
   */

  /** @type {Props} */
  let { value, onChange, options } = $props();

  let index = $derived(Math.max(0, options.findIndex((o) => o.value === value)));
</script>

<!-- Equal-width cells, so one pill can slide to the active one with a plain transform. -->
<div class="group" role="group" style:--n={options.length} style:--i={index}>
  <span class="pill" aria-hidden="true"></span>
  {#each options as o (o.value)}
    <button class="seg" class:active={value === o.value} aria-pressed={value === o.value} onclick={() => onChange(o.value)}>{o.label}</button>
  {/each}
</div>

<style>
  .group {
    position: relative;
    display: inline-grid;
    grid-template-columns: repeat(var(--n), 1fr);
    padding: 2px;
    gap: 0;
    border-radius: var(--radius-md);
    background: var(--bg-inset);
    box-shadow: inset 0 0 0 1px var(--border-muted);
  }
  .pill {
    position: absolute;
    top: 2px;
    bottom: 2px;
    left: 2px;
    width: calc((100% - 4px) / var(--n));
    border-radius: calc(var(--radius-md) - 2px);
    background: var(--bg-active);
    box-shadow: var(--shadow-sm), 0 0 0 1px var(--border-default);
    transform: translateX(calc(var(--i) * 100%));
    transition: transform var(--dur-base) var(--ease-out);
  }
  .seg {
    position: relative;
    z-index: 1;
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    font-weight: 500;
    padding: 4px 12px;
    border: none;
    border-radius: calc(var(--radius-md) - 2px);
    cursor: pointer;
    background: transparent;
    color: var(--text-secondary);
    white-space: nowrap;
    transition: color var(--dur-fast) ease;
  }
  .seg:hover {
    color: var(--text-primary);
  }
  .seg.active {
    color: var(--accent-emphasis);
    font-weight: 600;
  }
</style>
