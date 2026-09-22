<script>
  import { getToasts, dismiss } from '../toast/state.svelte';
  import Icon from './Icon.svelte';
  import IconButton from './IconButton.svelte';
  import Button from './Button.svelte';

  const variants = {
    default: { icon: 'info', color: 'var(--text-secondary)' },
    success: { icon: 'circle-check', color: 'var(--success-fg)' },
    danger: { icon: 'circle-alert', color: 'var(--danger-fg)' },
    warning: { icon: 'triangle-alert', color: 'var(--warning-fg)' },
  };

  let toasts = $derived(getToasts());

  // Taking the action makes the message describing it wrong, so the toast goes with it.
  /** @param {import('../toast/state.svelte').ToastItem} t */
  function runAction(t) {
    t.action?.onclick();
    dismiss(t.id);
  }
</script>

<div style="position:fixed;bottom:16px;right:16px;z-index:100;display:flex;flex-direction:column;gap:8px;width:320px">
  {#each toasts as t (t.id)}
    {@const v = variants[t.variant] || variants.default}
    <div style="display:flex;align-items:flex-start;gap:8px;padding:10px 12px;background:var(--gray-0);border:1px solid var(--border-default);border-radius:var(--radius-md);box-shadow:var(--shadow-md)">
      <Icon name={v.icon} size={16} color={v.color} />
      <div style="flex:1;font-family:var(--font-sans);font-size:var(--text-sm);color:var(--text-primary);padding-top:1px">{t.message}</div>
      {#if t.action}
        <Button size="sm" onclick={() => runAction(t)} style="flex-shrink:0">{t.action.label}</Button>
      {/if}
      <IconButton icon="x" title="Dismiss" size={20} onclick={() => dismiss(t.id)} />
    </div>
  {/each}
</div>
