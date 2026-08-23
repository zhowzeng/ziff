<script>
  import { icons } from 'lucide-svelte';

  let { name, size = 16, color = 'currentColor', strokeWidth = 1.75, class: className = '' } = $props();

  function toPascalCase(kebab) {
    return kebab.split('-').map((s) => s[0].toUpperCase() + s.slice(1)).join('');
  }

  let Comp = $derived(icons[toPascalCase(name)]);
</script>

{#if Comp}
  <Comp {size} {color} {strokeWidth} class={className} aria-hidden="true" />
{/if}
