use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use chrono::Utc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::models::Game;

/// Key used inside the JSON store file.
const GAMES_KEY: &str = "games";

/// Filename used for the games store, resolved relative to the app data dir.
const STORE_FILE: &str = "games.json";

/// Loads the store, returning the current game list.
/// Initialises an empty list on first run.
pub(crate) fn load_games(app: &AppHandle) -> Result<Vec<Game>, String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to open store: {e}"))?;

    match store.get(GAMES_KEY) {
        Some(raw) => serde_json::from_value::<Vec<Game>>(raw)
            .map_err(|e| format!("Failed to deserialise games: {e}")),
        // First run – no key yet; treat as empty list.
        None => Ok(Vec::new()),
    }
}

/// Saves the given game list back to the store and flushes it to disk.
pub(crate) fn save_games(app: &AppHandle, games: &[Game]) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to open store: {e}"))?;

    let value =
        serde_json::to_value(games).map_err(|e| format!("Failed to serialise games: {e}"))?;

    store.set(GAMES_KEY, value);
    store
        .save()
        .map_err(|e| format!("Failed to persist store to disk: {e}"))
}

// ─── Tauri Commands ──────────────────────────────────────────────────────────

fn validate_game_executable(game: &Game) -> Result<(), String> {
    let path = Path::new(&game.exe_path);
    
    if !path.exists() {
        return Err(format!("Executable path does not exist: {}", game.exe_path));
    }

    match game.platform {
        crate::models::GamePlatform::Windows => {
            if !game.exe_path.to_lowercase().ends_with(".exe") {
                return Err("Windows games must use a .exe executable file.".to_string());
            }
        }
        crate::models::GamePlatform::Linux => {
            let metadata = std::fs::metadata(path)
                .map_err(|e| format!("Failed to read metadata for executable: {e}"))?;
            
            if metadata.permissions().mode() & 0o111 == 0 {
                return Err("Linux native game executable must have execute permissions (chmod +x).".to_string());
            }
        }
    }
    
    Ok(())
}

/// Adds a new game to the persistent store.
/// The caller is responsible for setting a unique `id` (UUID v4).
#[tauri::command]
pub fn add_game(app: AppHandle, mut game: Game) -> Result<(), String> {
    validate_game_executable(&game)?;

    let mut games = load_games(&app)?;

    if games.iter().any(|g| g.id == game.id) {
        return Err(format!("A game with id '{}' already exists.", game.id));
    }

    // Timestamp when this game was added to the library (set once at creation time)
    game.date_added = Utc::now().to_rfc3339();

    games.push(game);
    save_games(&app, &games)
}

/// Returns the full list of persisted games.
#[tauri::command]
pub fn list_games(app: AppHandle) -> Result<Vec<Game>, String> {
    load_games(&app)
}

/// Removes the game identified by `id`.
/// Returns an error if no game with that id exists.
#[tauri::command]
pub fn remove_game(app: AppHandle, id: String) -> Result<(), String> {
    let mut games = load_games(&app)?;
    let before = games.len();
    games.retain(|g| g.id != id);

    if games.len() == before {
        return Err(format!("No game found with id '{id}'."));
    }

    save_games(&app, &games)?;

    // Cleanup: Ensure this game is also removed from any collections it belonged to.
    crate::collections::remove_game_from_all_collections(&app, &id)?;

    Ok(())
}

/// Replaces an existing game (matched by `game.id`) with the updated struct.
/// Returns an error if no game with that id exists.
#[tauri::command]
pub fn update_game(app: AppHandle, mut game: Game) -> Result<(), String> {
    validate_game_executable(&game)?;

    let mut games = load_games(&app)?;

    let entry = games
        .iter_mut()
        .find(|g| g.id == game.id)
        .ok_or_else(|| format!("No game found with id '{}'.", game.id))?;

    // Preserve the original date_added value so it never changes after creation
    game.date_added = entry.date_added.clone();

    *entry = game;
    save_games(&app, &games)
}
