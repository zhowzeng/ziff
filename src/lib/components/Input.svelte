<script>
  let { value = $bindable(''), placeholder = '', size = 'md', style = '', ...rest } = $props();

  let h = $derived(size === 'sm' ? '28px' : '34px');
  let focused = $state(false);

  let computedStyle = $derived(
    `height:${h};padding:0 10px;font-family:var(--font-sans);font-size:var(--text-sm);` +
      `border:1px solid var(--border-default);border-radius:var(--radius-md);background:var(--gray-0);` +
      `color:var(--text-primary);outline:none;width:100%;box-sizing:border-box;` +
      `box-shadow:${focused ? 'var(--focus-ring)' : 'none'};${style}`
  );
</script>

<input
  bind:value
  {placeholder}
  style={computedStyle}
  onfocus={() => (focused = true)}
  onblur={() => (focused = false)}
  {...rest}
/>
