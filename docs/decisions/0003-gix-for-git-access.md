# Rust backend uses gix (gitoxide) for git access

Ziff's [[Repo]] diffing needs are read-heavy and scoped: worktree/index/HEAD/branch diffs with line-level hunks, plus a `Fetch` action to refresh remote refs. No cloning, committing, merging, or pushing is required. We chose `gix` over shelling out to the system `git` CLI or binding to `git2-rs` (libgit2): it covers all three diff shapes (`gix-status` for worktree-vs-index, tree-to-tree diffing for index-vs-HEAD and branch-vs-branch) plus line-level hunks via `gix-diff`/`gix-imara-diff`, without needing to parse CLI text output or take a libgit2 C dependency.

Two accepted trade-offs:
- Rename/copy detection uses gix's first-candidate similarity heuristic, which can occasionally diverge from git CLI's up-to-four-candidate matching. Ziff doesn't need byte-identical output to `git diff`.
- `Fetch` relies on gix's SSH transport shelling out to the system `ssh` binary and its HTTPS auth going through git's credential-helper protocol — so Ziff is not fully dependency-free, but this matches what any GitHub-using developer already has configured.

## Considered Options

- **Shell out to system `git` CLI**: guarantees identical behavior to what the reviewer sees in their terminal (custom diff drivers, `diff.algorithm`, etc.), but means parsing text output and depending on a `git` install.
- **git2-rs (libgit2 bindings)**: structured API, no text parsing, but pulls in a C dependency and its own diff/rename behavior can diverge from git CLI too — without gix's advantage of being pure Rust and under active development.
