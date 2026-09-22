// The hand-off: checking the Comment Queue against the worktree on its way to a CLI
// agent (docs/decisions/0013), and turning what comes back into the text the agent
// reads. Both Copy All and the drawer's per-comment copy go through here, and so does
// Refresh — see resolveAgainstWorktree.

import { resolveAnchor, type AnchorResolution } from './anchor';
import { getFileContent } from './api';
import { commentQueue, type QueueItem } from './comment-queue.svelte';
import type { AgentComment } from './helpers';

// The worktree file, or null when the Comment's code cannot be there to find: a
// deleted or unreadable file rejects, and a binary one has no lines to compare.
async function readWorktree(repoId: string, file: string): Promise<string[] | null> {
  try {
    const content = await getFileContent(repoId, file);
    return content.binary ? null : content.lines;
  } catch {
    return null;
  }
}

// Checks every Comment in `items` against the worktree, writes the result back to the
// Comment Queue — re-anchored line numbers, or Orphaned — and returns the comments in
// hand-off form.
//
// Both hand-off entry points go through this, and so does Refresh: a Comment whose
// lines moved would otherwise keep marking the lines it used to be on. Refresh only
// wants that write-back and ignores the returned comments.
export async function resolveAgainstWorktree(
  repoId: string,
  items: QueueItem[]
): Promise<AgentComment[]> {
  const files = new Map<string, string[] | null>();
  const comments: AgentComment[] = [];
  for (const item of items) {
    if (!files.has(item.file)) files.set(item.file, await readWorktree(repoId, item.file));
    const resolution = resolveAnchor(files.get(item.file) ?? null, item);
    commentQueue.applyResolution(item.id, resolution);
    comments.push(toAgentComment(item, resolution));
  }
  return comments;
}

// An Orphaned Comment keeps the line numbers it was written at — they are what the
// hand-off reports it was commented at, alongside the quote that replaces them.
function toAgentComment(item: QueueItem, resolution: AnchorResolution): AgentComment {
  if (resolution.kind === 'anchored') {
    return {
      file: item.file,
      lineStart: resolution.lineStart,
      lineEnd: resolution.lineEnd,
      text: item.text,
    };
  }
  return {
    file: item.file,
    lineStart: item.lineStart,
    lineEnd: item.lineEnd,
    text: item.text,
    orphaned: { anchorText: item.anchorText, staged: item.diffMode === 'staged' },
  };
}
