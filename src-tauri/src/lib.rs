// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use utils::api::{get_exe_path, get_voices_list, start_tts, get_app_version};
pub mod utils;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_voices_list,
            start_tts,
            get_exe_path,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
