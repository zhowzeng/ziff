# Branch mode diffs the worktree, not the branch tip

[[Diff Mode]] `Branch` compares the worktree against the commit where the checked-out [[Branch]] and its [[Base Branch]] last met — so it shows everything done on the Branch, committed or not, down to a file written a minute ago and never added. Its new side used to be the Branch's tip, which meant the mode answered "what has this branch committed" while the reviewer was looking for "what have I done here". Both halves of that are worth fixing, and they are the same fix.

The first half is line numbers. A [[Comment]] is handed to a CLI agent as `path:L12` and the agent resolves it against the worktree (ADR 0007), so a side that is not the worktree produces numbers that mean something else. ADR 0010 closed the large version of this — reviewing a branch you never checked out — and accepted the small one, where the reviewer's own uncommitted edits leave the tip slightly behind the worktree. That residue is now gone rather than tolerated: in `Branch`, as in `Unstaged`, the number on screen is the number the agent will resolve. Only `Staged` still compares something the worktree isn't (the index), and it keeps the mismatch ADR 0010 described.

The second half is what a reviewer came to review. The uncommitted work is the work most likely to need a second pair of eyes, and a tip-only diff hides exactly that. Ziff already answered the same question once the same way: `Unstaged` lists untracked files although `git diff` does not, because a review tool that hid the file the reviewer just wrote would look broken. A Branch-mode diff that hid it for the same reason was the last place that argument had not reached.

`gix` supports this directly: its status platform takes `head_tree()` — documented for comparing "a tree that it possibly didn't originate from" — so setting it to the merge-base walks merge-base → index → worktree in one pass. **What that pass does not do is combine its findings.** It reports tree-to-index and index-to-worktree changes as separate items, in no guaranteed order, so a file changed both in a commit on the Branch and again in the worktree arrives twice and has to be composed into one change — old side from the merge-base, new side from the worktree. This is the part that fails quietly: keeping only the first of the two items leaves a file whose old or new side belongs to the wrong comparison, with a diff that still renders. Composition by path, and a test for a file changed on both sides of the index, are not optional extras here.

## Consequences

- Ziff no longer offers "what this branch contains as committed" — the PR-shaped diff. That comparison has no worktree behind it, so no Comment written against it could anchor (ADR 0010), which would make it a browse-only mode.
- `Unstaged` becomes a narrower view of the same worktree rather than a different kind of comparison. It stays because "what have I not staged yet" is a question worth asking on its own.

## Considered Options

- **Keep the tip as the new side**: rejected — it hides uncommitted work, which is the work most in need of review, and leaves line numbers that drift from the worktree the agent edits.
- **A fourth Diff Mode, so both comparisons exist**: rejected — a tip-vs-base mode produces line numbers no Comment can anchor to, so it would be browse-only, and a fourth mode is a fourth thing to explain for a comparison `git diff` already gives from a terminal.
- **Compose it by hand from `tree_index_status` and the index-worktree status**: rejected — the items still have to be composed either way, but two calls means a second traversal and two index reads that can disagree, where `head_tree()` gives one pass over one index.
