use std::path::{Path, PathBuf};

use crate::store;
use crate::types::{
    Branch, BranchList, Changes, DiffHunk, DiffLine, DiffSpec, FetchResult, FileContent, LineKind,
    Repo, TreeNode,
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
pub fn list_branches(app: tauri::AppHandle, repo_id: String) -> Result<BranchList, String> {
    let root = repo_root(&app, &repo_id)?;
    let git = gix::open(&root).map_err(|e| format!("Cannot open {}: {e}", root.display()))?;
    read_branches(&git).map_err(|e| format!("Cannot list branches: {e}"))
}

/// The Repo's local branches, most recently committed first.
///
/// Recency rather than alphabetical order: this list is the Base Branch picker, and the
/// branch a reviewer wants to compare against is far more likely to be one they have
/// touched than one whose name sorts early. Remote-tracking branches are deliberately
/// left out for now -- see the issue this came from.
fn read_branches(
    git: &gix::Repository,
) -> Result<BranchList, Box<dyn std::error::Error + Send + Sync>> {
    let head = git.head_name()?;
    let current = head.as_ref().map(|name| name.shorten().to_string());

    let mut rows = Vec::new();
    for reference in git.references()?.local_branches()? {
        let mut reference = reference?;
        let name = reference.name().shorten().to_string();
        let tip = reference.peel_to_id()?.detach();
        let committed_at = git.find_commit(tip)?.time()?.seconds;
        let (ahead, behind) = upstream_gap(git, &reference, tip);
        rows.push((
            committed_at,
            Branch {
                is_current: current.as_deref() == Some(name.as_str()),
                name,
                ahead,
                behind,
            },
        ));
    }
    rows.sort_by(|(a_time, a), (b_time, b)| b_time.cmp(a_time).then_with(|| a.name.cmp(&b.name)));

    Ok(BranchList {
        branches: rows.into_iter().map(|(_, branch)| branch).collect(),
        // A detached HEAD is on no branch at all, so every `is_current` above is false
        // and the frontend needs this to say what is checked out instead.
        detached_head: match head {
            Some(_) => None,
            None => Some(git.head_id()?.shorten_or_id().to_string()),
        },
    })
}

/// Commits each side has that the other does not, or `(None, None)` when the branch has
/// no upstream -- "nothing to compare" and "in sync" are different answers.
fn upstream_gap(
    git: &gix::Repository,
    reference: &gix::Reference<'_>,
    tip: gix::ObjectId,
) -> (Option<u32>, Option<u32>) {
    let Some(Ok(upstream_name)) = reference.remote_tracking_ref_name(gix::remote::Direction::Fetch)
    else {
        return (None, None);
    };
    // The name is derived from the refspec, so it can name a ref that was never fetched.
    let Ok(mut upstream) = git.find_reference(upstream_name.as_ref()) else {
        return (None, None);
    };
    let Ok(upstream_tip) = upstream.peel_to_id() else {
        return (None, None);
    };
    let upstream_tip = upstream_tip.detach();
    (
        count_commits(git, tip, upstream_tip),
        count_commits(git, upstream_tip, tip),
    )
}

/// `git rev-list --count <hidden>..<tip>`.
fn count_commits(git: &gix::Repository, tip: gix::ObjectId, hidden: gix::ObjectId) -> Option<u32> {
    let walk = git
        .rev_walk(Some(tip))
        .with_hidden(Some(hidden))
        .all()
        .ok()?;
    let mut count = 0;
    for info in walk {
        info.ok()?;
        count += 1;
    }
    Some(count)
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
    use super::{
        find_repo_root, read_branches, read_file_content, read_repo, resolve_in_repo, split_lines,
    };

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

    /// Branch fixtures need real commits, so these tests drive the `git` CLI rather
    /// than hand-writing objects. Global and system config are cut off so a
    /// contributor's own git settings can't change what the fixtures produce.
    fn git(dir: &std::path::Path, date: &str, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_AUTHOR_NAME", "Ziff Test")
            .env("GIT_AUTHOR_EMAIL", "test@ziff.invalid")
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_NAME", "Ziff Test")
            .env("GIT_COMMITTER_EMAIL", "test@ziff.invalid")
            .env("GIT_COMMITTER_DATE", date)
            .output()
            .expect("run git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// A repo on `main` with one commit.
    fn repo_with_a_commit(dir: &std::path::Path, date: &str) {
        git(dir, date, &["init", "-b", "main", "."]);
        std::fs::write(dir.join("file.txt"), "one\n").expect("write");
        git(dir, date, &["add", "."]);
        git(dir, date, &["commit", "-m", "first"]);
    }

    fn open(dir: &std::path::Path) -> gix::Repository {
        gix::open(dir).expect("open")
    }

    #[test]
    fn read_branches_marks_the_checked_out_branch_as_current() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path(), "2020-01-01T00:00:00Z");
        git(dir.path(), "2020-01-01T00:00:00Z", &["branch", "other"]);

        let list = read_branches(&open(dir.path())).expect("should read");
        let current: Vec<_> = list
            .branches
            .iter()
            .filter(|b| b.is_current)
            .map(|b| b.name.as_str())
            .collect();
        assert_eq!(current, ["main"]);
    }

    /// The gap ADR 0010 left open: with no branch checked out, nothing may be reported
    /// as current, or the topbar would name a branch the reviewer is not on.
    #[test]
    fn read_branches_marks_nothing_as_current_when_head_is_detached() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path(), "2020-01-01T00:00:00Z");
        git(
            dir.path(),
            "2020-01-01T00:00:00Z",
            &["checkout", "--detach"],
        );

        let list = read_branches(&open(dir.path())).expect("should read");
        assert!(!list.branches.iter().any(|b| b.is_current));
    }

    #[test]
    fn read_branches_names_the_commit_a_detached_head_sits_on() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path(), "2020-01-01T00:00:00Z");
        git(
            dir.path(),
            "2020-01-01T00:00:00Z",
            &["checkout", "--detach"],
        );

        let list = read_branches(&open(dir.path())).expect("should read");
        assert!(list.detached_head.is_some());
    }

    #[test]
    fn read_branches_reports_no_detached_head_while_on_a_branch() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path(), "2020-01-01T00:00:00Z");

        let list = read_branches(&open(dir.path())).expect("should read");
        assert_eq!(list.detached_head, None);
    }

    #[test]
    fn read_branches_puts_the_most_recently_committed_branch_first() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path(), "2020-01-01T00:00:00Z");
        git(
            dir.path(),
            "2021-06-01T00:00:00Z",
            &["checkout", "-b", "newer"],
        );
        std::fs::write(dir.path().join("file.txt"), "two\n").expect("write");
        git(
            dir.path(),
            "2021-06-01T00:00:00Z",
            &["commit", "-am", "second"],
        );

        let list = read_branches(&open(dir.path())).expect("should read");
        let names: Vec<_> = list.branches.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, ["newer", "main"]);
    }

    /// "No upstream" and "in sync" are different answers, so a branch with nothing to
    /// compare against must not report 0.
    #[test]
    fn read_branches_reports_no_counts_for_a_branch_without_an_upstream() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path(), "2020-01-01T00:00:00Z");

        let list = read_branches(&open(dir.path())).expect("should read");
        assert_eq!(
            (list.branches[0].ahead, list.branches[0].behind),
            (None, None)
        );
    }

    /// A clone, so `branch.main.merge` and `refs/remotes/origin/main` are set up the
    /// way they are for a repo a reviewer actually works in.
    fn clone_with_upstream(root: &std::path::Path) -> std::path::PathBuf {
        let origin = root.join("origin");
        std::fs::create_dir(&origin).expect("create origin");
        repo_with_a_commit(&origin, "2020-01-01T00:00:00Z");

        let clone = root.join("clone");
        git(
            root,
            "2020-01-01T00:00:00Z",
            &["clone", &origin.to_string_lossy(), &clone.to_string_lossy()],
        );
        clone
    }

    #[test]
    fn read_branches_counts_commits_ahead_of_the_upstream() {
        let dir = tempfile::tempdir().expect("tempdir");
        let clone = clone_with_upstream(dir.path());
        std::fs::write(clone.join("file.txt"), "local\n").expect("write");
        git(
            &clone,
            "2020-02-01T00:00:00Z",
            &["commit", "-am", "local work"],
        );

        let list = read_branches(&open(&clone)).expect("should read");
        assert_eq!(list.branches[0].ahead, Some(1));
    }

    #[test]
    fn read_branches_counts_commits_behind_the_upstream() {
        let dir = tempfile::tempdir().expect("tempdir");
        let clone = clone_with_upstream(dir.path());
        let origin = dir.path().join("origin");
        std::fs::write(origin.join("file.txt"), "remote\n").expect("write");
        git(
            &origin,
            "2020-02-01T00:00:00Z",
            &["commit", "-am", "remote work"],
        );
        git(&clone, "2020-02-01T00:00:00Z", &["fetch"]);

        let list = read_branches(&open(&clone)).expect("should read");
        assert_eq!(list.branches[0].behind, Some(1));
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
