<script>
  let { value = $bindable(''), placeholder = '', rows = 3, style = '', ...rest } = $props();

  let focused = $state(false);

  let computedStyle = $derived(
    `padding:8px 10px;font-family:var(--font-sans);font-size:var(--text-sm);line-height:1.5;` +
      `border:1px solid var(--border-default);border-radius:var(--radius-md);background:var(--gray-0);` +
      `color:var(--text-primary);outline:none;width:100%;box-sizing:border-box;resize:vertical;` +
      `box-shadow:${focused ? 'var(--focus-ring)' : 'none'};${style}`
  );
</script>

<textarea
  bind:value
  {placeholder}
  {rows}
  style={computedStyle}
  onfocus={() => (focused = true)}
  onblur={() => (focused = false)}
  {...rest}
></textarea>
