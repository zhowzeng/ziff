<script>
  import Icon from '$lib/components/Icon.svelte';

  /**
   * One line of a file shown in File View: worktree content with its own line number
   * and no diff colouring, but the same gutter affordance for leaving a comment.
   *
   * @typedef {Object} Props
   * @property {number} lineNo
   * @property {string} content
   * @property {number} index
   * @property {boolean} [selected]
   * @property {boolean} [commented]
   * @property {(index: number) => void} [onGutterDown]
   * @property {(index: number) => void} [onGutterEnter]
   */

  /** @type {Props} */
  let { lineNo, content, index, selected = false, commented = false, onGutterDown, onGutterEnter } = $props();

  let hover = $state(false);
</script>

<div
  role="presentation"
  onmouseenter={() => {
    hover = true;
    onGutterEnter?.(index);
  }}
  onmouseleave={() => (hover = false)}
  style={`display:flex;background:${selected ? 'var(--accent-subtle)' : 'transparent'};
    border-left:3px solid ${selected ? 'var(--accent-emphasis)' : 'transparent'};
    font-family:var(--font-mono);font-size:var(--diff-font-size);line-height:20px;position:relative`}
>
  {#if (hover || selected) && onGutterDown}
    <button
      onmousedown={(e) => {
        e.preventDefault();
        onGutterDown(index);
      }}
      title="Add comment (drag to select multiple lines)"
      style="position:absolute;left:2px;top:1px;width:16px;height:18px;border-radius:4px;border:none;background:var(--accent-emphasis);color:#fff;font-size:12px;line-height:1;cursor:pointer;z-index:1"
    >+</button>
  {/if}
  <span style="width:72px;text-align:right;color:var(--text-tertiary);user-select:none;padding-right:8px">{lineNo}</span>
  <span
    title={commented ? '這一行已留言' : undefined}
    style="width:16px;flex-shrink:0;display:flex;align-items:center;justify-content:center;user-select:none"
  >
    {#if commented}
      <Icon name="message-square" size={12} color="var(--accent-emphasis)" />
    {/if}
  </span>
  <!-- Stands in for the diff's +/- column so File View content lines up with a diff's. -->
  <span style="width:14px"></span>
  <span style="color:var(--text-primary);white-space:pre">{content}</span>
</div>
