<script>
  /**
   * @typedef {Object} Props
   * @property {boolean} [checked]
   * @property {string} [label]
   * @property {boolean} [disabled]
   * @property {(checked: boolean) => void} [onchange]
   */

  /** @type {Props} */
  let { checked = $bindable(false), label, disabled = false, onchange } = $props();

  let switchId = $props.id();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }
</script>

<label
  for={switchId}
  style={`display:inline-flex;align-items:center;gap:8px;font-family:var(--font-sans);font-size:var(--text-sm);color:var(--text-primary);
    cursor:${disabled ? 'not-allowed' : 'pointer'};opacity:${disabled ? 0.5 : 1}`}
>
  <span
    id={switchId}
    role="switch"
    aria-checked={checked}
    aria-disabled={disabled}
    tabindex={disabled ? -1 : 0}
    onclick={toggle}
    onkeydown={(e) => { if (e.key === ' ' || e.key === 'Enter') { e.preventDefault(); toggle(); } }}
    style={`position:relative;display:inline-flex;align-items:center;width:32px;height:18px;border-radius:var(--radius-full);flex-shrink:0;
      background:${checked ? 'var(--accent-emphasis)' : 'var(--bg-inset)'};transition:background var(--dur-base) var(--ease-out);box-shadow:inset 0 0 0 1px ${checked ? 'transparent' : 'var(--border-strong)'}`}
  >
    <span
      style={`position:absolute;top:2px;left:${checked ? '16px' : '2px'};width:14px;height:14px;border-radius:50%;
        background:${checked ? 'var(--on-emphasis)' : 'var(--text-tertiary)'};box-shadow:var(--shadow-sm);transition:left var(--dur-base) var(--ease-spring),background var(--dur-base) ease`}
    ></span>
  </span>
  {#if label}<span>{label}</span>{/if}
</label>
