<script>
  import Icon from '$lib/components/Icon.svelte';
  import FileTreeRow from './FileTreeRow.svelte';

  /** @param {string} name */
  function fileIcon(name) {
    if (name.endsWith('.rs')) return 'cog';
    if (name.endsWith('.toml') || name.endsWith('.lock')) return 'settings-2';
    if (name.endsWith('.md')) return 'file-text';
    if (name.endsWith('.tsx') || name.endsWith('.ts') || name.endsWith('.jsx') || name.endsWith('.js')) return 'file-code';
    return 'file';
  }

  let { node, depth, expanded, onToggle, selected, onSelect } = $props();

  let isDir = $derived(node.type === 'dir');
  let isOpen = $derived(expanded.has(node.path));
  let isSelected = $derived(selected === node.path);

  // j / k can select a file scrolled out of the sidebar; follow it there. `nearest`
  // leaves the list alone when the row is already on screen, as it is after a click.
  /** @param {HTMLElement} el */
  function followSelection(el) {
    if (isSelected) el.scrollIntoView({ block: 'nearest' });
  }
</script>

<div>
  <div
    class="row"
    class:selected={isSelected}
    class:dir={isDir}
    role="button"
    tabindex="0"
    aria-expanded={isDir ? isOpen : undefined}
    aria-current={isSelected ? "true" : undefined}
    {@attach followSelection}
    onclick={() => (isDir ? onToggle(node.path) : onSelect(node.path))}
    onkeydown={(e) => {
      if (e.key === ' ' || e.key === 'Enter') {
        e.preventDefault();
        isDir ? onToggle(node.path) : onSelect(node.path);
      }
    }}
    style:padding-left="{8 + depth * 14}px"
  >
    {#if depth > 0}
      <!-- One hairline per ancestor level, so deep nesting stays readable at a glance. -->
      {#each { length: depth } as _, i (i)}
        <span class="guide" style:left="{8 + i * 14 + 6}px"></span>
      {/each}
    {/if}
    {#if isDir}
      <span class="chev" class:open={isOpen}><Icon name="chevron-right" size={12} color="var(--text-tertiary)" /></span>
    {:else}
      <span class="chev"></span>
    {/if}
    <Icon
      name={isDir ? (isOpen ? 'folder-open' : 'folder') : fileIcon(node.name)}
      size={14}
      color={isSelected ? 'var(--accent-emphasis)' : 'var(--text-tertiary)'}
    />
    <span class="name">{node.name}</span>
    {#if node.renamedFrom}
      <span title={`從 ${node.renamedFrom} 搬過來`} class="renamed">
        <Icon name="corner-up-right" size={12} color="var(--text-tertiary)" />
      </span>
    {/if}
    {#if node.changes}
      <span class="stat">
        <span class="add">+{node.changes.add}</span>
        <span class="del">−{node.changes.del}</span>
      </span>
    {/if}
  </div>
  {#if isDir && isOpen && node.children}
    {#each node.children as child (child.path)}
      <FileTreeRow node={child} depth={depth + 1} {expanded} {onToggle} {selected} {onSelect} />
    {/each}
  {/if}
</div>

<style>
  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    cursor: pointer;
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    transition: background var(--dur-fast) ease;
  }
  .row :global(svg) {
    flex-shrink: 0;
  }
  .row:hover {
    background: var(--bg-inset);
  }
  .row.dir {
    color: var(--text-secondary);
    font-weight: 500;
  }
  .row.selected {
    background: var(--accent-subtle);
    color: var(--accent-emphasis);
    font-weight: 500;
    box-shadow: inset 0 0 0 1px var(--accent-muted-border);
  }
  .row.selected::before {
    content: '';
    position: absolute;
    left: -8px;
    top: 6px;
    bottom: 6px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--accent);
  }
  .guide {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--border-muted);
  }
  .chev {
    width: 13px;
    flex-shrink: 0;
    display: inline-flex;
    transition: transform var(--dur-base) var(--ease-out);
  }
  .chev.open {
    transform: rotate(90deg);
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .renamed {
    display: flex;
    flex-shrink: 0;
  }
  .stat {
    margin-left: auto;
    padding-right: 8px;
    display: flex;
    gap: 5px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }
  .add {
    color: var(--diff-add-text);
  }
  .del {
    color: var(--diff-remove-text);
  }
</style>
