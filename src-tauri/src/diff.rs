//! Resolving a Diff Mode into the two sides being compared, and working out what
//! changed between them: the counts per file, and the line-level hunks for one
//! (docs/decisions/0003 picks gix for this).

use std::collections::BTreeMap;
use std::path::PathBuf;

use gix::diff::blob::{Algorithm, Diff, Hunk, InternedInput, Token};

use crate::types::{Changes, DiffHunk, DiffLine, DiffMode, DiffSpec, FileDiff, LineKind};

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Unchanged lines shown either side of a change. git's default; Settings has no place
/// to put the choice yet, so it is not configurable.
const CONTEXT_LINES: u32 = 3;

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
    /// Where this file used to be, when it was renamed into `path`.
    pub old_path: Option<String>,
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

/// The line-level diff of one file, as hunks the frontend renders directly.
///
/// The numbers are the point of this: `old_no` / `new_no` are what a Comment anchors to,
/// and a del line carries no `new_no` at all -- the worktree does not have that line, so
/// there is nothing for `path:L12` to resolve against (ADR 0010).
pub fn file_diff(git: &gix::Repository, file: &ChangedFile) -> Result<FileDiff, Error> {
    let old = file.old.read(git)?;
    let new = file.new.read(git)?;
    let (Ok(old), Ok(new)) = (std::str::from_utf8(&old), std::str::from_utf8(&new)) else {
        return Ok(FileDiff {
            hunks: Vec::new(),
            binary: true,
        });
    };
    let input = InternedInput::new(old, new);
    // Same algorithm and heuristics as `count_changes`, so the hunks shown and the
    // +/- counts in the sidebar can't tell different stories about the same file.
    let diff = gix::diff::blob::diff_with_slider_heuristics(Algorithm::Histogram, &input);
    Ok(FileDiff {
        hunks: hunks(&input, &diff),
        binary: false,
    })
}

/// Groups the changed ranges into hunks the way git does: two changes whose context
/// lines would touch are one hunk, not two with a gap of repeated lines between them.
fn hunks(input: &InternedInput<&str>, diff: &Diff) -> Vec<DiffHunk> {
    let mut out = Vec::new();
    let mut group: Vec<Hunk> = Vec::new();
    for change in diff.hunks() {
        if let Some(last) = group.last() {
            if change.before.start - last.before.end > 2 * CONTEXT_LINES {
                out.push(build_hunk(input, &group));
                group.clear();
            }
        }
        group.push(change);
    }
    if !group.is_empty() {
        out.push(build_hunk(input, &group));
    }
    out
}

/// One hunk: the changes in `group`, the unchanged lines between them, and up to
/// `CONTEXT_LINES` of context either end.
fn build_hunk(input: &InternedInput<&str>, group: &[Hunk]) -> DiffHunk {
    let (first, last) = (&group[0], &group[group.len() - 1]);
    let old_start = first.before.start.saturating_sub(CONTEXT_LINES);
    let old_end = (last.before.end + CONTEXT_LINES).min(input.before.len() as u32);
    // The runs between changes are identical on both sides, so the leading context this
    // clamps off is the same length on both -- the two sides stay in step.
    let new_start = first.after.start.saturating_sub(CONTEXT_LINES);
    let new_end = (last.after.end + CONTEXT_LINES).min(input.after.len() as u32);

    let mut lines = Vec::new();
    let (mut old_no, mut new_no) = (old_start, new_start);
    for change in group {
        push_context(
            input,
            &mut lines,
            &mut old_no,
            &mut new_no,
            change.before.start,
        );
        for i in change.before.clone() {
            lines.push(DiffLine {
                kind: LineKind::Del,
                old_no: Some(i + 1),
                // ADR 0010: the worktree has no such line, so there is no new-side
                // number -- which is what withholds the comment affordance.
                new_no: None,
                content: text(input, &input.before, i),
            });
        }
        for i in change.after.clone() {
            lines.push(DiffLine {
                kind: LineKind::Add,
                old_no: None,
                new_no: Some(i + 1),
                content: text(input, &input.after, i),
            });
        }
        old_no = change.before.end;
        new_no = change.after.end;
    }
    push_context(input, &mut lines, &mut old_no, &mut new_no, old_end);

    DiffHunk {
        header: format!(
            "@@ -{} +{} @@",
            range(old_start, old_end - old_start),
            range(new_start, new_end - new_start)
        ),
        lines,
    }
}

/// Unchanged lines up to the old-side index `to`, advancing both sides in step.
fn push_context(
    input: &InternedInput<&str>,
    lines: &mut Vec<DiffLine>,
    old_no: &mut u32,
    new_no: &mut u32,
    to: u32,
) {
    while *old_no < to {
        lines.push(DiffLine {
            kind: LineKind::Context,
            old_no: Some(*old_no + 1),
            new_no: Some(*new_no + 1),
            content: text(input, &input.before, *old_no),
        });
        *old_no += 1;
        *new_no += 1;
    }
}

/// One side of a `@@ -a,b +c,d @@` header, spelled the way git spells it: an empty range
/// names the line it would follow, and a range of exactly one line leaves out the count.
fn range(start: u32, count: u32) -> String {
    let first = if count == 0 { start } else { start + 1 };
    if count == 1 {
        first.to_string()
    } else {
        format!("{first},{count}")
    }
}

/// The text of one line, without the newline that ended it. A file whose last line has
/// no newline is not marked as such: git's `\ No newline at end of file` has no line
/// number, so there is nothing for a Comment to anchor to either way.
fn text(input: &InternedInput<&str>, tokens: &[Token], i: u32) -> String {
    let line = input.interner[tokens[i as usize]];
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line).to_string()
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

/// Rename detection, set explicitly rather than read from the reviewer's git config, so
/// the same Repo produces the same tree on any machine. Copies are not tracked (gix's
/// default), so a `Rewrite` is always a rename.
fn rewrites() -> gix::diff::Rewrites {
    gix::diff::Rewrites::default()
}

fn tree_options() -> gix::diff::Options {
    let mut options = gix::diff::Options::default();
    options.track_rewrites(Some(rewrites()));
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
        .index_worktree_rewrites(Some(rewrites()))
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
                out.push(ChangedFile {
                    path,
                    old_path: None,
                    old,
                    new,
                });
            }
            Item::DirectoryContents { entry, .. } => {
                let path = entry.rela_path.to_string();
                out.push(ChangedFile {
                    path: path.clone(),
                    old_path: None,
                    old: Side::Missing,
                    new: Side::Worktree(root.join(&path)),
                });
            }
            Item::Rewrite {
                source,
                dirwalk_entry,
                ..
            } => {
                use gix::status::index_worktree::RewriteSource;
                // The other variant is only ever produced for copies, which `rewrites()`
                // does not track.
                let RewriteSource::RewriteFromIndex {
                    source_entry,
                    source_rela_path,
                    ..
                } = source
                else {
                    continue;
                };
                let path = dirwalk_entry.rela_path.to_string();
                out.push(ChangedFile {
                    old: Side::Blob(source_entry.id),
                    new: Side::Worktree(root.join(&path)),
                    old_path: Some(source_rela_path.to_string()),
                    path,
                });
            }
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
        gix::status::tree_index::TrackRenames::Given(rewrites()),
        |change, _, _| {
            use gix::diff::index::ChangeRef;
            let file = match change {
                ChangeRef::Addition { location, id, .. } => ChangedFile {
                    path: location.to_string(),
                    old_path: None,
                    old: Side::Missing,
                    new: Side::Blob(id.into_owned()),
                },
                ChangeRef::Deletion { location, id, .. } => ChangedFile {
                    path: location.to_string(),
                    old_path: None,
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
                    old_path: None,
                    old: Side::Blob(previous_id.into_owned()),
                    new: Side::Blob(id.into_owned()),
                },
                ChangeRef::Rewrite {
                    source_location,
                    source_id,
                    location,
                    id,
                    ..
                } => ChangedFile {
                    path: location.to_string(),
                    old_path: Some(source_location.to_string()),
                    old: Side::Blob(source_id.into_owned()),
                    new: Side::Blob(id.into_owned()),
                },
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
    for change in git.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), tree_options())? {
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
                old_path: None,
                old: Side::Missing,
                new: Side::Blob(id),
            },
            ChangeDetached::Deletion { location, id, .. } => ChangedFile {
                path: location.to_string(),
                old_path: None,
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
                old_path: None,
                old: Side::Blob(previous_id),
                new: Side::Blob(id),
            },
            ChangeDetached::Rewrite {
                source_location,
                source_id,
                location,
                id,
                ..
            } => ChangedFile {
                path: location.to_string(),
                old_path: Some(source_location.to_string()),
                old: Side::Blob(source_id),
                new: Side::Blob(id),
            },
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{changed_files, count_changes, file_diff, tracked_paths, Side};
    use crate::test_repo::{git, git_stdout, open, repo_with_a_commit, write, DATE};
    use crate::types::{DiffHunk, DiffMode, DiffSpec, LineKind};

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

    /// Big enough that similarity detection has something to work with.
    fn commit_a_big_file(dir: &std::path::Path, name: &str) {
        let body: String = (1..=200).map(|i| format!("line {i}\n")).collect();
        write(dir, name, &body);
        git(dir, DATE, &["add", "."]);
        git(dir, DATE, &["commit", "-m", "big"]);
    }

    #[test]
    fn unstaged_reports_a_moved_file_once_under_its_new_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        commit_a_big_file(dir.path(), "big.txt");
        std::fs::rename(dir.path().join("big.txt"), dir.path().join("moved.txt")).expect("mv");

        let found = paths_of(&open(dir.path()), &spec(DiffMode::Unstaged, None));
        assert_eq!(found, ["moved.txt"]);
    }

    #[test]
    fn unstaged_says_where_a_moved_file_came_from() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        commit_a_big_file(dir.path(), "big.txt");
        std::fs::rename(dir.path().join("big.txt"), dir.path().join("moved.txt")).expect("mv");

        let files = changed_files(&open(dir.path()), &spec(DiffMode::Unstaged, None)).expect("ok");
        assert_eq!(files[0].old_path.as_deref(), Some("big.txt"));
    }

    #[test]
    fn staged_says_where_a_moved_file_came_from() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        commit_a_big_file(dir.path(), "big.txt");
        git(dir.path(), DATE, &["mv", "big.txt", "moved.txt"]);

        let files = changed_files(&open(dir.path()), &spec(DiffMode::Staged, None)).expect("ok");
        assert_eq!(
            (files[0].path.as_str(), files[0].old_path.as_deref()),
            ("moved.txt", Some("big.txt"))
        );
    }

    #[test]
    fn branch_says_where_a_moved_file_came_from() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        commit_a_big_file(dir.path(), "big.txt");
        git(dir.path(), DATE, &["checkout", "-b", "feature"]);
        git(dir.path(), DATE, &["mv", "big.txt", "moved.txt"]);
        git(dir.path(), DATE, &["commit", "-m", "rename"]);

        let mut spec = spec(DiffMode::Branch, Some("main"));
        spec.branch = "feature".into();
        let files = changed_files(&open(dir.path()), &spec).expect("ok");
        assert_eq!(
            (files[0].path.as_str(), files[0].old_path.as_deref()),
            ("moved.txt", Some("big.txt"))
        );
    }

    /// A move with edits on top still diffs against the file's own old content, so the
    /// counts are the edit -- not the whole file twice over.
    #[test]
    fn a_move_with_edits_counts_only_the_edits() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        commit_a_big_file(dir.path(), "big.txt");
        git(dir.path(), DATE, &["mv", "big.txt", "moved.txt"]);
        let edited: String = (1..=200)
            .map(|i| {
                if i == 5 {
                    "EDITED\n".to_string()
                } else {
                    format!("line {i}\n")
                }
            })
            .collect();
        write(dir.path(), "moved.txt", &edited);
        git(dir.path(), DATE, &["add", "-A"]);

        let git_repo = open(dir.path());
        let files = changed_files(&git_repo, &spec(DiffMode::Staged, None)).expect("ok");
        let changes = count_changes(&git_repo, &files[0]).expect("count");
        assert_eq!((changes.add, changes.del), (1, 1));
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

    fn hunks_of(dir: &std::path::Path, spec: &DiffSpec, path: &str) -> Vec<DiffHunk> {
        let git_repo = open(dir);
        let files = changed_files(&git_repo, spec).expect("changed files");
        let file = files
            .iter()
            .find(|file| file.path == path)
            .unwrap_or_else(|| panic!("{path} is not in the diff"));
        file_diff(&git_repo, file).expect("file diff").hunks
    }

    /// Ziff's hunks written back out as unified diff text, so they can be held up
    /// against what `git diff` prints.
    fn as_unified(hunks: &[DiffHunk]) -> String {
        let mut out = String::new();
        for hunk in hunks {
            out.push_str(&hunk.header);
            out.push('\n');
            for line in &hunk.lines {
                out.push(match line.kind {
                    LineKind::Context => ' ',
                    LineKind::Add => '+',
                    LineKind::Del => '-',
                });
                out.push_str(&line.content);
                out.push('\n');
            }
        }
        out
    }

    /// `git diff` from the first `@@` on, with the function context git appends to a
    /// hunk header dropped -- Ziff's headers are the `@@ -a,b +c,d @@` part alone.
    fn git_diff_hunks(dir: &std::path::Path, args: &[&str]) -> String {
        let mut all = vec!["-c", "diff.algorithm=histogram", "diff"];
        all.extend_from_slice(args);
        let printed = git_stdout(dir, DATE, &all);
        printed
            .lines()
            .skip_while(|line| !line.starts_with("@@"))
            .map(|line| match line.starts_with("@@") {
                true => &line[..line.match_indices("@@").nth(1).expect("closing @@").0 + 2],
                false => line,
            })
            .fold(String::new(), |mut acc, line| {
                acc.push_str(line);
                acc.push('\n');
                acc
            })
    }

    /// The whole point of this module: a hunk Ziff shows has to name the same lines
    /// `git diff` does, or the `path:L12` handed to a CLI agent points somewhere else.
    #[test]
    fn hunks_match_what_git_diff_prints() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        let before: String = (1..=40).map(|i| format!("line {i}\n")).collect();
        write(dir.path(), "big.txt", &before);
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "big"]);
        // Two edits far enough apart to stay separate hunks, plus one at the very top
        // where there is no room for three lines of leading context.
        let after: String = (1..=40)
            .map(|i| match i {
                2 => "EDITED 2\n".to_string(),
                20 => "EDITED 20\n".to_string(),
                _ => format!("line {i}\n"),
            })
            .collect();
        write(dir.path(), "big.txt", &after);

        let hunks = hunks_of(dir.path(), &spec(DiffMode::Unstaged, None), "big.txt");
        assert_eq!(
            as_unified(&hunks),
            git_diff_hunks(dir.path(), &["--", "big.txt"])
        );
    }

    #[test]
    fn a_new_file_is_one_hunk_starting_at_line_one() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "new.txt", "alpha\nbeta\n");

        let hunks = hunks_of(dir.path(), &spec(DiffMode::Unstaged, None), "new.txt");
        assert_eq!(hunks[0].header, "@@ -0,0 +1,2 @@");
        assert_eq!(
            hunks[0]
                .lines
                .iter()
                .map(|line| (line.old_no, line.new_no))
                .collect::<Vec<_>>(),
            [(None, Some(1)), (None, Some(2))]
        );
    }

    /// ADR 0010 hangs the add-comment affordance on a del line having no new-side
    /// number: the worktree does not have that line for `path:L12` to resolve against.
    #[test]
    fn a_deleted_line_carries_no_new_side_number() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "file.txt", "one\nthree\n");

        let hunks = hunks_of(dir.path(), &spec(DiffMode::Unstaged, None), "file.txt");
        let del = hunks[0]
            .lines
            .iter()
            .find(|line| matches!(line.kind, LineKind::Del))
            .expect("a del line");
        assert_eq!(
            (del.old_no, del.new_no, del.content.as_str()),
            (Some(2), None, "two")
        );
    }

    /// Context lines are numbered on both sides, and the two numbering spaces drift
    /// apart by exactly what the changes above them added or removed.
    #[test]
    fn context_lines_are_numbered_on_both_sides() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "file.txt", "zero\none\ntwo\nthree\n");

        let hunks = hunks_of(dir.path(), &spec(DiffMode::Unstaged, None), "file.txt");
        let numbers: Vec<_> = hunks[0]
            .lines
            .iter()
            .map(|line| (line.old_no, line.new_no))
            .collect();
        assert_eq!(
            numbers,
            [
                (None, Some(1)),
                (Some(1), Some(2)),
                (Some(2), Some(3)),
                (Some(3), Some(4))
            ]
        );
    }

    /// git merges two changes whose context lines touch into one hunk rather than
    /// printing the same lines twice.
    #[test]
    fn changes_closer_than_twice_the_context_are_one_hunk() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        let before: String = (1..=30).map(|i| format!("line {i}\n")).collect();
        write(dir.path(), "big.txt", &before);
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "big"]);
        // Six unchanged lines between the two edits: exactly the two context runs.
        let after: String = (1..=30)
            .map(|i| match i {
                10 | 17 => format!("EDITED {i}\n"),
                _ => format!("line {i}\n"),
            })
            .collect();
        write(dir.path(), "big.txt", &after);

        let hunks = hunks_of(dir.path(), &spec(DiffMode::Unstaged, None), "big.txt");
        assert_eq!(
            hunks.len(),
            1,
            "got: {:?}",
            hunks.iter().map(|h| &h.header).collect::<Vec<_>>()
        );
        assert_eq!(hunks[0].header, "@@ -7,14 +7,14 @@");
    }

    /// A single-line range has no count in a git hunk header.
    #[test]
    fn a_one_line_side_leaves_the_count_out_of_the_header() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "one-liner.txt", "only\n");
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "one-liner"]);
        write(dir.path(), "one-liner.txt", "CHANGED\n");

        let hunks = hunks_of(dir.path(), &spec(DiffMode::Unstaged, None), "one-liner.txt");
        assert_eq!(hunks[0].header, "@@ -1 +1 @@");
    }

    /// A binary file has no lines to diff, and saying so is not the same as saying the
    /// file did not change.
    #[test]
    fn a_binary_file_says_so_rather_than_showing_an_empty_diff() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        std::fs::write(dir.path().join("logo.png"), [0x89, b'P', b'N', b'G']).expect("write");

        let git_repo = open(dir.path());
        let files = changed_files(&git_repo, &spec(DiffMode::Unstaged, None)).expect("ok");
        let diff = file_diff(&git_repo, &files[0]).expect("file diff");
        assert!(diff.binary);
        assert!(diff.hunks.is_empty());
    }

    /// A staged rename with edits on top diffs the file against its own old content,
    /// under the path the reviewer clicked -- the new one.
    #[test]
    fn a_renamed_file_diffs_against_where_it_came_from() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        commit_a_big_file(dir.path(), "big.txt");
        git(dir.path(), DATE, &["mv", "big.txt", "moved.txt"]);
        let edited: String = (1..=200)
            .map(|i| match i {
                5 => "EDITED\n".to_string(),
                _ => format!("line {i}\n"),
            })
            .collect();
        write(dir.path(), "moved.txt", &edited);
        git(dir.path(), DATE, &["add", "-A"]);

        let hunks = hunks_of(dir.path(), &spec(DiffMode::Staged, None), "moved.txt");
        assert_eq!(as_unified(&hunks), "@@ -2,7 +2,7 @@\n line 2\n line 3\n line 4\n-line 5\n+EDITED\n line 6\n line 7\n line 8\n");
    }

    /// Branch mode's sides are commits, not the worktree, but the hunks are built the
    /// same way -- and `git diff <merge-base>..<tip>` is the same comparison.
    #[test]
    fn branch_mode_hunks_match_git_diff_against_the_merge_base() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        git(dir.path(), DATE, &["checkout", "-b", "feature"]);
        write(dir.path(), "file.txt", "one\ntwo\nTHREE\nfour\n");
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "edit"]);

        let mut spec = spec(DiffMode::Branch, Some("main"));
        spec.branch = "feature".into();
        let hunks = hunks_of(dir.path(), &spec, "file.txt");
        assert_eq!(
            as_unified(&hunks),
            git_diff_hunks(dir.path(), &["main...feature", "--", "file.txt"])
        );
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
