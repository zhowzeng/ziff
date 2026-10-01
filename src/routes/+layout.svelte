<script>
  import '$lib/styles/tokens/index.css';
  import '$lib/styles/global.css';
  import Toast from '$lib/components/Toast.svelte';
  import { settings } from '$lib/settings/state.svelte';

  let { children } = $props();

  // "system" leaves data-theme off so the prefers-color-scheme rule in colors.css decides.
  $effect(() => {
    const root = document.documentElement;
    if (settings.theme === 'system') root.removeAttribute('data-theme');
    else root.dataset.theme = settings.theme;
  });
</script>

{@render children?.()}
<Toast />
