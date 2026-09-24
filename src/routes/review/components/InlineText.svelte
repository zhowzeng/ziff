<script lang="ts">
  import type { InlineSegment } from "../helpers";
  import { paintLine, type SyntaxToken } from "../highlight";

  interface Props {
    text: string;
    /** What changed within the line, when it has a paired line to compare against. */
    segments?: InlineSegment[];
    /** The line's syntax colours, once highlighting has landed. */
    tokens?: SyntaxToken[];
    kind: "context" | "add" | "del";
  }
  let { text, segments, tokens, kind }: Props = $props();

  let pieces = $derived(paintLine(text, tokens, segments));
</script>

<!-- Rendered inside a `white-space: pre` span, so the markup carries no whitespace of
     its own between the pieces of text. -->
{#each pieces as piece, i (i)}<span class={piece.changed ? `changed ${kind}` : undefined} style:color={piece.color}>{piece.text}</span>{/each}

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
