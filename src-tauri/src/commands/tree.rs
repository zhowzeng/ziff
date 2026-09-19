//! The sidebar's file tree: which files the Diff Mode touched, and how the flat list of
//! repo-relative paths becomes the nested tree the frontend renders.

use std::collections::BTreeMap;

use super::paths::repo_root;
use crate::diff;
use crate::types::{Changes, DiffSpec, TreeNode};

#[tauri::command]
pub fn get_file_tree(app: tauri::AppHandle, spec: DiffSpec) -> Result<Vec<TreeNode>, String> {
    let root = repo_root(&app, &spec.repo_id)?;
    let git = gix::open(&root).map_err(|e| format!("Cannot open {}: {e}", root.display()))?;
    read_file_tree(&git, &spec).map_err(|e| format!("Cannot list changed files: {e}"))
}

/// The sidebar's tree: every file the checkout tracks, with `changes` on the ones that
/// differ in this Diff Mode.
///
/// Unchanged files are in here on purpose. "顯示所有檔案" browses the whole project, and
/// File View lets a reviewer comment on a file with no diff at all (ADR 0007) -- the
/// frontend prunes to the changed ones when that box is unticked, so returning only
/// changed files here would take both away.
fn read_file_tree(
    git: &gix::Repository,
    spec: &DiffSpec,
) -> Result<Vec<TreeNode>, Box<dyn std::error::Error + Send + Sync>> {
    let changed = diff::changed_files(git, spec)?;
    let mut changes = BTreeMap::new();
    let mut renames = BTreeMap::new();
    let mut moved_away = Vec::new();
    for file in &changed {
        changes.insert(file.path.clone(), diff::count_changes(git, file)?);
        if let Some(old_path) = &file.old_path {
            renames.insert(file.path.clone(), old_path.clone());
            moved_away.push(old_path.clone());
        }
    }
    // A file deleted on this side isn't in the index any more, so it has to be carried
    // in from the change list or it would vanish from the tree entirely.
    let extra: Vec<String> = changed.into_iter().map(|file| file.path).collect();
    let mut paths = diff::tracked_paths(git, &extra)?;
    // A renamed file's old path is still in the index until the rename is committed.
    // Leaving it in would list a file the worktree no longer has, right next to the
    // same file under its new name.
    paths.retain(|path| !moved_away.contains(path));
    Ok(build_level(&paths, &changes, &renames, "", 0))
}

/// Groups sorted repo-relative paths into one level of the tree, recursing per
/// directory. Sorted input is what makes this work: everything under one directory is
/// adjacent, so a directory's children are a contiguous slice.
fn build_level(
    paths: &[String],
    changes: &BTreeMap<String, Changes>,
    renames: &BTreeMap<String, String>,
    prefix: &str,
    depth: usize,
) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let mut i = 0;
    while i < paths.len() {
        let name = paths[i]
            .split('/')
            .nth(depth)
            .unwrap_or_default()
            .to_string();
        let is_file = paths[i].split('/').count() == depth + 1;
        if is_file {
            nodes.push(TreeNode::File {
                name,
                changes: changes.get(&paths[i]).copied(),
                renamed_from: renames.get(&paths[i]).cloned(),
                path: paths[i].clone(),
            });
            i += 1;
        } else {
            let dir_prefix = format!("{prefix}{name}/");
            let len = paths[i..]
                .iter()
                .take_while(|path| path.starts_with(&dir_prefix))
                .count();
            nodes.push(TreeNode::Dir {
                path: format!("{prefix}{name}"),
                name,
                children: build_level(&paths[i..i + len], changes, renames, &dir_prefix, depth + 1),
            });
            i += len;
        }
    }
    // Directories first, the way a file tree is normally read.
    nodes.sort_by(|a, b| match (a, b) {
        (TreeNode::Dir { .. }, TreeNode::File { .. }) => std::cmp::Ordering::Less,
        (TreeNode::File { .. }, TreeNode::Dir { .. }) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    });
    nodes
}

#[cfg(test)]
mod tests {
    use super::{build_level, read_file_tree};
    use crate::test_repo::{git, open, repo_with_a_commit, write, DATE};
    use crate::types::{DiffMode, DiffSpec, TreeNode};

    fn unstaged_spec() -> DiffSpec {
        DiffSpec {
            repo_id: "/unused".into(),
            branch: "main".into(),
            diff_mode: DiffMode::Unstaged,
            base_branch: None,
        }
    }

    fn names(nodes: &[TreeNode]) -> Vec<&str> {
        nodes
            .iter()
            .map(|node| match node {
                TreeNode::Dir { name, .. } | TreeNode::File { name, .. } => name.as_str(),
            })
            .collect()
    }

    /// The sidebar browses the whole project ("顯示所有檔案") and File View comments on
    /// files with no diff (ADR 0007), so an unchanged file has to be in the tree --
    /// just without `changes`.
    #[test]
    fn the_tree_keeps_a_file_that_did_not_change() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());

        let tree = read_file_tree(&open(dir.path()), &unstaged_spec()).expect("tree");
        assert!(matches!(
            tree.as_slice(),
            [TreeNode::File { changes: None, .. }]
        ));
    }

    #[test]
    fn the_tree_marks_a_changed_file_with_its_counts() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(
            dir.path(),
            "file.txt",
            "one
two
three
four
",
        );

        let tree = read_file_tree(&open(dir.path()), &unstaged_spec()).expect("tree");
        let TreeNode::File { changes, .. } = &tree[0] else {
            panic!("expected a file, got {tree:?}");
        };
        assert_eq!(changes.map(|c| (c.add, c.del)), Some((1, 0)));
    }

    #[test]
    fn the_tree_nests_a_file_under_its_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "src/deep/main.rs", "fn main() {}\n");
        git(dir.path(), DATE, &["add", "."]);

        let tree = read_file_tree(&open(dir.path()), &unstaged_spec()).expect("tree");
        let TreeNode::Dir { children, .. } = &tree[0] else {
            panic!("expected a dir first, got {tree:?}");
        };
        let TreeNode::Dir { children, .. } = &children[0] else {
            panic!("expected a nested dir, got {children:?}");
        };
        assert_eq!(names(children), ["main.rs"]);
    }

    #[test]
    fn the_tree_puts_directories_before_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), "src/main.rs", "fn main() {}\n");
        git(dir.path(), DATE, &["add", "."]);

        let tree = read_file_tree(&open(dir.path()), &unstaged_spec()).expect("tree");
        assert_eq!(names(&tree), ["src", "file.txt"]);
    }

    #[test]
    fn the_tree_leaves_out_an_ignored_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        write(dir.path(), ".gitignore", "target\n");
        git(dir.path(), DATE, &["add", ".gitignore"]);
        git(dir.path(), DATE, &["commit", "-m", "ignore"]);
        write(dir.path(), "target/build.log", "noise\n");

        let tree = read_file_tree(&open(dir.path()), &unstaged_spec()).expect("tree");
        assert!(!names(&tree).contains(&"target"), "got: {:?}", names(&tree));
    }

    /// The old path is still in the index until the rename is committed. Leaving it in
    /// the tree would list a file the worktree no longer has, right beside the same
    /// file under its new name.
    #[test]
    fn the_tree_drops_the_path_a_file_moved_away_from() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        let body: String = (1..=200).map(|i| format!("line {i}\n")).collect();
        write(dir.path(), "big.txt", &body);
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "big"]);
        std::fs::rename(dir.path().join("big.txt"), dir.path().join("moved.txt")).expect("mv");

        let tree = read_file_tree(&open(dir.path()), &unstaged_spec()).expect("tree");
        assert_eq!(names(&tree), ["file.txt", "moved.txt"]);
    }

    #[test]
    fn the_tree_says_where_a_moved_file_came_from() {
        let dir = tempfile::tempdir().expect("tempdir");
        repo_with_a_commit(dir.path());
        let body: String = (1..=200).map(|i| format!("line {i}\n")).collect();
        write(dir.path(), "big.txt", &body);
        git(dir.path(), DATE, &["add", "."]);
        git(dir.path(), DATE, &["commit", "-m", "big"]);
        std::fs::rename(dir.path().join("big.txt"), dir.path().join("moved.txt")).expect("mv");

        let tree = read_file_tree(&open(dir.path()), &unstaged_spec()).expect("tree");
        let renamed: Vec<_> = tree
            .iter()
            .filter_map(|node| match node {
                TreeNode::File { renamed_from, .. } => renamed_from.as_deref(),
                TreeNode::Dir { .. } => None,
            })
            .collect();
        assert_eq!(renamed, ["big.txt"]);
    }

    #[test]
    fn build_level_of_no_paths_is_an_empty_tree() {
        assert!(
            build_level(&[], &super::BTreeMap::new(), &super::BTreeMap::new(), "", 0).is_empty()
        );
    }
}
