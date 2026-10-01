<script>
  import Icon from './Icon.svelte';

  /**
   * @typedef {Object} Props
   * @property {import('./Icon.svelte').IconName} icon
   * @property {number} [size]
   * @property {string} [title]
   * @property {(e: MouseEvent) => void} [onclick]
   * @property {boolean} [active]
   */

  /** @type {Props} */
  let { icon, size = 28, title, onclick, active = false } = $props();
</script>

<button class="icon-btn" class:active {onclick} {title} aria-label={title} style:width="{size}px" style:height="{size}px">
  <Icon name={icon} size={15} />
</button>

<style>
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background var(--dur-fast) ease, color var(--dur-fast) ease;
  }
  .icon-btn:hover {
    background: var(--bg-inset);
    color: var(--text-primary);
  }
  .icon-btn:active {
    background: var(--border-default);
  }
  .icon-btn.active {
    background: var(--bg-inset);
    color: var(--text-primary);
  }
</style>
