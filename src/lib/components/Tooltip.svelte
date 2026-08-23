<script>
  let { text, position = 'top', delay = 300, children } = $props();

  let visible = $state(false);
  let timer;

  function show() {
    timer = setTimeout(() => (visible = true), delay);
  }
  function hide() {
    clearTimeout(timer);
    visible = false;
  }

  const positions = {
    top: 'bottom:100%;left:50%;transform:translateX(-50%);margin-bottom:6px',
    bottom: 'top:100%;left:50%;transform:translateX(-50%);margin-top:6px',
    left: 'right:100%;top:50%;transform:translateY(-50%);margin-right:6px',
    right: 'left:100%;top:50%;transform:translateY(-50%);margin-left:6px',
  };
</script>

<span
  style="position:relative;display:inline-flex"
  onmouseenter={show}
  onmouseleave={hide}
  onfocusin={show}
  onfocusout={hide}
>
  {@render children?.()}
  {#if visible && text}
    <span
      role="tooltip"
      style={`position:absolute;${positions[position] || positions.top};white-space:nowrap;pointer-events:none;z-index:90;
        font-family:var(--font-sans);font-size:11px;font-weight:500;color:var(--gray-0);background:var(--gray-900);
        padding:4px 8px;border-radius:var(--radius-sm);box-shadow:var(--shadow-md)`}
    >
      {text}
    </span>
  {/if}
</span>
