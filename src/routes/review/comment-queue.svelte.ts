// Comment Queue is frontend-only state (docs/decisions/0006) — it isn't sent to
// the backend and clears when the app closes.
//
// It is also the only place a comment's text lives: the diff's comment thread reads
// back out of here rather than keeping its own copy, so an edit in the thread and the
// text handed to the CLI agent can't drift apart.

import type { CommentAnchor } from './helpers';

export interface QueueItem extends CommentAnchor {
  id: string;
  repoId: string;
  file: string;
  text: string;
  createdAt: number;
}

function sameAnchor(a: CommentAnchor, b: CommentAnchor) {
  return (
    a.side === b.side &&
    a.lineStart === b.lineStart &&
    (a.lineEnd ?? a.lineStart) === (b.lineEnd ?? b.lineStart)
  );
}

class CommentQueue {
  // Every Repo's comments share this list, but nothing reads it directly: a queue
  // belongs to one Repo (docs/decisions/0009), so readers go through itemsFor(),
  // find() and lineKeys(), which all take the Repo they're asking about. A comment
  // written in one Repo can then never surface against another — the file paths
  // handed to the CLI agent are repo-relative, so `Cargo.toml:L12` from the wrong
  // Repo would resolve against the current one and be edited silently.
  #items = $state<QueueItem[]>([]);
  open = $state(false);

  // Switching Repo swaps which comments are visible; the ones left behind stay here
  // and come back when the reviewer returns to that Repo.
  itemsFor(repoId: string | null): QueueItem[] {
    if (!repoId) return [];
    return this.#items.filter((i) => i.repoId === repoId);
  }

  add(item: Omit<QueueItem, 'id' | 'createdAt'>) {
    this.#items.push({ id: crypto.randomUUID(), createdAt: Date.now(), ...item });
    this.open = true;
  }

  remove(id: string) {
    this.#items = this.#items.filter((i) => i.id !== id);
  }

  update(id: string, text: string) {
    const item = this.#items.find((i) => i.id === id);
    if (item) item.text = text;
  }

  // The comment already anchored to this exact range, if there is one. Ziff is a
  // solo-review tool with one comment per range (docs/decisions/0002), so reopening a
  // commented range edits that comment instead of starting a second one.
  find(repoId: string | null, file: string, anchor: CommentAnchor): QueueItem | null {
    if (!repoId) return null;
    return (
      this.#items.find((i) => i.repoId === repoId && i.file === file && sameAnchor(i, anchor)) ??
      null
    );
  }

  // Every line of `file` covered by a comment, keyed "<side>:<number>", for marking
  // commented lines in the diff and in File View. Line numbers are unique within each
  // numbering space, so the side prefix is enough to tell the spaces apart.
  //
  // A deleted line swept up in a selection that also touched the new side isn't
  // covered: the anchor counts in new line numbers, which that line has none of.
  lineKeys(repoId: string | null, file: string): Set<string> {
    const keys = new Set<string>();
    if (!repoId) return keys;
    for (const item of this.#items) {
      if (item.repoId !== repoId || item.file !== file) continue;
      for (let n = item.lineStart; n <= (item.lineEnd ?? item.lineStart); n++) {
        keys.add(`${item.side}:${n}`);
      }
    }
    return keys;
  }
}

export const commentQueue = new CommentQueue();
