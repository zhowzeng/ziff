mod commands;
mod store;
mod types;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::list_repos,
            commands::add_repo,
            commands::remove_repo,
            commands::list_branches,
            commands::get_file_tree,
            commands::get_file_diff,
            commands::get_file_content,
            commands::fetch_remote,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
