<script>
  const kinds = {
    add: { bg: 'var(--diff-add-bg)', bar: 'var(--diff-add-bg-strong)', text: 'var(--diff-add-text)', prefix: '+' },
    del: { bg: 'var(--diff-remove-bg)', bar: 'var(--diff-remove-bg-strong)', text: 'var(--diff-remove-text)', prefix: '-' },
    context: { bg: 'transparent', bar: 'transparent', text: 'var(--text-primary)', prefix: ' ' },
  };

  let { left, right, onAddComment } = $props();

  let leftHover = $state(false);
  let rightHover = $state(false);
</script>

{#snippet half(side, hover, setHover)}
  {@const k = kinds[side?.kind || 'context']}
  <div
    onmouseenter={() => setHover(true)}
    onmouseleave={() => setHover(false)}
    style={`flex:1;display:flex;background:${side ? k.bg : 'var(--bg-subtle)'};
      border-left:3px solid ${side ? k.bar : 'transparent'};min-width:0`}
  >
    <span style="width:34px;text-align:right;color:var(--text-tertiary);user-select:none;padding-right:6px;flex-shrink:0">{side ? side.no ?? '' : ''}</span>
    <span style={`width:14px;color:${side ? k.text : 'transparent'};user-select:none;flex-shrink:0`}>{side ? k.prefix : ''}</span>
    <span style={`color:${side ? k.text : 'transparent'};white-space:pre;overflow:hidden;text-overflow:ellipsis`}>{side ? side.text : ''}</span>
    {#if side?.commentable && hover}
      <button
        onclick={onAddComment}
        title="Add comment"
        style="margin-left:auto;margin-right:8px;width:18px;height:18px;border-radius:4px;border:none;background:var(--accent-emphasis);color:#fff;font-size:12px;line-height:1;cursor:pointer;flex-shrink:0"
      >+</button>
    {/if}
  </div>
{/snippet}

<div style="display:flex;font-family:var(--font-mono);font-size:var(--text-sm);line-height:20px">
  {@render half(left, leftHover, (v) => (leftHover = v))}
  <div style="width:1px;background:var(--border-muted);flex-shrink:0"></div>
  {@render half(right, rightHover, (v) => (rightHover = v))}
</div>
