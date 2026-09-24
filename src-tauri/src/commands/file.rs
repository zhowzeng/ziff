//! What the two panes of the reviewer's main view read: the hunks for a changed file,
//! and the worktree text File View numbers lines against (ADR 0007).

use std::path::Path;

use super::paths::{repo_root, resolve_in_repo};
use crate::diff;
use crate::types::{DiffSpec, FileContent, FileDiff};

#[tauri::command]
pub fn get_file_diff(
    app: tauri::AppHandle,
    spec: DiffSpec,
    path: String,
) -> Result<FileDiff, String> {
    let root = repo_root(&app, &spec.repo_id)?;
    let git = gix::open(&root).map_err(|e| format!("Cannot open {}: {e}", root.display()))?;
    read_file_diff(&git, &spec, &path).map_err(|e| format!("Cannot diff {path}: {e}"))
}

/// The hunks for one file of the current diff.
///
/// A path that does not differ in this Diff Mode has no hunks: the frontend opens File
/// View for those (ADR 0007), so this is only reached for a file whose tree node lost
/// its `changes` between the tree load and the click.
fn read_file_diff(
    git: &gix::Repository,
    spec: &DiffSpec,
    path: &str,
) -> Result<FileDiff, Box<dyn std::error::Error + Send + Sync>> {
    let changed = diff::changed_files(git, spec)?;
    match changed.iter().find(|file| file.path == path) {
        Some(file) => diff::file_diff(git, file),
        None => Ok(FileDiff {
            hunks: Vec::new(),
            binary: false,
            old_text: String::new(),
            new_text: String::new(),
        }),
    }
}

/// Reads a file the way File View shows it: the worktree copy, never the index or
/// HEAD, because the `path:L12` handed to a CLI agent is resolved against the
/// worktree too (ADR 0007, ADR 0010).
#[tauri::command]
pub fn get_file_content(
    app: tauri::AppHandle,
    repo_id: String,
    path: String,
) -> Result<FileContent, String> {
    let root = repo_root(&app, &repo_id)?;
    let file = resolve_in_repo(&root, &path)?;
    read_file_content(&file).map_err(|e| format!("Cannot read {path}: {e}"))
}

/// A file that is not valid UTF-8 is reported as binary rather than decoded lossily:
/// its line numbers would be made up, and a Comment anchors to a line number.
fn read_file_content(file: &Path) -> std::io::Result<FileContent> {
    let bytes = std::fs::read(file)?;
    match String::from_utf8(bytes) {
        Ok(text) => Ok(FileContent {
            lines: split_lines(&text),
            binary: false,
        }),
        Err(_) => Ok(FileContent {
            lines: Vec::new(),
            binary: true,
        }),
    }
}

/// Splits file text into the lines File View numbers from 1. The newline that ends a
/// well-formed file does not start a further line, and a CRLF checkout must not render
/// a stray carriage return on every line.
fn split_lines(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    text.strip_suffix('\n')
        .unwrap_or(text)
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{read_file_content, split_lines};

    #[test]
    fn read_file_content_reports_a_file_that_is_not_utf8_as_binary() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("logo.png");
        // A PNG's first bytes: 0x89 never starts a valid UTF-8 sequence.
        std::fs::write(&file, [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]).expect("write");

        let content = read_file_content(&file).expect("should read");
        assert!(content.binary);
    }

    #[test]
    fn read_file_content_reports_a_binary_file_as_having_no_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("logo.png");
        std::fs::write(&file, [0x89, b'P', b'N', b'G']).expect("write");

        let content = read_file_content(&file).expect("should read");
        assert!(content.lines.is_empty());
    }

    #[test]
    fn read_file_content_reports_a_text_file_as_not_binary() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("main.rs");
        std::fs::write(&file, "fn main() {}\n").expect("write");

        let content = read_file_content(&file).expect("should read");
        assert!(!content.binary);
    }

    #[test]
    fn split_lines_does_not_count_the_newline_that_ends_a_file() {
        assert_eq!(split_lines("one\ntwo\n"), vec!["one", "two"]);
    }

    #[test]
    fn split_lines_keeps_a_last_line_with_no_trailing_newline() {
        assert_eq!(split_lines("one\ntwo"), vec!["one", "two"]);
    }

    #[test]
    fn split_lines_strips_crlf() {
        assert_eq!(split_lines("one\r\ntwo\r\n"), vec!["one", "two"]);
    }

    #[test]
    fn split_lines_of_an_empty_file_has_no_lines() {
        assert!(split_lines("").is_empty());
    }

    #[test]
    fn split_lines_of_a_lone_newline_is_one_empty_line() {
        assert_eq!(split_lines("\n"), vec![""]);
    }
}
