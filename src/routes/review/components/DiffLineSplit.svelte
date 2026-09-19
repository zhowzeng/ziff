<script>
  import Icon from '$lib/components/Icon.svelte';

  const kinds = {
    add: { bg: 'var(--diff-add-bg)', bar: 'var(--diff-add-bg-strong)', text: 'var(--diff-add-text)', prefix: '+' },
    del: { bg: 'var(--diff-remove-bg)', bar: 'var(--diff-remove-bg-strong)', text: 'var(--diff-remove-text)', prefix: '-' },
    context: { bg: 'transparent', bar: 'transparent', text: 'var(--text-primary)', prefix: ' ' },
  };

  /**
   * @typedef {{ kind?: 'context'|'add'|'del', no?: number|null, text?: string, commentable?: boolean } | null} Side
   */

  /**
   * @typedef {Object} Props
   * @property {Side} left
   * @property {Side} right
   * @property {boolean} [leftCommented]
   * @property {boolean} [rightCommented]
   * @property {(e: MouseEvent) => void} [onAddComment]
   */

  /** @type {Props} */
  let { left, right, leftCommented = false, rightCommented = false, onAddComment } = $props();

  let leftHover = $state(false);
  let rightHover = $state(false);

  let kLeft = $derived(kinds[left?.kind || 'context']);
  let kRight = $derived(kinds[right?.kind || 'context']);
</script>

<div style="display:flex;font-family:var(--font-mono);font-size:var(--text-sm);line-height:20px">
  <div
    role="presentation"
    onmouseenter={() => (leftHover = true)}
    onmouseleave={() => (leftHover = false)}
    style={`flex:1;display:flex;background:${left ? kLeft.bg : 'var(--bg-subtle)'};
      border-left:3px solid ${left ? kLeft.bar : 'transparent'};min-width:0`}
  >
    <span style="width:34px;text-align:right;color:var(--text-tertiary);user-select:none;padding-right:6px;flex-shrink:0">{left ? left.no ?? '' : ''}</span>
    <span
      title={left && leftCommented ? '這一行已留言' : undefined}
      style="width:14px;flex-shrink:0;display:flex;align-items:center;justify-content:center;user-select:none"
    >
      {#if left && leftCommented}
        <Icon name="message-square" size={11} color="var(--accent-emphasis)" />
      {/if}
    </span>
    <span style={`width:14px;color:${left ? kLeft.text : 'transparent'};user-select:none;flex-shrink:0`}>{left ? kLeft.prefix : ''}</span>
    <span style={`color:${left ? kLeft.text : 'transparent'};white-space:pre;overflow:hidden;text-overflow:ellipsis`}>{left ? left.text : ''}</span>
    {#if left?.commentable && leftHover}
      <button
        onclick={onAddComment}
        title="Add comment"
        style="margin-left:auto;margin-right:8px;width:18px;height:18px;border-radius:4px;border:none;background:var(--accent-emphasis);color:#fff;font-size:12px;line-height:1;cursor:pointer;flex-shrink:0"
      >+</button>
    {/if}
  </div>
  <div style="width:1px;background:var(--border-muted);flex-shrink:0"></div>
  <div
    role="presentation"
    onmouseenter={() => (rightHover = true)}
    onmouseleave={() => (rightHover = false)}
    style={`flex:1;display:flex;background:${right ? kRight.bg : 'var(--bg-subtle)'};
      border-left:3px solid ${right ? kRight.bar : 'transparent'};min-width:0`}
  >
    <span style="width:34px;text-align:right;color:var(--text-tertiary);user-select:none;padding-right:6px;flex-shrink:0">{right ? right.no ?? '' : ''}</span>
    <span
      title={right && rightCommented ? '這一行已留言' : undefined}
      style="width:14px;flex-shrink:0;display:flex;align-items:center;justify-content:center;user-select:none"
    >
      {#if right && rightCommented}
        <Icon name="message-square" size={11} color="var(--accent-emphasis)" />
      {/if}
    </span>
    <span style={`width:14px;color:${right ? kRight.text : 'transparent'};user-select:none;flex-shrink:0`}>{right ? kRight.prefix : ''}</span>
    <span style={`color:${right ? kRight.text : 'transparent'};white-space:pre;overflow:hidden;text-overflow:ellipsis`}>{right ? right.text : ''}</span>
    {#if right?.commentable && rightHover}
      <button
        onclick={onAddComment}
        title="Add comment"
        style="margin-left:auto;margin-right:8px;width:18px;height:18px;border-radius:4px;border:none;background:var(--accent-emphasis);color:#fff;font-size:12px;line-height:1;cursor:pointer;flex-shrink:0"
      >+</button>
    {/if}
  </div>
</div>
