<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import { filterFiles, flattenFiles, type PaletteEntry } from "../palette";
  import type { TreeNode } from "../types";

  interface Props {
    open: boolean;
    tree: TreeNode[];
    selectedFile: string | null;
    onSelect: (path: string) => void;
    onClose: () => void;
  }
  let { open, tree, selectedFile, onSelect, onClose }: Props = $props();

  let query = $state("");
  let active = $state(0);
  let input = $state<HTMLInputElement>();
  let list = $state<HTMLElement>();

  let entries = $derived(flattenFiles(tree));
  let results = $derived(filterFiles(entries, query));

  $effect(() => {
    if (!open) return;
    query = "";
    active = Math.max(0, entries.findIndex((e) => e.path === selectedFile));
    const previous = document.activeElement as HTMLElement | null;
    queueMicrotask(() => input?.focus());
    // Back to wherever the reviewer was, unless choosing a file already moved focus.
    return () => previous?.focus?.();
  });
  $effect(() => {
    active;
    list?.querySelector<HTMLElement>('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
  });

  function choose(e: PaletteEntry | undefined) {
    if (!e) return;
    onClose();
    onSelect(e.path);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      active = Math.min(active + 1, results.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      active = Math.max(active - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      choose(results[active]);
    }
  }
</script>

{#if open}
  <div class="scrim" role="presentation" onmousedown={(e) => e.target === e.currentTarget && onClose()}>
    <div class="palette" role="dialog" aria-modal="true" aria-label="Jump to file">
      <div class="search">
        <Icon name="search" size={16} color="var(--text-tertiary)" />
        <input role="combobox" aria-expanded="true" aria-controls="palette-list" aria-activedescendant={results[active] ? `palette-opt-${active}` : undefined} bind:this={input} bind:value={query} oninput={() => (active = 0)} onkeydown={onKey} placeholder="Jump to a changed file…" spellcheck="false" autocomplete="off" />
        <kbd>esc</kbd>
      </div>
      <div class="list" id="palette-list" role="listbox" bind:this={list}>
        {#each results as r, i (r.path)}
          <button class="item" id="palette-opt-{i}" tabindex="-1" role="option" aria-selected={i === active} class:active={i === active} onmousemove={() => (active = i)} onclick={() => choose(r)}>
            <Icon name="file-code" size={15} color={i === active ? "var(--accent-emphasis)" : "var(--text-tertiary)"} />
            <span class="path"><span class="dir">{r.dir}</span><span class="name">{r.name}</span></span>
            {#if r.add !== undefined}
              <span class="stat"><span class="add">+{r.add}</span><span class="del">−{r.del}</span></span>
            {/if}
          </button>
        {:else}
          <div class="none">No file matches “{query}”</div>
        {/each}
      </div>
      <div class="foot">
        <span><kbd>↑</kbd><kbd>↓</kbd> navigate</span>
        <span><kbd>↵</kbd> open</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 90;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 14vh;
    background: var(--bg-scrim);
    backdrop-filter: blur(8px) saturate(1.2);
    -webkit-backdrop-filter: blur(8px) saturate(1.2);
    animation: z-fade-in var(--dur-base) ease-out;
  }
  .palette {
    width: 560px;
    max-width: calc(100vw - 32px);
    border-radius: var(--radius-xl);
    background: var(--bg-raised);
    box-shadow: var(--shadow-lg), 0 0 0 1px var(--border-default);
    overflow: hidden;
    animation: z-pop var(--dur-slow) var(--ease-out);
  }
  .search {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 16px;
    height: 54px;
    border-bottom: 1px solid var(--border-muted);
  }
  input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text-primary);
    font: 500 var(--text-md) var(--font-sans);
  }
  input::placeholder {
    color: var(--text-tertiary);
    font-weight: 400;
  }
  kbd {
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-inset);
    box-shadow: inset 0 0 0 1px var(--border-default);
    font: 500 11px var(--font-mono);
    color: var(--text-secondary);
  }
  .list {
    max-height: 340px;
    overflow-y: auto;
    padding: 6px;
  }
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 38px;
    padding: 0 10px;
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    cursor: pointer;
    text-align: left;
    font: 400 var(--text-sm) var(--font-sans);
  }
  .item.active {
    background: var(--accent-subtle);
    box-shadow: inset 0 0 0 1px var(--accent-muted-border);
  }
  .path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .dir {
    color: var(--text-tertiary);
  }
  .name {
    color: var(--text-primary);
    font-weight: 600;
  }
  .stat {
    display: flex;
    gap: 6px;
    font: 500 11px var(--font-mono);
  }
  .add { color: var(--diff-add-text); }
  .del { color: var(--diff-remove-text); }
  .none {
    padding: 28px 12px;
    text-align: center;
    color: var(--text-tertiary);
    font: 400 var(--text-sm) var(--font-sans);
  }
  .foot {
    display: flex;
    gap: 16px;
    padding: 9px 16px;
    border-top: 1px solid var(--border-muted);
    background: var(--bg-subtle);
    color: var(--text-tertiary);
    font: 400 11px var(--font-sans);
  }
  .foot span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
</style>
