//! Persistence for the reviewer's Repo list (see docs/decisions/0008).

use std::io::ErrorKind;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::types::Repo;

fn repos_file(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("repos.json"))
        .map_err(|e| format!("Cannot resolve the app config directory: {e}"))
}

pub fn load_repos(app: &AppHandle) -> Result<Vec<Repo>, String> {
    let file = repos_file(app)?;
    match std::fs::read(&file) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|e| format!("Cannot read {}: {e}", file.display())),
        // Nothing saved yet — the reviewer hasn't added a Repo on this machine.
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("Cannot read {}: {e}", file.display())),
    }
}

pub fn save_repos(app: &AppHandle, repos: &[Repo]) -> Result<(), String> {
    let file = repos_file(app)?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create {}: {e}", parent.display()))?;
    }
    let json =
        serde_json::to_vec_pretty(repos).map_err(|e| format!("Cannot serialize repos: {e}"))?;
    std::fs::write(&file, json).map_err(|e| format!("Cannot write {}: {e}", file.display()))
}
