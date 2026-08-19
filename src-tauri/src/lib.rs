// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod launcher;
mod models;
mod proton;
mod store;

use launcher::launch_game;
use proton::{is_umu_installed, list_proton_versions};
use store::{add_game, list_games, remove_game, update_game};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), String> {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            add_game,
            list_games,
            remove_game,
            update_game,
            list_proton_versions,
            is_umu_installed,
            launch_game,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| format!("error while running tauri application: {}", e))
}
