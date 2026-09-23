// The Comment Queue keeps every Repo's comments in one list and relies on each reader
// being handed the Repo it is asking about (docs/decisions/0009). These tests are what
// stops a missing `repoId` comparison from coming back: a comment leaking across Repos
// doesn't break the screen, it hands the CLI agent a path that resolves against the
// wrong worktree.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { CommentAnchor } from './helpers';

type Queue = typeof import('./comment-queue.svelte').commentQueue;

// The queue is a module singleton, so each test gets its own copy of the module.
let queue: Queue;

beforeEach(async () => {
  vi.resetModules();
  ({ commentQueue: queue } = await import('./comment-queue.svelte'));
});

const anchor = (lineStart: number, lineEnd?: number): CommentAnchor => ({
  lineStart,
  lineEnd,
  side: 'new',
});

// Nothing here turns on the Anchor Text or the Diff Mode a comment was written in
// (docs/decisions/0013) — these tests are about which Repo a comment belongs to — so
// every comment carries the same placeholder pair.
const comment = (repoId: string, file: string, text: string, a: CommentAnchor) => ({
  repoId,
  file,
  text,
  anchorText: ['anchor'],
  diffMode: 'unstaged' as const,
  ...a,
});

describe('itemsFor', () => {
  it('returns only the asking Repo, leaving the other Repo untouched', () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));
    queue.add(comment('b', 'Cargo.toml', 'from b', anchor(12)));

    expect(queue.itemsFor('a').map((i) => i.text)).toEqual(['from a']);
    expect(queue.itemsFor('b').map((i) => i.text)).toEqual(['from b']);
  });

  it('is empty with no Repo selected', () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));

    expect(queue.itemsFor(null)).toEqual([]);
  });
});

describe('find', () => {
  it('does not match a same-file, same-range comment from another Repo', () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));

    expect(queue.find('b', 'Cargo.toml', anchor(12))).toBeNull();
    expect(queue.find('a', 'Cargo.toml', anchor(12))?.text).toBe('from a');
  });

  it('finds nothing with no Repo selected', () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));

    expect(queue.find(null, 'Cargo.toml', anchor(12))).toBeNull();
  });

  it('treats an absent lineEnd as a single-line range', () => {
    queue.add(comment('a', 'Cargo.toml', 'one line', anchor(12)));

    expect(queue.find('a', 'Cargo.toml', anchor(12, 12))?.text).toBe('one line');
    expect(queue.find('a', 'Cargo.toml', anchor(12, 13))).toBeNull();
  });

  it('distinguishes the two numbering spaces', () => {
    queue.add(comment('a', 'Cargo.toml', 'diff side', anchor(12)));

    expect(queue.find('a', 'Cargo.toml', { lineStart: 12, side: 'file' })).toBeNull();
  });
});

describe('lineKeys', () => {
  it('does not mark lines a comment in another Repo covers', () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12, 14)));
    queue.add(comment('b', 'Cargo.toml', 'from b', anchor(30)));

    expect([...queue.lineKeys('a', 'Cargo.toml')]).toEqual(['new:12', 'new:13', 'new:14']);
    expect([...queue.lineKeys('b', 'Cargo.toml')]).toEqual(['new:30']);
  });

  it('is empty with no Repo selected', () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));

    expect(queue.lineKeys(null, 'Cargo.toml').size).toBe(0);
  });

  it('marks no lines for an Orphaned comment', () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));
    const [item] = queue.itemsFor('a');

    queue.applyResolution(item.id, { kind: 'orphaned' });

    expect(queue.lineKeys('a', 'Cargo.toml').size).toBe(0);
  });
});

// Copying the queue empties it (docs/decisions/0014), and the toast's undo puts that
// batch back. Both halves are Repo-scoped: a hand-off in one Repo must not take the
// other Repo's comments with it.
describe('takeFor', () => {
  it("takes only the asking Repo's comments, and returns them", () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));
    queue.add(comment('b', 'Cargo.toml', 'from b', anchor(12)));

    const taken = queue.takeFor('a');

    expect(taken.map((i) => i.text)).toEqual(['from a']);
    expect(queue.itemsFor('a')).toEqual([]);
    expect(queue.itemsFor('b').map((i) => i.text)).toEqual(['from b']);
  });

  it('restores the same comments, ids and all', () => {
    queue.add(comment('a', 'Cargo.toml', 'first', anchor(12)));
    queue.add(comment('a', 'Cargo.toml', 'second', anchor(30)));
    const ids = queue.itemsFor('a').map((i) => i.id);

    queue.restore(queue.takeFor('a'));

    expect(queue.itemsFor('a').map((i) => i.id)).toEqual(ids);
    expect(queue.itemsFor('a').map((i) => i.text)).toEqual(['first', 'second']);
    // A restored comment is still the one its range reopens into, not a second copy.
    expect(queue.find('a', 'Cargo.toml', anchor(12))?.id).toBe(ids[0]);
  });

  it('leaves the other Repo where it was when a hand-off is undone', () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));
    queue.add(comment('b', 'Cargo.toml', 'from b', anchor(12)));

    queue.restore(queue.takeFor('a'));

    expect(queue.itemsFor('b').map((i) => i.text)).toEqual(['from b']);
  });
});

describe('removeRepo', () => {
  it("takes only the removed Repo's comments", () => {
    queue.add(comment('a', 'Cargo.toml', 'from a', anchor(12)));
    queue.add(comment('b', 'Cargo.toml', 'from b', anchor(12)));

    queue.removeRepo('a');

    expect(queue.itemsFor('a')).toEqual([]);
    expect(queue.itemsFor('b').map((i) => i.text)).toEqual(['from b']);
  });
});
