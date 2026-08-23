<script>
  let { value, onChange, options } = $props();
</script>

<div class="group" role="group">
  {#each options as o (o.value)}
    <button
      class="seg"
      class:active={value === o.value}
      onclick={() => onChange(o.value)}
    >{o.label}</button>
  {/each}
</div>

<style>
  .group {
    display: inline-flex;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--gray-0);
  }
  .seg {
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    font-weight: 500;
    padding: 5px 12px;
    border: none;
    border-left: 1px solid var(--border-default);
    cursor: pointer;
    background: transparent;
    color: var(--text-secondary);
  }
  .seg:first-child {
    border-left: none;
  }
  .seg:hover {
    background: var(--bg-subtle);
  }
  .seg.active {
    background: var(--accent-subtle);
    color: var(--accent-emphasis);
    font-weight: 600;
  }
</style>
