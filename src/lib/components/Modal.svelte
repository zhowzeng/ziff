<script>
  import IconButton from './IconButton.svelte';

  /**
   * @typedef {Object} Props
   * @property {boolean} [open]
   * @property {() => void} [onClose]
   * @property {string} [title]
   * @property {number} [width]
   * @property {import('svelte').Snippet} [children]
   * @property {import('svelte').Snippet} [footer]
   */

  /** @type {Props} */
  let { open = false, onClose, title, width = 420, children, footer } = $props();

  /** @type {HTMLElement | undefined} */
  let dialog = $state();

  const FOCUSABLE = 'button:not([disabled]), [href], input:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

  // While open: Escape closes, Tab stays inside the dialog, focus moves in on open and
  // goes back to whatever had it when the dialog closes.
  $effect(() => {
    if (!open || !dialog) return;
    const el = dialog;
    const previous = /** @type {HTMLElement | null} */ (document.activeElement);
    /** @param {KeyboardEvent} e */
    function onKey(e) {
      if (e.key === 'Escape') return onClose?.();
      if (e.key !== 'Tab') return;
      const items = /** @type {HTMLElement[]} */ ([...el.querySelectorAll(FOCUSABLE)]);
      if (items.length === 0) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
    (el.querySelector(FOCUSABLE) instanceof HTMLElement ? /** @type {HTMLElement} */ (el.querySelector(FOCUSABLE)) : el).focus();
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('keydown', onKey);
      previous?.focus?.();
    };
  });
</script>

{#if open}
  <div
    role="presentation"
    onmousedown={(e) => { if (e.target === e.currentTarget) onClose?.(); }}
    style="position:fixed;inset:0;background:var(--bg-scrim);backdrop-filter:blur(6px) saturate(1.1);-webkit-backdrop-filter:blur(6px);z-index:80;display:flex;align-items:center;justify-content:center;animation:z-fade-in var(--dur-base) ease-out"
  >
    <div
      bind:this={dialog}
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      style={`width:${width}px;max-width:calc(100vw - 32px);max-height:calc(100vh - 64px);display:flex;flex-direction:column;
        background:var(--bg-raised);border:1px solid var(--border-default);border-radius:var(--radius-xl);box-shadow:var(--shadow-lg);animation:z-pop var(--dur-slow) var(--ease-out)`}
    >
      {#if title}
        <div style="display:flex;align-items:center;gap:8px;padding:14px 16px;border-bottom:1px solid var(--border-default)">
          <span style="flex:1;font-family:var(--font-sans);font-weight:600;font-size:var(--text-md);letter-spacing:-0.01em;color:var(--text-primary)">{title}</span>
          <IconButton icon="x" title="Close" size={24} onclick={onClose} />
        </div>
      {/if}
      <div style="padding:16px;overflow-y:auto">
        {@render children?.()}
      </div>
      {#if footer}
        <div style="display:flex;justify-content:flex-end;gap:8px;padding:12px 16px;border-top:1px solid var(--border-default)">
          {@render footer?.()}
        </div>
      {/if}
    </div>
  </div>
{/if}
