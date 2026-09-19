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

#[tauri::command]
pub fn get_file_content(_repo_id: String, path: String) -> Result<FileContent, String> {
    let lines = match path.as_str() {
        "src/xlsx_tool.rs" => vec![
            "use calamine::{open_workbook, Reader, Xlsx};".to_string(),
            "".to_string(),
            "calamine = \"0.24\"".to_string(),
            "".to_string(),
            "let mut wb: Xlsx<_> = open_workbook_auto(path)?;".to_string(),
            "let sheet = wb.worksheet_range(\"Sheet1\")?;".to_string(),
        ],
        _ => vec![format!("// mock contents of {path}")],
    };
    Ok(FileContent { lines })
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
    use super::read_repo;

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
}
