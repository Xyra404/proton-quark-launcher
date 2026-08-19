// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod collections;
mod custom_proton;
mod launcher;
mod models;
mod proton;
mod proton_downloader;
mod store;

use collections::{
    add_game_to_collection, create_collection, delete_collection, list_collections,
    remove_game_from_collection, rename_collection,
};
use custom_proton::{add_custom_proton_path, list_custom_proton_paths, remove_custom_proton_path};
use launcher::launch_game;
use proton::{is_umu_installed, list_proton_versions};
use proton_downloader::{
    delete_proton_version, download_proton_version, list_available_proton_downloads,
};
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
            add_custom_proton_path,
            remove_custom_proton_path,
            list_custom_proton_paths,
            list_available_proton_downloads,
            download_proton_version,
            delete_proton_version,
            list_collections,
            create_collection,
            rename_collection,
            delete_collection,
            add_game_to_collection,
            remove_game_from_collection,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| format!("error while running tauri application: {}", e))
}
