//! Resolving a Diff Mode into the two sides being compared, and counting what changed
//! between them (docs/decisions/0003 picks gix for this).

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::types::{Changes, DiffMode, DiffSpec};

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Where one side of a file's content comes from.
///
/// Three of the four combinations matter: `Unstaged` compares an index blob against the
/// file on disk, `Staged` compares two blobs, and either side is `Missing` for a file
/// that was added or deleted.
#[derive(Debug, Clone)]
pub enum Side {
    Missing,
    Blob(gix::ObjectId),
    Worktree(PathBuf),
}

/// One file that differs between the two sides of a Diff Mode.
#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub old: Side,
    pub new: Side,
}

impl Side {
    pub fn read(&self, git: &gix::Repository) -> Result<Vec<u8>, Error> {
        match self {
            Side::Missing => Ok(Vec::new()),
            Side::Blob(id) => Ok(git.find_object(*id)?.detach().data),
            // A file the index still lists but the worktree no longer has reads as
            // empty, which is what a deletion diffs to anyway.
            Side::Worktree(path) => match std::fs::read(path) {
                Ok(bytes) => Ok(bytes),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
                Err(e) => Err(e.into()),
            },
        }
    }
}

/// The files that differ between the two sides `spec` names.
///
/// | Diff Mode  | old side                          | new side       |
/// |------------|-----------------------------------|----------------|
/// | `Unstaged` | index                             | worktree       |
/// | `Staged`   | HEAD                              | index          |
/// | `Branch`   | merge-base(Base Branch, branch)   | branch tip     |
pub fn changed_files(git: &gix::Repository, spec: &DiffSpec) -> Result<Vec<ChangedFile>, Error> {
    let mut files = match spec.diff_mode {
        DiffMode::Unstaged => unstaged(git)?,
        DiffMode::Staged => staged(git)?,
        DiffMode::Branch => branch(git, spec)?,
    };
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files.dedup_by(|a, b| a.path == b.path);
    Ok(files)
}

/// How many lines were added and deleted between the two sides.
///
/// A file neither side can decode as UTF-8 has no lines to count, so it reports zero of
/// each while still being listed as changed.
pub fn count_changes(git: &gix::Repository, file: &ChangedFile) -> Result<Changes, Error> {
    let old = file.old.read(git)?;
    let new = file.new.read(git)?;
    let (Ok(old), Ok(new)) = (std::str::from_utf8(&old), std::str::from_utf8(&new)) else {
        return Ok(Changes { add: 0, del: 0 });
    };
    let input = gix::diff::blob::InternedInput::new(old, new);
    // Slider heuristics move hunk boundaries to where a human would put them, which is
    // also where git puts them.
    let diff =
        gix::diff::blob::diff_with_slider_heuristics(gix::diff::blob::Algorithm::Histogram, &input);
    Ok(Changes {
        add: diff.count_additions(),
        del: diff.count_removals(),
    })
}

/// Every file the checkout tracks, plus anything `extra` adds (untracked files show up
/// that way). Paths are repo-relative and sorted.
pub fn tracked_paths(git: &gix::Repository, extra: &[String]) -> Result<Vec<String>, Error> {
    let index = git.index_or_empty()?;
    let mut paths: BTreeMap<String, ()> = BTreeMap::new();
    for entry in index.entries() {
        paths.insert(entry.path(&index).to_string(), ());
    }
    for path in extra {
        paths.insert(path.clone(), ());
    }
    Ok(paths.into_keys().collect())
}

/// Whether a tree-to-tree change is about file content, rather than a directory or a
/// submodule -- neither of which Ziff can show a diff for.
fn is_file(change: &gix::object::tree::diff::ChangeDetached) -> bool {
    use gix::object::tree::diff::ChangeDetached;
    let mode = match change {
        ChangeDetached::Addition { entry_mode, .. }
        | ChangeDetached::Deletion { entry_mode, .. }
        | ChangeDetached::Modification { entry_mode, .. } => entry_mode,
        ChangeDetached::Rewrite { entry_mode, .. } => entry_mode,
    };
    mode.is_blob_or_symlink()
}

/// Rename detection is off everywhere: `TreeNode::File` has no way to say "this file
/// used to be called something else", so a rename is reported the way
/// `git diff --no-renames` reports it -- the old path deleted, the new path added.
fn no_rewrites() -> gix::diff::Options {
    let mut options = gix::diff::Options::default();
    options.track_rewrites(None);
    options
}

/// index vs worktree. Untracked files count as changes: a file the reviewer just wrote
/// is usually the first thing they want to look at, and `git diff`'s habit of hiding it
/// would make Ziff look broken.
fn unstaged(git: &gix::Repository) -> Result<Vec<ChangedFile>, Error> {
    let root = git
        .workdir()
        .ok_or("This repo has no working tree")?
        .to_owned();
    let iter = git
        .status(gix::progress::Discard)?
        .untracked_files(gix::status::UntrackedFiles::Files)
        .index_worktree_rewrites(None)
        .into_index_worktree_iter(Vec::new())?;

    let mut out = Vec::new();
    for item in iter {
        use gix::status::index_worktree::Item;
        use gix::status::plumbing::index_as_worktree::{Change, EntryStatus};
        match item? {
            Item::Modification {
                entry,
                rela_path,
                status,
                ..
            } => {
                let path = rela_path.to_string();
                let new = match status {
                    // Only the stat cache is stale -- the content is identical.
                    EntryStatus::NeedsUpdate(_) => continue,
                    EntryStatus::Change(Change::Removed) => Side::Missing,
                    _ => Side::Worktree(root.join(&path)),
                };
                let old = match status {
                    EntryStatus::IntentToAdd => Side::Missing,
                    _ => Side::Blob(entry.id),
                };
                out.push(ChangedFile { path, old, new });
            }
            Item::DirectoryContents { entry, .. } => {
                let path = entry.rela_path.to_string();
                out.push(ChangedFile {
                    path: path.clone(),
                    old: Side::Missing,
                    new: Side::Worktree(root.join(&path)),
                });
            }
            // Rewrite tracking is off, so a rename arrives as a deletion and an
            // addition instead.
            Item::Rewrite { .. } => {}
        }
    }
    Ok(out)
}

/// HEAD vs index.
fn staged(git: &gix::Repository) -> Result<Vec<ChangedFile>, Error> {
    let head_tree = git.head_tree_id_or_empty()?;
    let index = git.index_or_empty()?;
    let mut out = Vec::new();
    git.tree_index_status(
        &head_tree,
        &index,
        None,
        gix::status::tree_index::TrackRenames::Disabled,
        |change, _, _| {
            use gix::diff::index::ChangeRef;
            let file = match change {
                ChangeRef::Addition { location, id, .. } => ChangedFile {
                    path: location.to_string(),
                    old: Side::Missing,
                    new: Side::Blob(id.into_owned()),
                },
                ChangeRef::Deletion { location, id, .. } => ChangedFile {
                    path: location.to_string(),
                    old: Side::Blob(id.into_owned()),
                    new: Side::Missing,
                },
                ChangeRef::Modification {
                    location,
                    previous_id,
                    id,
                    ..
                } => ChangedFile {
                    path: location.to_string(),
                    old: Side::Blob(previous_id.into_owned()),
                    new: Side::Blob(id.into_owned()),
                },
                ChangeRef::Rewrite { .. } => return Ok(std::ops::ControlFlow::Continue(())),
            };
            out.push(file);
            Ok::<_, std::convert::Infallible>(std::ops::ControlFlow::Continue(()))
        },
    )?;
    Ok(out)
}

/// merge-base(Base Branch, branch) vs the branch tip -- the commits this branch added,
/// without the ones the Base Branch has moved on to since.
fn branch(git: &gix::Repository, spec: &DiffSpec) -> Result<Vec<ChangedFile>, Error> {
    let base_name = spec
        .base_branch
        .as_deref()
        .ok_or("Branch mode needs a Base Branch to compare against")?;
    // ADR 0010: the branch under review is whichever one is checked out, so its tip is
    // what the reviewer's worktree line numbers line up with.
    let tip = git
        .find_reference(spec.branch.as_str())?
        .into_fully_peeled_id()?;
    let base = git.find_reference(base_name)?.into_fully_peeled_id()?;
    let merge_base = git.merge_base(base, tip)?;

    let old_tree = git.find_commit(merge_base.detach())?.tree()?;
    let new_tree = git.find_commit(tip.detach())?.tree()?;

    let mut out = Vec::new();
    for change in git.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), no_rewrites())? {
        use gix::object::tree::diff::ChangeDetached;
        // A tree-to-tree diff reports the directories along the way too. They have no
        // content to diff, and left in they would show up as files named `src`,
        // colliding with the directory of the same name.
        if !is_file(&change) {
            continue;
        }
        out.push(match change {
            ChangeDetached::Addition { location, id, .. } => ChangedFile {
                path: location.to_string(),
                old: Side::Missing,
                new: Side::Blob(id),
            },
            ChangeDetached::Deletion { location, id, .. } => ChangedFile {
                path: location.to_string(),
                old: Side::Blob(id),
                new: Side::Missing,
            },
            ChangeDetached::Modification {
                location,
                previous_id,
                id,
                ..
            } => ChangedFile {
                path: location.to_string(),
                old: Side::Blob(previous_id),
                new: Side::Blob(id),
            },
            ChangeDetached::Rewrite { .. } => continue,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{changed_files, count_changes, tracked_paths, Side};
    use crate::test_repo::{git, open, repo_with_a_commit, write, DATE};
    use crate::types::{DiffMode, DiffSpec};

    fn spec(diff_mode: DiffMode, base_branch: Option<&str>) -> DiffSpec {
        DiffSpec {
            repo_id: "/unused".into(),
            branch: "main".into(),
            diff_mode,
            base_branch: base_branch.map(str::to_string),
        }
    }

    fn paths_of(git: &gix::Repository, spec: &DiffSpec) -> Vec<String> {
        changed_files(git, spec)
            .expect("changed files")
            .into_iter()
            .map(|file| file.path)
            .collect()
    }

    #[test]
    fn unstaged_lists_a_file_edited_in_the_worktree() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "file.txt", "one\nCHANGED\nthree\n");

        let found = paths_of(&open(dir.path()), &spec(DiffMode::Unstaged, None));
        assert_eq!(found, ["file.txt"]);
    }

    /// `git diff` hides untracked files. A review tool that hid the file the reviewer
    /// just wrote would look broken, so Ziff lists it.
    #[test]
    fn unstaged_lists_an_untracked_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "brand-new.txt", "hello\n");

        let found = paths_of(&open(dir.path()), &spec(DiffMode::Unstaged, None));
        assert_eq!(found, ["brand-new.txt"]);
    }

    #[test]
    fn unstaged_leaves_out_an_ignored_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), ".gitignore", "secret.txt\n");
        git(dir.path(), DATE, &["add", ".gitignore"]);
        git(dir.path(), DATE, &["commit", "-m", "ignore"]);
        write(dir.path(), "secret.txt", "shh\n");

        let found = paths_of(&open(dir.path()), &spec(DiffMode::Unstaged, None));
        assert!(!found.contains(&"secret.txt".to_string()), "got: {found:?}");
    }

    #[test]
    fn unstaged_reports_a_deleted_file_as_missing_on_the_new_side() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        std::fs::remove_file(dir.path().join("file.txt")).expect("remove");

        let files = changed_files(&open(dir.path()), &spec(DiffMode::Unstaged, None)).expect("ok");
        assert!(matches!(files[0].new, Side::Missing));
    }

    #[test]
    fn staged_lists_only_what_was_added_to_the_index() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "staged.txt", "staged\n");
        git(dir.path(), DATE, &["add", "staged.txt"]);
        write(dir.path(), "loose.txt", "not staged\n");

        let found = paths_of(&open(dir.path()), &spec(DiffMode::Staged, None));
        assert_eq!(found, ["staged.txt"]);
    }

    #[test]
    fn staged_leaves_out_an_untracked_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "brand-new.txt", "hello\n");

        let found = paths_of(&open(dir.path()), &spec(DiffMode::Staged, None));
        assert!(found.is_empty(), "got: {found:?}");
    }

    /// Branch mode compares against the merge base, not the Base Branch tip: work the
    /// Base Branch picked up after the fork isn't this branch's to answer for.
    #[test]
    fn branch_ignores_commits_the_base_branch_made_after_the_fork() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        git(dir.path(), DATE, &["checkout", "-b", "feature"]);
        write(dir.path(), "mine.txt", "my work\n");
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "my work"]);

        git(dir.path(), DATE, &["checkout", "main"]);
        write(dir.path(), "theirs.txt", "their work\n");
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "their work"]);
        git(dir.path(), DATE, &["checkout", "feature"]);

        let mut spec = spec(DiffMode::Branch, Some("main"));
        spec.branch = "feature".into();
        let found = paths_of(&open(dir.path()), &spec);
        assert_eq!(found, ["mine.txt"]);
    }

    /// A tree-to-tree diff reports the directories a change sits in as well. Left in,
    /// `src` would arrive as a file and collide with the directory of the same name.
    #[test]
    fn branch_leaves_out_the_directories_a_changed_file_sits_in() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        git(dir.path(), DATE, &["checkout", "-b", "feature"]);
        write(dir.path(), "src/deep/main.rs", "fn main() {}\n");
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "nested"]);

        let mut spec = spec(DiffMode::Branch, Some("main"));
        spec.branch = "feature".into();
        let found = paths_of(&open(dir.path()), &spec);
        assert_eq!(found, ["src/deep/main.rs"]);
    }

    #[test]
    fn counts_added_and_deleted_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        // one / two / three  ->  one / TWO / three / four
        write(dir.path(), "file.txt", "one\nTWO\nthree\nfour\n");

        let git_repo = open(dir.path());
        let files = changed_files(&git_repo, &spec(DiffMode::Unstaged, None)).expect("ok");
        let changes = count_changes(&git_repo, &files[0]).expect("count");
        assert_eq!((changes.add, changes.del), (2, 1));
    }

    #[test]
    fn a_binary_file_has_no_lines_to_count() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        std::fs::write(dir.path().join("logo.png"), [0x89, b'P', b'N', b'G']).expect("write");

        let git_repo = open(dir.path());
        let files = changed_files(&git_repo, &spec(DiffMode::Unstaged, None)).expect("ok");
        let changes = count_changes(&git_repo, &files[0]).expect("count");
        assert_eq!((changes.add, changes.del), (0, 0));
    }

    #[test]
    fn tracked_paths_lists_files_that_did_not_change() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());

        let found = tracked_paths(&open(dir.path()), &[]).expect("tracked");
        assert_eq!(found, ["file.txt"]);
    }

    #[test]
    fn tracked_paths_carries_in_a_path_the_index_no_longer_has() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());

        let found = tracked_paths(&open(dir.path()), &["gone.txt".into()]).expect("tracked");
        assert_eq!(found, ["file.txt", "gone.txt"]);
    }
}
