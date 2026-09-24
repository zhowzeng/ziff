<script>
  import Icon from '$lib/components/Icon.svelte';
  import InlineText from './InlineText.svelte';

  const kinds = {
    add: { bg: 'var(--diff-add-bg)', bar: 'var(--diff-add-bg-strong)', text: 'var(--diff-add-text)', prefix: '+' },
    del: { bg: 'var(--diff-remove-bg)', bar: 'var(--diff-remove-bg-strong)', text: 'var(--diff-remove-text)', prefix: '-' },
    context: { bg: 'transparent', bar: 'transparent', text: 'var(--text-primary)', prefix: ' ' },
  };

  /**
   * @typedef {{ idx: number, kind?: 'context'|'add'|'del', no?: number|null, text?: string, commentable?: boolean } | null} Side
   */

  /**
   * @typedef {Object} Props
   * @property {Side} left
   * @property {Side} right
   * @property {import('../helpers').InlineSegment[]} [leftSegments]
   * @property {import('../helpers').InlineSegment[]} [rightSegments]
   * @property {boolean} [leftCommented]
   * @property {boolean} [rightCommented]
   * @property {boolean} [leftSelected]
   * @property {boolean} [rightSelected]
   * @property {(index: number) => void} [onGutterDown]
   * @property {(index: number) => void} [onGutterEnter]
   */

  /** @type {Props} */
  let {
    left,
    right,
    leftSegments,
    rightSegments,
    leftCommented = false,
    rightCommented = false,
    leftSelected = false,
    rightSelected = false,
    onGutterDown,
    onGutterEnter,
  } = $props();

  let rightHover = $state(false);

  let kLeft = $derived(kinds[left?.kind || 'context']);
  let kRight = $derived(kinds[right?.kind || 'context']);
</script>

<div style="display:flex;font-family:var(--font-mono);font-size:var(--diff-font-size);line-height:20px">
  <!-- The left column carries the old side only: a drag started there would anchor a
       comment to a line the worktree no longer has (docs/decisions/0010). It still
       highlights when a drag down the new side sweeps past it, same as Unified view. -->
  <div
    style={`flex:1;display:flex;background:${leftSelected ? 'var(--accent-subtle)' : left ? kLeft.bg : 'var(--bg-subtle)'};
      border-left:3px solid ${leftSelected ? 'var(--accent-emphasis)' : left ? kLeft.bar : 'transparent'};min-width:0`}
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
    <span style={`color:${left ? kLeft.text : 'transparent'};white-space:pre;overflow:hidden;text-overflow:ellipsis`}>{#if left}<InlineText text={left.text ?? ''} segments={leftSegments} kind={left.kind || 'context'} />{/if}</span>
  </div>
  <div style="width:1px;background:var(--border-muted);flex-shrink:0"></div>
  <div
    role="presentation"
    onmouseenter={() => {
      rightHover = true;
      if (right) onGutterEnter?.(right.idx);
    }}
    onmouseleave={() => (rightHover = false)}
    style={`flex:1;display:flex;position:relative;background:${rightSelected ? 'var(--accent-subtle)' : right ? kRight.bg : 'var(--bg-subtle)'};
      border-left:3px solid ${rightSelected ? 'var(--accent-emphasis)' : right ? kRight.bar : 'transparent'};min-width:0`}
  >
    {#if right?.commentable && (rightHover || rightSelected) && onGutterDown}
      <button
        onmousedown={(e) => {
          e.preventDefault();
          onGutterDown(right.idx);
        }}
        title="Add comment (drag to select multiple lines)"
        style="position:absolute;left:2px;top:1px;width:16px;height:18px;border-radius:4px;border:none;background:var(--accent-emphasis);color:#fff;font-size:12px;line-height:1;cursor:pointer;z-index:1"
      >+</button>
    {/if}
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
    <span style={`color:${right ? kRight.text : 'transparent'};white-space:pre;overflow:hidden;text-overflow:ellipsis`}>{#if right}<InlineText text={right.text ?? ''} segments={rightSegments} kind={right.kind || 'context'} />{/if}</span>
  </div>
</div>
