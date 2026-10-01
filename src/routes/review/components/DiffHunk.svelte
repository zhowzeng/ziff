<script>
  import Icon from '$lib/components/Icon.svelte';
  import { nextHunkShortcut, prevHunkShortcut } from '../shortcuts';

  /**
   * @typedef {Object} Props
   * @property {string} label
   * @property {boolean} [collapsed]
   * @property {() => void} onToggle
   */

  /** @type {Props} */
  let { label, collapsed = false, onToggle } = $props();

  // `@@ -10,9 +10,14 @@ function name` → the range pill and the enclosing symbol, set apart.
  let parts = $derived.by(() => {
    const m = /^(@@[^@]*@@)\s*(.*)$/.exec(label);
    return m ? { range: m[1], ctx: m[2] } : { range: '', ctx: label };
  });
</script>

<button
  class="hunk"
  onclick={onToggle}
  data-hunk-header
  title={`${collapsed ? 'Expand hunk' : 'Collapse hunk'}（${nextHunkShortcut.label} / ${prevHunkShortcut.label} 跳到下一個／上一個 hunk）`}
>
  <span class="chev" class:open={!collapsed}><Icon name="chevron-right" size={12} /></span>
  {#if parts.range}<span class="range">{parts.range}</span>{/if}
  {#if parts.ctx}<span class="ctx">{parts.ctx}</span>{/if}
</button>

<style>
  .hunk {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    text-align: left;
    cursor: pointer;
    background: var(--diff-hunk-bg);
    color: var(--diff-hunk-text);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: 5px 12px;
    border: none;
    border-top: 1px solid var(--border-muted);
    border-bottom: 1px solid var(--border-muted);
    transition: filter var(--dur-fast) ease;
  }
  .hunk:hover {
    filter: brightness(0.97);
  }
  .chev {
    display: inline-flex;
    transition: transform var(--dur-base) var(--ease-out);
  }
  .chev.open {
    transform: rotate(90deg);
  }
  .range {
    color: var(--text-link);
    font-weight: 500;
    flex-shrink: 0;
  }
  .ctx {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
