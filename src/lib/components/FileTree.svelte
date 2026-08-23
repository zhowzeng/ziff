<script>
  import FileTreeRow from './FileTreeRow.svelte';

  let { tree, selected, onSelect, defaultExpanded } = $props();

  let expanded = $state(
    new Set(defaultExpanded || tree.filter((n) => n.type === 'dir').map((n) => n.path))
  );

  function onToggle(path) {
    const next = new Set(expanded);
    next.has(path) ? next.delete(path) : next.add(path);
    expanded = next;
  }
</script>

<div style="font-family:var(--font-sans)">
  {#each tree as node (node.path)}
    <FileTreeRow {node} depth={0} {expanded} {onToggle} {selected} {onSelect} />
  {/each}
</div>
