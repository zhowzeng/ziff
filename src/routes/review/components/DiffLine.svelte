<script>
  import Icon from '$lib/components/Icon.svelte';

  const kinds = {
    add: { bg: 'var(--diff-add-bg)', bar: 'var(--diff-add-bg-strong)', text: 'var(--diff-add-text)', prefix: '+' },
    del: { bg: 'var(--diff-remove-bg)', bar: 'var(--diff-remove-bg-strong)', text: 'var(--diff-remove-text)', prefix: '-' },
    context: { bg: 'transparent', bar: 'transparent', text: 'var(--text-primary)', prefix: ' ' },
  };

  /**
   * @typedef {Object} Props
   * @property {'context'|'add'|'del'} [kind]
   * @property {number|null} [oldNo]
   * @property {number|null} [newNo]
   * @property {import('svelte').Snippet} [children]
   * @property {boolean} [commentable]
   * @property {(e: MouseEvent) => void} [onAddComment]
   * @property {number} index
   * @property {boolean} [selected]
   * @property {boolean} [commented]
   * @property {(index: number) => void} [onGutterDown]
   * @property {(index: number) => void} [onGutterEnter]
   */

  /** @type {Props} */
  let {
    kind = 'context',
    oldNo,
    newNo,
    children,
    commentable = false,
    onAddComment,
    index,
    selected = false,
    commented = false,
    onGutterDown,
    onGutterEnter,
  } = $props();

  let hover = $state(false);
  let k = $derived(kinds[kind] || kinds.context);
</script>

<div
  role="presentation"
  onmouseenter={() => {
    hover = true;
    onGutterEnter?.(index);
  }}
  onmouseleave={() => (hover = false)}
  style={`display:flex;background:${selected ? 'var(--accent-subtle)' : k.bg};
    border-left:3px solid ${selected ? 'var(--accent-emphasis)' : k.bar};
    font-family:var(--font-mono);font-size:var(--diff-font-size);line-height:20px;position:relative`}
>
  <!-- A del line is gone from the worktree, so `path:L12` for it would name a line the
       CLI agent reads as something else — no affordance there (docs/decisions/0010).
       This matches Split view, where only the new-side column offers it. -->
  {#if (hover || selected) && onGutterDown && newNo !== null}
    <button
      onmousedown={(e) => {
        e.preventDefault();
        onGutterDown(index);
      }}
      title="Add comment (drag to select multiple lines)"
      style="position:absolute;left:2px;top:1px;width:16px;height:18px;border-radius:4px;border:none;background:var(--accent-emphasis);color:#fff;font-size:12px;line-height:1;cursor:pointer;z-index:1"
    >+</button>
  {/if}
  <span style="width:36px;text-align:right;color:var(--text-tertiary);user-select:none;padding-right:6px">{oldNo ?? ''}</span>
  <span style="width:36px;text-align:right;color:var(--text-tertiary);user-select:none;padding-right:8px">{newNo ?? ''}</span>
  <span
    title={commented ? '這一行已留言' : undefined}
    style="width:16px;flex-shrink:0;display:flex;align-items:center;justify-content:center;user-select:none"
  >
    {#if commented}
      <Icon name="message-square" size={12} color="var(--accent-emphasis)" />
    {/if}
  </span>
  <span style={`width:14px;color:${k.text};user-select:none`}>{k.prefix}</span>
  <span style={`color:${k.text};white-space:pre`}>{@render children?.()}</span>
  {#if commentable && hover && !onGutterDown}
    <button
      onclick={onAddComment}
      title="Add comment"
      style="margin-left:auto;margin-right:8px;width:18px;height:18px;border-radius:4px;border:none;background:var(--accent-emphasis);color:#fff;font-size:12px;line-height:1;cursor:pointer"
    >+</button>
  {/if}
</div>
