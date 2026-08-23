<script>
  import Icon from './Icon.svelte';
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
</script>

<div>
  <div
    role="button"
    tabindex="0"
    onclick={() => (isDir ? onToggle(node.path) : onSelect(node.path))}
    onkeydown={(e) => {
      if (e.key === ' ' || e.key === 'Enter') {
        e.preventDefault();
        isDir ? onToggle(node.path) : onSelect(node.path);
      }
    }}
    style={`display:flex;align-items:center;gap:4px;height:26px;padding-left:${8 + depth * 14}px;
      font-family:var(--font-sans);font-size:var(--text-sm);cursor:pointer;border-radius:var(--radius-sm);
      background:${isSelected ? 'var(--accent-subtle)' : 'transparent'};
      color:${isSelected ? 'var(--accent-emphasis)' : 'var(--text-primary)'}`}
  >
    {#if isDir}
      <Icon name={isOpen ? 'chevron-down' : 'chevron-right'} size={13} color="var(--text-tertiary)" />
    {:else}
      <span style="width:13px;display:inline-block"></span>
    {/if}
    <Icon name={isDir ? (isOpen ? 'folder-open' : 'folder') : fileIcon(node.name)} size={14} color="var(--text-tertiary)" />
    <span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{node.name}</span>
    {#if node.changes}
      <span style="margin-left:auto;padding-right:8px;font-family:var(--font-mono);font-size:11px">
        <span style="color:var(--diff-add-text)">+{node.changes.add} </span>
        <span style="color:var(--diff-remove-text)">-{node.changes.del}</span>
      </span>
    {/if}
  </div>
  {#if isDir && isOpen && node.children}
    {#each node.children as child (child.path)}
      <FileTreeRow node={child} depth={depth + 1} {expanded} {onToggle} {selected} {onSelect} />
    {/each}
  {/if}
</div>
