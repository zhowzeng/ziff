<script lang="ts">
  import type { InlineSegment } from "../helpers";

  interface Props {
    text: string;
    /** What changed within the line, when it has a paired line to compare against. */
    segments?: InlineSegment[];
    kind: "context" | "add" | "del";
  }
  let { text, segments, kind }: Props = $props();
</script>

<!-- Rendered inside a `white-space: pre` span, so the markup carries no whitespace of
     its own between the pieces of text. -->
{#if segments}{#each segments as seg, i (i)}{#if seg.changed}<span class={`changed ${kind}`}>{seg.text}</span>{:else}{seg.text}{/if}{/each}{:else}{text}{/if}

<style>
  .changed {
    border-radius: 2px;
  }

  .changed.add {
    background: var(--diff-add-bg-strong);
  }

  .changed.del {
    background: var(--diff-remove-bg-strong);
  }
</style>
