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
</script>

<button
  onclick={onToggle}
  data-hunk-header
  title={`${collapsed ? 'Expand hunk' : 'Collapse hunk'}（${nextHunkShortcut.label} / ${prevHunkShortcut.label} 跳到下一個／上一個 hunk）`}
  style="width:100%;display:flex;align-items:center;gap:6px;text-align:left;cursor:pointer;background:var(--diff-hunk-bg);color:var(--diff-hunk-text);font-family:var(--font-mono);font-size:var(--text-xs);padding:4px 12px;border:none;border-top:1px solid var(--border-muted);border-bottom:1px solid var(--border-muted)"
>
  <Icon name={collapsed ? 'chevron-right' : 'chevron-down'} size={12} />
  <span style="min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{label}</span>
</button>
