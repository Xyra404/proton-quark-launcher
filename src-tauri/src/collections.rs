use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

use crate::models::Collection;

const COLLECTIONS_KEY: &str = "collections";
const STORE_FILE: &str = "collections.json";

/// Loads the store, returning the current collection list.
fn load_collections(app: &AppHandle) -> Result<Vec<Collection>, String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to open collections store: {e}"))?;

    match store.get(COLLECTIONS_KEY) {
        Some(raw) => serde_json::from_value::<Vec<Collection>>(raw)
            .map_err(|e| format!("Failed to deserialise collections: {e}")),
        None => Ok(Vec::new()),
    }
}

/// Saves the given collection list back to the store and flushes it to disk.
fn save_collections(app: &AppHandle, collections: &[Collection]) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to open collections store: {e}"))?;

    let value = serde_json::to_value(collections)
        .map_err(|e| format!("Failed to serialise collections: {e}"))?;

    store.set(COLLECTIONS_KEY, value);
    store
        .save()
        .map_err(|e| format!("Failed to persist collections to disk: {e}"))
}

// ─── Tauri Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_collections(app: AppHandle) -> Result<Vec<Collection>, String> {
    load_collections(&app)
}

#[tauri::command]
pub fn create_collection(app: AppHandle, name: String) -> Result<Collection, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Collection name cannot be empty.".to_string());
    }

    let mut collections = load_collections(&app)?;

    if collections.iter().any(|c| c.name.eq_ignore_ascii_case(trimmed)) {
        return Err(format!("A collection named '{}' already exists.", trimmed));
    }

    let collection = Collection {
        id: Uuid::new_v4().to_string(),
        name: trimmed.to_string(),
        game_ids: Vec::new(),
    };

    collections.push(collection.clone());
    save_collections(&app, &collections)?;

    Ok(collection)
}

#[tauri::command]
pub fn rename_collection(app: AppHandle, id: String, new_name: String) -> Result<(), String> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err("Collection name cannot be empty.".to_string());
    }

    let mut collections = load_collections(&app)?;

    // Check for duplicates (excluding the current collection being renamed)
    if collections
        .iter()
        .any(|c| c.id != id && c.name.eq_ignore_ascii_case(trimmed))
    {
        return Err(format!("A collection named '{}' already exists.", trimmed));
    }

    let coll = collections
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("No collection found with id '{}'.", id))?;

    coll.name = trimmed.to_string();
    save_collections(&app, &collections)
}

#[tauri::command]
pub fn delete_collection(app: AppHandle, id: String) -> Result<(), String> {
    let mut collections = load_collections(&app)?;
    let before = collections.len();
    collections.retain(|c| c.id != id);

    if collections.len() == before {
        return Err(format!("No collection found with id '{id}'."));
    }

    save_collections(&app, &collections)
}

#[tauri::command]
pub fn add_game_to_collection(
    app: AppHandle,
    collection_id: String,
    game_id: String,
) -> Result<(), String> {
    let mut collections = load_collections(&app)?;

    let coll = collections
        .iter_mut()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| format!("No collection found with id '{}'.", collection_id))?;

    if !coll.game_ids.contains(&game_id) {
        coll.game_ids.push(game_id);
        save_collections(&app, &collections)?;
    }

    Ok(())
}

#[tauri::command]
pub fn remove_game_from_collection(
    app: AppHandle,
    collection_id: String,
    game_id: String,
) -> Result<(), String> {
    let mut collections = load_collections(&app)?;

    let coll = collections
        .iter_mut()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| format!("No collection found with id '{}'.", collection_id))?;

    let before = coll.game_ids.len();
    coll.game_ids.retain(|id| id != &game_id);

    if coll.game_ids.len() != before {
        save_collections(&app, &collections)?;
    }

    Ok(())
}

// ─── Internal API ────────────────────────────────────────────────────────────

/// Removes a game's ID from all collections. Called from `store.rs` when a game
/// is deleted entirely from the launcher to ensure collections don't reference
/// non-existent games.
pub(crate) fn remove_game_from_all_collections(
    app: &AppHandle,
    game_id: &str,
) -> Result<(), String> {
    let mut collections = load_collections(app)?;
    let mut changed = false;

    for coll in &mut collections {
        let before = coll.game_ids.len();
        coll.game_ids.retain(|id| id != game_id);
        if coll.game_ids.len() != before {
            changed = true;
        }
    }

    if changed {
        save_collections(app, &collections)?;
    }

    Ok(())
}
