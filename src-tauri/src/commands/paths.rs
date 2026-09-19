//! Resolving what the frontend names -- a `repoId`, a Repo-relative path -- to a real
//! location on disk, and refusing anything that lands outside the Repo (ADR 0005).

use std::path::{Path, PathBuf};

use crate::store;
use crate::types::Repo;

/// Resolves a `repoId` from the frontend to the folder that Repo lives in.
pub(super) fn repo_root(app: &tauri::AppHandle, repo_id: &str) -> Result<PathBuf, String> {
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
pub(super) fn resolve_in_repo(root: &Path, path: &str) -> Result<PathBuf, String> {
    let resolved = std::fs::canonicalize(root.join(path))
        .map_err(|e| format!("Cannot resolve {path}: {e}"))?;
    if !resolved.starts_with(root) {
        return Err(format!("{path} is outside the repo"));
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::{find_repo_root, resolve_in_repo};

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
}
