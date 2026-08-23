<script>
  import { untrack } from 'svelte';
  import FileTreeRow from './FileTreeRow.svelte';

  /**
   * @typedef {Object} TreeNode
   * @property {string} name
   * @property {string} path
   * @property {'dir'|'file'} type
   * @property {{add: number, del: number}} [changes]
   * @property {TreeNode[]} [children]
   */

  /**
   * @typedef {Object} Props
   * @property {TreeNode[]} tree
   * @property {string} [selected]
   * @property {(path: string) => void} onSelect
   * @property {string[]} [defaultExpanded]
   */

  /** @type {Props} */
  let { tree, selected, onSelect, defaultExpanded } = $props();

  let expanded = $state(
    untrack(() => new Set(defaultExpanded || tree.filter((n) => n.type === 'dir').map((n) => n.path)))
  );

  /** @param {string} path */
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
