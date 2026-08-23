<script>
  import Button from '$lib/components/Button.svelte';
  import Textarea from '$lib/components/Textarea.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import CommentItem from './CommentItem.svelte';

  let {
    file,
    lineStart,
    lineEnd,
    comments,
    replyValue = $bindable(''),
    onComment,
    onClose,
  } = $props();

  let range = $derived(lineEnd && lineEnd !== lineStart ? `L${lineStart}–L${lineEnd}` : `L${lineStart}`);
</script>

<div style="border:1px solid var(--border-default);border-radius:var(--radius-lg);background:var(--gray-0);box-shadow:var(--shadow-md);overflow:hidden;max-width:560px;position:relative">
  {#if onClose}
    <button
      onclick={onClose}
      title="Close"
      style="position:absolute;top:8px;right:8px;width:22px;height:22px;display:flex;align-items:center;justify-content:center;border:none;background:transparent;border-radius:var(--radius-sm);cursor:pointer;z-index:1"
    >
      <Icon name="x" size={14} color="var(--text-tertiary)" />
    </button>
  {/if}

  {#if file}
    <div style="display:flex;align-items:center;gap:6px;padding:8px 16px;background:var(--bg-subtle);border-bottom:1px solid var(--border-muted);font-family:var(--font-mono);font-size:var(--text-xs);color:var(--text-secondary);min-width:0">
      <span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap;min-width:0">{file}</span>
      <span style="color:var(--text-tertiary);flex-shrink:0">{range}</span>
    </div>
  {/if}

  {#if comments.length > 0}
    <div style="padding:14px 16px;display:flex;flex-direction:column;gap:14px">
      {#each comments.slice(0, 1) as c, i (i)}
        <CommentItem
          time={c.time}
          text={c.text}
          editable={c.editable}
          onEdit={(text) => c.onEdit?.(text)}
        />
      {/each}
    </div>
  {:else}
    <div style="padding:12px;display:flex;flex-direction:column;gap:10px">
      <Textarea placeholder="Write a comment" rows={2} bind:value={replyValue} />
      <div style="display:flex;justify-content:flex-end;align-items:center">
        <div style="display:flex;gap:8px">
          <Button variant="primary" size="sm" onclick={onComment} disabled={!replyValue || !replyValue.trim()}>
            Add comment
          </Button>
        </div>
      </div>
    </div>
  {/if}
</div>
