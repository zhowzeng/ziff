// Which lines the reviewer is picking out to comment on, and the draft text for them.
//
// This lives outside the diff panel because it has to be dropped from elsewhere:
// choosing another Repo, Diff Mode or file replaces the diff on screen, and a selection
// names flat line indexes into the diff it was made in — against the next one those
// indexes point at different lines, or at nothing.

interface Range {
  lo: number;
  hi: number;
}

class Selection {
  // The committed range the comment thread is open on, or null when no thread is open.
  range = $state<Range | null>(null);
  // A drag in progress. `start` is wherever the mouse went down, so it can sit above or
  // below `end` — the pair is only put in order once the drag is committed.
  #drag = $state<{ start: number; end: number } | null>(null);
  draft = $state('');

  // Where the drag began, so the caller can check its own "still the same hunk?" rule
  // before extending — which lines are adjacent is the diff's business, not this one's.
  get dragStart(): number | null {
    return this.#drag?.start ?? null;
  }

  startDrag(i: number) {
    this.#drag = { start: i, end: i };
  }

  extendDrag(i: number) {
    if (this.#drag) this.#drag = { ...this.#drag, end: i };
  }

  // Runs on mouseup wherever it lands, so a drag that ran off the diff still commits.
  commitDrag() {
    if (!this.#drag) return;
    const { start, end } = this.#drag;
    this.#drag = null;
    this.openAt(Math.min(start, end), Math.max(start, end));
  }

  openAt(lo: number, hi: number = lo) {
    this.range = { lo, hi };
    this.draft = '';
  }

  // The drag wins while one is in progress, so the highlight grows with the mouse
  // instead of staying on whatever range was open before it started.
  includes(i: number) {
    const drag = this.#drag;
    const range = drag
      ? { lo: Math.min(drag.start, drag.end), hi: Math.max(drag.start, drag.end) }
      : this.range;
    return range !== null && i >= range.lo && i <= range.hi;
  }

  close() {
    this.range = null;
    this.#drag = null;
    this.draft = '';
  }
}

export const selection = new Selection();
