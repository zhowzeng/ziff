<script>
  import Icon from '$lib/components/Icon.svelte';

  let { count = 0, open = false, onclick } = $props();
</script>

<button class="fab" class:open {onclick} title="Comment Queue" aria-label="Comment Queue">
  <Icon name="message-square" size={20} color={open ? 'var(--on-emphasis)' : 'var(--text-secondary)'} />
  {#if count > 0}
    {#key count}<span class="badge">{count}</span>{/key}
  {/if}
</button>

<style>
  .fab {
    position: fixed;
    right: 24px;
    bottom: 24px;
    width: 52px;
    height: 52px;
    padding: 0;
    border-radius: var(--radius-full);
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    box-shadow: var(--shadow-lg);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 20;
    transition: right var(--dur-slow) var(--ease-out), transform var(--dur-base) var(--ease-spring), background var(--dur-base) ease, box-shadow var(--dur-base) ease;
  }
  .fab:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-lg), 0 0 0 4px var(--accent-glow);
  }
  .fab.open {
    right: 364px;
    background: var(--accent-emphasis);
    border-color: transparent;
  }
  .badge {
    position: absolute;
    top: -4px;
    right: -4px;
    min-width: 20px;
    height: 20px;
    padding: 0 5px;
    border-radius: 10px;
    background: var(--danger-emphasis);
    color: #fff;
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 700;
    line-height: 16px;
    text-align: center;
    border: 2px solid var(--bg-canvas);
    box-sizing: border-box;
    animation: z-pop var(--dur-slow) var(--ease-spring);
  }
</style>
