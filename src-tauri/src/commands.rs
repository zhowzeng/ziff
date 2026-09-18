use crate::types::{
    Branch, Changes, DiffHunk, DiffLine, DiffSpec, FetchResult, FileContent, LineKind, Repo,
    TreeNode,
};

#[tauri::command]
pub fn list_repos() -> Result<Vec<Repo>, String> {
    Ok(vec![
        Repo {
            id: "goose".into(),
            name: "goose".into(),
            path: "~/dev/goose".into(),
            default_branch: "main".into(),
        },
        Repo {
            id: "goose-mcp-extensions".into(),
            name: "goose-mcp-extensions".into(),
            path: "~/dev/goose-mcp-extensions".into(),
            default_branch: "main".into(),
        },
        Repo {
            id: "block-design-system".into(),
            name: "block-design-system".into(),
            path: "~/dev/block-design-system".into(),
            default_branch: "main".into(),
        },
    ])
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
