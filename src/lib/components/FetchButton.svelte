<script>
  /**
   * @typedef {Object} Props
   * @property {boolean} [fetching]
   * @property {string} [lastFetched]
   * @property {() => void} onFetch
   */

  import Icon from './Icon.svelte';

  /** @type {Props} */
  let { fetching = false, lastFetched = '尚未 fetch', onFetch } = $props();
</script>

<button class="fetch-btn" onclick={onFetch} title={`上次 fetch：${lastFetched}`}>
  <Icon name="refresh-cw" size={13} color="var(--text-tertiary)" class={fetching ? 'spin' : ''} />
  Fetch
</button>

<style>
  .fetch-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-default);
    background: var(--bg-raised);
    box-shadow: var(--shadow-sm);
    cursor: pointer;
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--text-secondary);
    transition: background var(--dur-fast) ease, border-color var(--dur-fast) ease;
  }
  .fetch-btn:hover {
    background: var(--bg-subtle);
    border-color: var(--border-strong);
    color: var(--text-primary);
  }
  .fetch-btn :global(.spin) {
    animation: z-spin 0.7s linear infinite;
  }
</style>
