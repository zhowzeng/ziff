//! Resolving a Diff Mode into the two sides being compared, and working out what
//! changed between them (docs/decisions/0003 picks gix for this).
//!
//! The work splits in two, and the dependency only runs one way:
//!
//! - [`changes`] answers *which files differ*, working in git refs, trees and the index.
//! - [`hunks`] answers *how one file differs*, working in lines of text -- it reads a
//!   [`ChangedFile`]'s two [`Side`]s and never looks at a ref again.

mod changes;
mod hunks;

pub use changes::{changed_files, tracked_paths};
pub use hunks::{count_changes, file_diff};

use std::path::PathBuf;

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
                Ok(bytes) => to_git(git, path, bytes),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
                Err(e) => Err(e.into()),
            },
        }
    }
}

/// A worktree file's bytes as git would store them -- `core.autocrlf`, `.gitattributes`
/// `eol` / `text` and the rest applied -- so they diff against a blob the way `git diff`
/// does. Without this, a CRLF checkout of an LF blob differs on every line.
fn to_git(git: &gix::Repository, path: &std::path::Path, bytes: Vec<u8>) -> Result<Vec<u8>, Error> {
    let rela_path = path.strip_prefix(git.workdir().ok_or("This repo has no working tree")?)?;
    let (mut pipeline, index) = git.filter_pipeline(None)?;
    let mut outcome = pipeline.convert_to_git(bytes.as_slice(), rela_path, &index)?;
    if !outcome.is_changed() {
        return Ok(bytes);
    }
    let mut out = Vec::new();
    std::io::Read::read_to_end(&mut outcome, &mut out)?;
    Ok(out)
}

/// Fixtures the tests on both halves need: `changes` drives a rename through every Diff
/// Mode, and `hunks` diffs a renamed file against where it came from.
#[cfg(test)]
mod fixtures {
    use crate::test_repo::{git, write, DATE};
    use crate::types::{DiffMode, DiffSpec};

    pub(super) fn spec(diff_mode: DiffMode, base_branch: Option<&str>) -> DiffSpec {
        DiffSpec {
            repo_id: "/unused".into(),
            branch: "main".into(),
            diff_mode,
            base_branch: base_branch.map(str::to_string),
        }
    }

    /// Big enough that similarity detection has something to work with.
    pub(super) fn commit_a_big_file(dir: &std::path::Path, name: &str) {
        let body: String = (1..=200).map(|i| format!("line {i}\n")).collect();
        write(dir, name, &body);
        git(dir, DATE, &["add", "."]);
        git(dir, DATE, &["commit", "-m", "big"]);
    }
}
