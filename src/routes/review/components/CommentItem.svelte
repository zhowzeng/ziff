<script>
  import { untrack } from 'svelte';
  import Button from '$lib/components/Button.svelte';
  import Textarea from '$lib/components/Textarea.svelte';
  import Icon from '$lib/components/Icon.svelte';

  /**
   * @typedef {Object} Props
   * @property {string} time
   * @property {string} text
   * @property {boolean} [editable]
   * @property {(text: string) => void} [onEdit]
   */

  /** @type {Props} */
  let { time, text, editable = false, onEdit } = $props();

  let editing = $state(false);
  let draft = $state(untrack(() => text));
  let menuOpen = $state(false);
</script>

<div style="display:flex;gap:8px">
  <div style="flex:1;min-width:0">
    <div style="display:flex;align-items:center;gap:6px">
      <span style="font-family:var(--font-sans);font-size:var(--text-xs);color:var(--text-tertiary)">{time}</span>
      {#if editable}
        <div style="margin-left:auto;position:relative">
          <button
            onclick={() => (menuOpen = !menuOpen)}
            title="More"
            style="width:22px;height:22px;display:flex;align-items:center;justify-content:center;border:none;background:transparent;border-radius:var(--radius-sm);cursor:pointer"
          >
            <Icon name="more-vertical" size={14} color="var(--text-tertiary)" />
          </button>
          {#if menuOpen}
            <div style="position:absolute;top:24px;right:0;background:var(--gray-0);border:1px solid var(--border-default);border-radius:var(--radius-md);box-shadow:var(--shadow-md);z-index:2;min-width:84px;overflow:hidden">
              <button
                onclick={() => {
                  editing = true;
                  menuOpen = false;
                }}
                style="display:block;width:100%;text-align:left;padding:6px 10px;border:none;background:transparent;cursor:pointer;font-family:var(--font-sans);font-size:var(--text-sm);color:var(--text-primary)"
              >Edit</button>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {#if editing}
      <div style="margin-top:4px;display:flex;flex-direction:column;gap:6px">
        <Textarea rows={2} bind:value={draft} />
        <div style="display:flex;gap:6px">
          <Button
            variant="secondary"
            size="sm"
            onclick={() => {
              draft = text;
              editing = false;
            }}
          >Cancel</Button>
          <Button
            variant="primary"
            size="sm"
            onclick={() => {
              onEdit?.(draft);
              editing = false;
            }}
          >Save</Button>
        </div>
      </div>
    {:else}
      <div style="font-family:var(--font-sans);font-size:var(--text-sm);color:var(--text-primary);margin-top:4px;line-height:var(--leading-normal)">
        {text}
      </div>
    {/if}
  </div>
</div>
