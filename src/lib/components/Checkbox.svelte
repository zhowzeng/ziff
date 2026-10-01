<script>
  import Icon from './Icon.svelte';

  /**
   * @typedef {Object} Props
   * @property {boolean} [checked]
   * @property {string} [label]
   * @property {boolean} [disabled]
   * @property {(checked: boolean) => void} [onchange]
   */

  /** @type {Props} */
  let { checked = $bindable(false), label, disabled = false, onchange } = $props();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }
</script>

<!-- The click lives on the label so the text toggles the box too; the inner span stays the
     keyboard-reachable control. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_click_events_have_key_events -->
<label
  onclick={toggle}
  style={`display:inline-flex;align-items:center;gap:8px;font-family:var(--font-sans);font-size:var(--text-sm);color:var(--text-primary);
    cursor:${disabled ? 'not-allowed' : 'pointer'};opacity:${disabled ? 0.5 : 1}`}
>
  <span
    role="checkbox"
    aria-checked={checked}
    aria-disabled={disabled}
    tabindex={disabled ? -1 : 0}
    onkeydown={(e) => { if (e.key === ' ' || e.key === 'Enter') { e.preventDefault(); toggle(); } }}
    style={`display:inline-flex;align-items:center;justify-content:center;width:16px;height:16px;border-radius:4px;flex-shrink:0;
      border:1px solid ${checked ? 'var(--accent-emphasis)' : 'var(--border-default)'};background:${checked ? 'var(--accent-emphasis)' : 'var(--bg-raised)'};
      transition:background var(--dur-fast) ease, border-color var(--dur-fast) ease`}
  >
    {#if checked}
      <Icon name="check" size={11} color="var(--accent-fg)" strokeWidth={2.5} />
    {/if}
  </span>
  {#if label}<span>{label}</span>{/if}
</label>
