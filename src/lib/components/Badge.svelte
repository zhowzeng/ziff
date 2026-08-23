<script>
  const variants = {
    neutral: 'background:var(--bg-inset);color:var(--text-secondary)',
    accent: 'background:var(--accent-subtle);color:var(--accent-emphasis)',
    success: 'background:var(--success-subtle);color:var(--success-fg)',
    danger: 'background:var(--danger-subtle);color:var(--danger-fg)',
    outline: 'background:transparent;color:var(--text-secondary);border:1px solid var(--border-default)',
  };

  /**
   * @typedef {Object} Props
   * @property {'neutral'|'accent'|'success'|'danger'|'outline'} [variant]
   * @property {import('svelte').Snippet} [children]
   */

  /** @type {Props} */
  let { variant = 'neutral', children } = $props();

  let computedStyle = $derived(
    `display:inline-flex;align-items:center;font-family:var(--font-sans);` +
      `font-size:var(--text-xs);font-weight:500;padding:2px 8px;border-radius:var(--radius-full);` +
      `line-height:1.6;${variants[variant] || variants.neutral}`
  );
</script>

<span style={computedStyle}>
  {@render children?.()}
</span>
