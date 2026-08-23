<script>
  import { icons } from 'lucide-svelte';

  /**
   * @typedef {Object} Props
   * @property {string} name
   * @property {number} [size]
   * @property {string} [color]
   * @property {number} [strokeWidth]
   * @property {string} [class]
   */

  /** @type {Props} */
  let { name, size = 16, color = 'currentColor', strokeWidth = 1.75, class: className = '' } = $props();

  const iconsMap = /** @type {Record<string, any>} */ (icons);

  /**
   * @param {string} kebab
   * @returns {string}
   */
  function toPascalCase(kebab) {
    return kebab.split('-').map((/** @type {string} */ s) => s[0].toUpperCase() + s.slice(1)).join('');
  }

  let Comp = $derived(iconsMap[toPascalCase(name)]);
</script>

{#if Comp}
  <Comp {size} {color} {strokeWidth} class={className} aria-hidden="true" />
{/if}
