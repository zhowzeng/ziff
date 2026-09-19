//! The Repo's branches: what is checked out, what the Base Branch picker offers, and
//! how far each branch has drifted from its upstream. `fetch_remote` lives here because
//! refreshing the remote refs is what makes those ahead/behind counts current.

use super::paths::repo_root;
use crate::types::{Branch, BranchList, FetchResult};

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
pub fn fetch_remote(_repo_id: String) -> Result<FetchResult, String> {
    Ok(FetchResult {
        success: true,
        message: "Already up to date.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::read_branches;
    use crate::test_repo::{git, open, repo_with_a_commit, DATE};

    #[test]
    fn read_branches_marks_the_checked_out_branch_as_current() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        git(dir.path(), DATE, &["branch", "other"]);

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
        repo_with_a_commit(dir.path());
        git(dir.path(), DATE, &["checkout", "--detach"]);

        let list = read_branches(&open(dir.path())).expect("should read");
        assert!(!list.branches.iter().any(|b| b.is_current));
    }

    #[test]
    fn read_branches_names_the_commit_a_detached_head_sits_on() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        git(dir.path(), DATE, &["checkout", "--detach"]);

        let list = read_branches(&open(dir.path())).expect("should read");
        assert!(list.detached_head.is_some());
    }

    #[test]
    fn read_branches_reports_no_detached_head_while_on_a_branch() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());

        let list = read_branches(&open(dir.path())).expect("should read");
        assert_eq!(list.detached_head, None);
    }

    #[test]
    fn read_branches_puts_the_most_recently_committed_branch_first() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
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
        repo_with_a_commit(dir.path());

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
        repo_with_a_commit(&origin);

        let clone = root.join("clone");
        git(
            root,
            DATE,
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
}
