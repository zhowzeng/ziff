use std::path::{Path, PathBuf};

use crate::store;
use crate::types::{
    Branch, Changes, DiffHunk, DiffLine, DiffSpec, FetchResult, FileContent, LineKind, Repo,
    TreeNode,
};

#[tauri::command]
pub fn list_repos(app: tauri::AppHandle) -> Result<Vec<Repo>, String> {
    store::load_repos(&app)
}

/// Registers the git repository at `path` as a Repo, keyed by its canonical path so
/// adding the same folder twice doesn't duplicate it.
#[tauri::command]
pub fn add_repo(app: tauri::AppHandle, path: String) -> Result<Repo, String> {
    let repo = read_repo(&path)?;
    let mut repos = store::load_repos(&app)?;
    if let Some(existing) = repos.iter().find(|r| r.id == repo.id) {
        return Ok(existing.clone());
    }
    repos.push(repo.clone());
    store::save_repos(&app, &repos)?;
    Ok(repo)
}

/// Drops the Repo from the reviewer's list. This only edits Ziff's own list — the
/// folder on disk is left alone.
#[tauri::command]
pub fn remove_repo(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut repos = store::load_repos(&app)?;
    repos.retain(|r| r.id != id);
    store::save_repos(&app, &repos)
}

fn read_repo(path: &str) -> Result<Repo, String> {
    let git = gix::open(path).map_err(|e| format!("Not a git repository: {path} ({e})"))?;
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("Cannot resolve {path}: {e}"))?
        .to_string_lossy()
        .into_owned();
    let name = std::path::Path::new(&canonical)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| canonical.clone());
    Ok(Repo {
        id: canonical.clone(),
        name,
        path: canonical,
        default_branch: default_branch(&git),
    })
}

/// The Repo's default branch: what `origin/HEAD` points at, falling back to whatever
/// branch is checked out for a repository with no remote.
fn default_branch(git: &gix::Repository) -> String {
    if let Ok(head) = git.find_reference("refs/remotes/origin/HEAD") {
        if let gix::refs::TargetRef::Symbolic(name) = head.target() {
            let short = name.shorten().to_string();
            if let Some(branch) = short.strip_prefix("origin/") {
                return branch.to_string();
            }
        }
    }
    git.head_name()
        .ok()
        .flatten()
        .map(|name| name.shorten().to_string())
        .unwrap_or_else(|| "main".to_string())
}

/// Resolves a `repoId` from the frontend to the folder that Repo lives in.
fn repo_root(app: &tauri::AppHandle, repo_id: &str) -> Result<PathBuf, String> {
    find_repo_root(&store::load_repos(app)?, repo_id)
}

/// A Repo's id is the canonical path it had when it was added (ADR 0008), but the
/// folder can have been moved or deleted since, so the path is resolved again here
/// rather than trusted.
fn find_repo_root(repos: &[Repo], repo_id: &str) -> Result<PathBuf, String> {
    let repo = repos
        .iter()
        .find(|r| r.id == repo_id)
        .ok_or_else(|| format!("No such repo: {repo_id}"))?;
    std::fs::canonicalize(&repo.path)
        .map_err(|e| format!("Repo folder is no longer readable: {} ({e})", repo.path))
}

/// Turns a Repo-relative path from the frontend into an absolute one, refusing
/// anything that lands outside the Repo.
///
/// ADR 0005 puts this check in Rust rather than in static capability config, because
/// a Repo is an arbitrary folder added at runtime. `path` is caller-supplied, so both
/// `../` and a symlink pointing out of the Repo have to be caught *after* the path is
/// resolved on disk -- inspecting the string alone would miss the symlink.
fn resolve_in_repo(root: &Path, path: &str) -> Result<PathBuf, String> {
    let resolved = std::fs::canonicalize(root.join(path))
        .map_err(|e| format!("Cannot resolve {path}: {e}"))?;
    if !resolved.starts_with(root) {
        return Err(format!("{path} is outside the repo"));
    }
    Ok(resolved)
}

#[tauri::command]
pub fn list_branches(_repo_id: String) -> Result<Vec<Branch>, String> {
    Ok(vec![
        Branch {
            name: "feature/xlsx-api-upgrade".into(),
            is_current: true,
            ahead: 3,
            behind: 0,
        },
        Branch {
            name: "main".into(),
            is_current: false,
            ahead: 0,
            behind: 2,
        },
        Branch {
            name: "develop".into(),
            is_current: false,
            ahead: 0,
            behind: 0,
        },
        Branch {
            name: "feature/mcp-comment-queue".into(),
            is_current: false,
            ahead: 0,
            behind: 0,
        },
    ])
}

#[tauri::command]
pub fn get_file_tree(_spec: DiffSpec) -> Result<Vec<TreeNode>, String> {
    Ok(vec![
        TreeNode::Dir {
            name: "src".into(),
            path: "src".into(),
            children: vec![
                TreeNode::File {
                    name: "xlsx_tool.rs".into(),
                    path: "src/xlsx_tool.rs".into(),
                    changes: Some(Changes { add: 12, del: 4 }),
                },
                TreeNode::File {
                    name: "main.rs".into(),
                    path: "src/main.rs".into(),
                    changes: None,
                },
            ],
        },
        TreeNode::File {
            name: "Cargo.toml".into(),
            path: "Cargo.toml".into(),
            changes: Some(Changes { add: 2, del: 1 }),
        },
    ])
}

#[tauri::command]
pub fn get_file_diff(_spec: DiffSpec, path: String) -> Result<Vec<DiffHunk>, String> {
    match path.as_str() {
        "src/xlsx_tool.rs" => Ok(vec![DiffHunk {
            header: "@@ -10,5 +10,5 @@ fn load_workbook(path: &Path) -> Result<Xlsx<...>>".into(),
            lines: vec![
                DiffLine {
                    kind: LineKind::Context,
                    old_no: Some(10),
                    new_no: Some(10),
                    content: "use calamine::{open_workbook, Reader, Xlsx};".into(),
                },
                DiffLine {
                    kind: LineKind::Del,
                    old_no: Some(11),
                    new_no: None,
                    content: "calamine = \"0.22\"".into(),
                },
                DiffLine {
                    kind: LineKind::Add,
                    old_no: None,
                    new_no: Some(11),
                    content: "calamine = \"0.24\"".into(),
                },
                DiffLine {
                    kind: LineKind::Context,
                    old_no: Some(12),
                    new_no: Some(12),
                    content: "".into(),
                },
                DiffLine {
                    kind: LineKind::Del,
                    old_no: Some(13),
                    new_no: None,
                    content: "let mut wb: Xlsx<_> = open_workbook(path)?;".into(),
                },
                DiffLine {
                    kind: LineKind::Add,
                    old_no: None,
                    new_no: Some(13),
                    content: "let mut wb: Xlsx<_> = open_workbook_auto(path)?;".into(),
                },
                DiffLine {
                    kind: LineKind::Context,
                    old_no: Some(14),
                    new_no: Some(14),
                    content: "let sheet = wb.worksheet_range(\"Sheet1\")?;".into(),
                },
            ],
        }]),
        "Cargo.toml" => Ok(vec![DiffHunk {
            header: "@@ -20,3 +20,4 @@".into(),
            lines: vec![
                DiffLine {
                    kind: LineKind::Context,
                    old_no: Some(20),
                    new_no: Some(20),
                    content: "[dependencies]".into(),
                },
                DiffLine {
                    kind: LineKind::Del,
                    old_no: Some(21),
                    new_no: None,
                    content: "calamine = \"0.22\"".into(),
                },
                DiffLine {
                    kind: LineKind::Add,
                    old_no: None,
                    new_no: Some(21),
                    content: "calamine = \"0.24\"".into(),
                },
                DiffLine {
                    kind: LineKind::Add,
                    old_no: None,
                    new_no: Some(22),
                    content: "anyhow = \"1\"".into(),
                },
            ],
        }]),
        _ => Ok(vec![]),
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

#[tauri::command]
pub fn fetch_remote(_repo_id: String) -> Result<FetchResult, String> {
    Ok(FetchResult {
        success: true,
        message: "Already up to date.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::{find_repo_root, read_file_content, read_repo, resolve_in_repo, split_lines};

    /// Lays out a git repository whose HEAD is on `head_branch`, optionally with an
    /// `origin/HEAD` symref — the two things `default_branch` reads.
    fn make_repo(dir: &std::path::Path, head_branch: &str, origin_head: Option<&str>) {
        gix::init(dir).expect("init");
        let git_dir = dir.join(".git");
        std::fs::write(
            git_dir.join("HEAD"),
            format!("ref: refs/heads/{head_branch}\n"),
        )
        .expect("write HEAD");
        if let Some(branch) = origin_head {
            let remote = git_dir.join("refs/remotes/origin");
            std::fs::create_dir_all(&remote).expect("create origin refs");
            std::fs::write(
                remote.join("HEAD"),
                format!("ref: refs/remotes/origin/{branch}\n"),
            )
            .expect("write origin/HEAD");
        }
    }

    #[test]
    fn rejects_a_directory_that_is_not_a_git_repo() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = read_repo(&dir.path().to_string_lossy()).expect_err("should reject");
        assert!(err.starts_with("Not a git repository:"), "got: {err}");
    }

    #[test]
    fn default_branch_comes_from_origin_head() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("my-repo");
        std::fs::create_dir(&root).expect("create repo dir");
        make_repo(&root, "feature/xlsx-api-upgrade", Some("main"));

        let repo = read_repo(&root.to_string_lossy()).expect("should read");
        assert_eq!(repo.name, "my-repo");
        assert_eq!(repo.default_branch, "main");
        // A Repo is identified by its canonical path, so the same folder added twice
        // keeps one id.
        assert_eq!(repo.id, repo.path);
        assert_eq!(
            repo.path,
            std::fs::canonicalize(&root).unwrap().to_string_lossy()
        );
    }

    #[test]
    fn falls_back_to_the_checked_out_branch_without_a_remote() {
        let dir = tempfile::tempdir().expect("tempdir");
        make_repo(dir.path(), "develop", None);

        let repo = read_repo(&dir.path().to_string_lossy()).expect("should read");
        assert_eq!(repo.default_branch, "develop");
    }

    /// A Repo entry as `repos.json` holds it, pointing at `path`.
    fn repo_entry(path: &std::path::Path) -> crate::types::Repo {
        let path = path.to_string_lossy().into_owned();
        crate::types::Repo {
            id: path.clone(),
            name: "repo".into(),
            path,
            default_branch: "main".into(),
        }
    }

    #[test]
    fn find_repo_root_rejects_an_id_that_is_not_in_the_list() {
        let err = find_repo_root(&[], "/not/added").expect_err("should reject");
        assert_eq!(err, "No such repo: /not/added");
    }

    #[test]
    fn find_repo_root_reports_a_repo_folder_that_has_gone_away() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("moved-away");
        std::fs::create_dir(&root).expect("create repo dir");
        let repos = vec![repo_entry(&root)];
        std::fs::remove_dir(&root).expect("remove repo dir");

        let err = find_repo_root(&repos, &repos[0].id).expect_err("should reject");
        assert!(
            err.starts_with("Repo folder is no longer readable:"),
            "got: {err}"
        );
    }

    #[test]
    fn resolve_in_repo_accepts_a_file_inside_the_repo() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(dir.path()).expect("canonicalize");
        std::fs::create_dir(root.join("src")).expect("create src");
        std::fs::write(root.join("src/main.rs"), "fn main() {}").expect("write file");

        let resolved = resolve_in_repo(&root, "src/main.rs").expect("should resolve");
        assert_eq!(resolved, root.join("src/main.rs"));
    }

    #[test]
    fn resolve_in_repo_rejects_a_relative_path_that_climbs_out() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(dir.path()).expect("canonicalize");
        let repo = root.join("repo");
        std::fs::create_dir(&repo).expect("create repo dir");
        std::fs::write(root.join("secret.txt"), "shh").expect("write outside file");

        let err = resolve_in_repo(&repo, "../secret.txt").expect_err("should reject");
        assert_eq!(err, "../secret.txt is outside the repo");
    }

    #[test]
    fn resolve_in_repo_rejects_an_absolute_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(dir.path()).expect("canonicalize");
        let repo = root.join("repo");
        std::fs::create_dir(&repo).expect("create repo dir");
        let outside = root.join("secret.txt");
        std::fs::write(&outside, "shh").expect("write outside file");

        let err = resolve_in_repo(&repo, &outside.to_string_lossy()).expect_err("should reject");
        assert!(err.ends_with("is outside the repo"), "got: {err}");
    }

    /// A symlink is why the guard resolves the path on disk instead of inspecting the
    /// string: this one contains no `..` at all.
    #[cfg(unix)]
    #[test]
    fn resolve_in_repo_rejects_a_symlink_pointing_out_of_the_repo() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = std::fs::canonicalize(dir.path()).expect("canonicalize");
        let repo = root.join("repo");
        std::fs::create_dir(&repo).expect("create repo dir");
        let outside = root.join("secret.txt");
        std::fs::write(&outside, "shh").expect("write outside file");
        std::os::unix::fs::symlink(&outside, repo.join("link.txt")).expect("symlink");

        let err = resolve_in_repo(&repo, "link.txt").expect_err("should reject");
        assert_eq!(err, "link.txt is outside the repo");
    }

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
