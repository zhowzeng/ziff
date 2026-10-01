<script>
  const sizes = {
    sm: { padding: '4px 10px', fontSize: 'var(--text-xs)', gap: '4px', height: '26px' },
    md: { padding: '5px 12px', fontSize: 'var(--text-sm)', gap: '6px', height: '32px' },
    lg: { padding: '8px 16px', fontSize: 'var(--text-base)', gap: '6px', height: '38px' },
  };
  const variants = {
    primary: { background: 'var(--accent-emphasis)', color: 'var(--accent-fg)', border: '1px solid var(--accent-emphasis)' },
    secondary: { background: 'var(--bg-raised)', color: 'var(--text-primary)', border: '1px solid var(--border-default)' },
    ghost: { background: 'transparent', color: 'var(--text-secondary)', border: '1px solid transparent' },
    danger: { background: 'var(--danger-emphasis)', color: '#fff', border: '1px solid var(--danger-emphasis)' },
  };
  const hoverVariants = {
    primary: { background: 'var(--accent-strong)' },
    secondary: { background: 'var(--bg-subtle)' },
    ghost: { background: 'var(--bg-subtle)' },
    danger: { background: 'var(--red-600)' },
  };

  /**
   * @typedef {Object} Props
   * @property {'primary'|'secondary'|'ghost'|'danger'} [variant]
   * @property {'sm'|'md'|'lg'} [size]
   * @property {boolean} [disabled]
   * @property {(e: MouseEvent) => void} [onclick]
   * @property {string} [style]
   * @property {import('svelte').Snippet} [children]
   */

  /** @type {Props & Record<string, any>} */
  let {
    variant = 'secondary',
    size = 'md',
    disabled = false,
    onclick,
    style = '',
    children,
    ...rest
  } = $props();

  let hover = $state(false);

  /** @param {Record<string, string>} obj */
  function toStyle(obj) {
    return Object.entries(obj).map(([k, v]) => `${k.replace(/[A-Z]/g, (m) => '-' + m.toLowerCase())}:${v}`).join(';');
  }

  let s = $derived(sizes[size] || sizes.md);
  let v = $derived(variants[variant] || variants.secondary);
  let hv = $derived(hover && !disabled ? hoverVariants[variant] : {});

  let computedStyle = $derived(
    `display:inline-flex;align-items:center;justify-content:center;gap:${s.gap};` +
      `font-family:var(--font-sans);font-weight:500;font-size:${s.fontSize};line-height:1;` +
      `padding:${s.padding};height:${s.height};border-radius:var(--radius-md);${variant === 'primary' ? 'box-shadow:var(--shadow-sm),inset 0 1px 0 rgba(255,255,255,.18);' : variant === 'secondary' ? 'box-shadow:var(--shadow-sm);' : ''}` +
      `cursor:${disabled ? 'not-allowed' : 'pointer'};opacity:${disabled ? 0.5 : 1};` +
      `transition:background var(--dur-fast) ease, border-color var(--dur-fast) ease, box-shadow var(--dur-base) var(--ease-out);` +
      `${toStyle(v)};${toStyle(hv)};${style}`
  );
</script>

<button
  {disabled}
  {onclick}
  onmouseenter={() => (hover = true)}
  onmouseleave={() => (hover = false)}
  style={computedStyle}
  {...rest}
>
  {@render children?.()}
</button>
