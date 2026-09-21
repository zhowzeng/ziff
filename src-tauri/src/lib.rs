mod commands;
mod diff;
mod store;
#[cfg(test)]
mod test_repo;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::repo::list_repos,
            commands::repo::add_repo,
            commands::repo::remove_repo,
            commands::branch::list_branches,
            commands::tree::get_file_tree,
            commands::file::get_file_diff,
            commands::file::get_file_content,
            commands::branch::fetch_remote,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
