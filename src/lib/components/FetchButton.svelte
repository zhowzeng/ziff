<script>
  import Icon from './Icon.svelte';

  let fetching = $state(false);
  let lastFetched = $state('剛剛');

  function doFetch() {
    if (fetching) return;
    fetching = true;
    setTimeout(() => {
      fetching = false;
      lastFetched = '剛剛';
    }, 900);
  }
</script>

<button
  onclick={doFetch}
  title={`上次 fetch：${lastFetched}`}
  style={`display:flex;align-items:center;gap:6px;height:28px;padding:0 10px;border-radius:var(--radius-sm);
    border:1px solid var(--border-default);background:var(--gray-0);cursor:${fetching ? 'default' : 'pointer'};
    font-family:var(--font-sans);font-size:var(--text-xs);font-weight:500;color:var(--text-secondary)`}
>
  <Icon name="refresh-cw" size={13} color="var(--text-tertiary)" class={fetching ? 'spin' : ''} />
  Fetch
</button>

<style>
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  button :global(.spin) {
    animation: spin 0.7s linear infinite;
  }
</style>
