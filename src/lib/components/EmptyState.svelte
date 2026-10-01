<script>
  import Icon from './Icon.svelte';

  /** @type {{ icon: import('./Icon.svelte').IconName, title: string, hint?: string, size?: 'md' | 'sm' }} */
  let { icon, title, hint = undefined, size = 'md' } = $props();
</script>

<div class="empty {size}">
  {#if size === 'md'}
    <div class="art" aria-hidden="true">
      <div class="badge" class:spin={icon === 'loader'}><Icon name={icon} size={24} color="var(--accent-emphasis)" /></div>
    </div>
  {:else}
    <span class:spin={icon === 'loader'} class="sm-icon"><Icon name={icon} size={20} color="var(--text-tertiary)" /></span>
  {/if}
  <div class="title">{title}</div>
  {#if hint}
    <div class="hint">{hint}</div>
  {/if}
</div>

<style>
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 24px 16px;
    gap: 8px;
    animation: z-fade-up var(--dur-slow) var(--ease-out) both;
  }
  .title {
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-secondary);
  }
  .hint {
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    max-width: 200px;
    line-height: var(--leading-normal);
  }
  .md {
    gap: 10px;
  }
  .md .title {
    font-size: var(--text-lg);
    font-weight: 650;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin-top: 10px;
  }
  .md .hint {
    font-size: var(--text-sm);
    max-width: 340px;
  }

  .sm-icon {
    display: inline-flex;
  }
  .spin :global(svg) {
    animation: z-spin 0.9s linear infinite;
  }
  .art {
    position: relative;
    width: 64px;
    height: 64px;
    display: grid;
    place-items: center;
  }
  .badge {
    position: relative;
    width: 56px;
    height: 56px;
    display: grid;
    place-items: center;
    border-radius: var(--radius-xl);
    background: var(--bg-raised);
    box-shadow: var(--shadow-sm), inset 0 0 0 1px var(--border-default);
  }
</style>
