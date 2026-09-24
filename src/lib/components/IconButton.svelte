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
  let hover = $state(false);

  let computedStyle = $derived(
    `display:inline-flex;align-items:center;justify-content:center;` +
      `width:${size}px;height:${size}px;border:1px solid transparent;border-radius:var(--radius-sm);` +
      `background:${active ? 'var(--bg-inset)' : hover ? 'var(--bg-subtle)' : 'transparent'};` +
      `color:${active ? 'var(--text-primary)' : 'var(--text-secondary)'};cursor:pointer;`
  );
</script>

<button
  {onclick}
  {title}
  aria-label={title}
  onmouseenter={() => (hover = true)}
  onmouseleave={() => (hover = false)}
  style={computedStyle}
>
  <Icon name={icon} size={15} />
</button>
