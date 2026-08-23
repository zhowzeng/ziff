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
      background:${checked ? 'var(--accent-emphasis)' : 'var(--gray-200)'};transition:background .12s ease`}
  >
    <span
      style={`position:absolute;top:2px;left:${checked ? '16px' : '2px'};width:14px;height:14px;border-radius:50%;
        background:var(--gray-0);box-shadow:var(--shadow-sm);transition:left .12s ease`}
    ></span>
  </span>
  {#if label}<span>{label}</span>{/if}
</label>
