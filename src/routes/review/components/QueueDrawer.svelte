<script>
  import Icon from '$lib/components/Icon.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import Button from '$lib/components/Button.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import QueueItem from './QueueItem.svelte';
  import { formatForAgent } from '../helpers';

  /**
   * @typedef {Object} QueueItemData
   * @property {string} id
   * @property {string} file
   * @property {number} lineStart
   * @property {number} [lineEnd]
   * @property {string} text
   */

  /**
   * @typedef {Object} Props
   * @property {QueueItemData[]} items
   * @property {(id: string) => void} onRemove
   * @property {() => void} onClose
   */

  /** @type {Props} */
  let { items, onRemove, onClose } = $props();

  /** @param {QueueItemData} item */
  function copyOne(item) {
    navigator.clipboard.writeText(formatForAgent(item));
  }

  function copyAll() {
    navigator.clipboard.writeText(items.map(formatForAgent).join('\n\n'));
  }

  const isMac = /Mac/.test(navigator.platform);
  const shortcutLabel = isMac ? '⌘⇧C' : 'Ctrl+Shift+C';

  $effect(() => {
    /** @param {KeyboardEvent} e */
    function handler(e) {
      const mod = isMac ? e.metaKey : e.ctrlKey;
      if (mod && e.shiftKey && e.key.toLowerCase() === 'c' && items.length > 0) {
        e.preventDefault();
        copyAll();
      }
    }
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  });
</script>

<div style="width:320px;min-width:320px;border-left:1px solid var(--border-default);background:var(--bg-subtle);display:flex;flex-direction:column">
  <div style="height:48px;display:flex;align-items:center;gap:8px;padding:0 12px;border-bottom:1px solid var(--border-default);background:var(--gray-0)">
    <Icon name="terminal" size={14} color="var(--text-secondary)" />
    <span style="font-family:var(--font-sans);font-weight:600;font-size:var(--text-sm);color:var(--text-primary)">Comment Queue</span>
    <Badge variant="neutral">{items.length}</Badge>
    <div style="margin-left:auto">
      <IconButton icon="x" title="Close" size={24} onclick={onClose} />
    </div>
  </div>

  <div style="flex:1;overflow-y:auto;padding:10px;display:flex;flex-direction:column;gap:8px">
    {#if items.length === 0}
      <div style="font-family:var(--font-sans);font-size:var(--text-sm);color:var(--text-tertiary);text-align:center;padding:24px 8px">
        Save a comment to build a queue for your CLI agent.
      </div>
    {:else}
      {#each items as item (item.id)}
        <QueueItem {item} {onRemove} onCopy={copyOne} />
      {/each}
    {/if}
  </div>

  {#if items.length > 0}
    <div style="padding:10px;border-top:1px solid var(--border-default)">
      <Button variant="primary" size="sm" onclick={copyAll} style="width:100%;display:flex;align-items:center;justify-content:center;gap:8px">
        <span>Copy all for agent</span>
        <span style="font-family:var(--font-mono);font-size:11px;opacity:0.75">{shortcutLabel}</span>
      </Button>
    </div>
  {/if}
</div>
