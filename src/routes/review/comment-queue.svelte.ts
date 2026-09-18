// Comment Queue is frontend-only state (docs/decisions/0006) — it isn't sent to
// the backend and clears when the app closes.

export interface QueueItem {
  id: string;
  file: string;
  lineStart: number;
  lineEnd?: number;
  text: string;
}

class CommentQueue {
  items = $state<QueueItem[]>([]);
  open = $state(false);

  add(item: Omit<QueueItem, 'id'>) {
    this.items.push({ id: crypto.randomUUID(), ...item });
    this.open = true;
  }

  remove(id: string) {
    this.items = this.items.filter((i) => i.id !== id);
  }
}

export const commentQueue = new CommentQueue();
