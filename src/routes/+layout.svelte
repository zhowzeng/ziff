<script>
  import '$lib/styles/tokens/index.css';
  import Toast from '$lib/components/Toast.svelte';

  let { children } = $props();
</script>

<style>
  :global(body) {
    margin: 0;
    background: var(--bg-canvas);
    color: var(--text-primary);
  }
</style>

{@render children?.()}
<Toast />
