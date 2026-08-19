use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// Key used inside the JSON store file.
const PATHS_KEY: &str = "custom_paths";

/// Filename used for the custom proton paths store, resolved relative to the app data dir.
const STORE_FILE: &str = "custom_proton_paths.json";

/// Loads the store, returning the list of registered custom Proton folder paths.
/// Initialises an empty list on first run.
pub(crate) fn load_custom_proton_paths(app: &AppHandle) -> Result<Vec<String>, String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to open custom proton store: {e}"))?;

    match store.get(PATHS_KEY) {
        Some(raw) => serde_json::from_value::<Vec<String>>(raw)
            .map_err(|e| format!("Failed to deserialise custom proton paths: {e}")),
        None => Ok(Vec::new()),
    }
}

/// Saves the given custom paths list back to the store and flushes it to disk.
pub(crate) fn save_custom_proton_paths(app: &AppHandle, paths: &[String]) -> Result<(), String> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("Failed to open custom proton store: {e}"))?;

    let value = serde_json::to_value(paths)
        .map_err(|e| format!("Failed to serialise custom proton paths: {e}"))?;

    store.set(PATHS_KEY, value);
    store
        .save()
        .map_err(|e| format!("Failed to persist custom proton store to disk: {e}"))
}

/// Validates that `path_str` exists, is a directory, and contains an executable `proton` binary.
/// Returns the canonical absolute path string on success.
fn validate_proton_dir(path_str: &str) -> Result<String, String> {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return Err("Proton directory path cannot be empty.".to_owned());
    }

    let path = Path::new(trimmed);
    if !path.exists() {
        return Err(format!("The path does not exist: '{trimmed}'"));
    }

    if !path.is_dir() {
        return Err(format!("The path is not a directory: '{trimmed}'"));
    }

    let candidate = path.join("proton");
    if !candidate.exists() {
        return Err(format!(
            "No 'proton' binary found in '{trimmed}'. Please select the root directory of a Proton installation."
        ));
    }

    if candidate.is_dir() {
        return Err(format!(
            "'proton' inside '{trimmed}' is a directory, not an executable file."
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(&candidate) {
            Ok(meta) => {
                if meta.permissions().mode() & 0o111 == 0 {
                    return Err(format!(
                        "The 'proton' binary in '{trimmed}' is not executable. Please check file permissions."
                    ));
                }
            }
            Err(e) => {
                return Err(format!(
                    "Failed to read metadata for 'proton' binary in '{trimmed}': {e}"
                ));
            }
        }
    }

    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve absolute path for '{trimmed}': {e}"))?;

    Ok(canonical.to_string_lossy().into_owned())
}

// ─── Tauri Commands ──────────────────────────────────────────────────────────

/// Adds a new custom Proton installation folder path after validation.
#[tauri::command]
pub fn add_custom_proton_path(app: AppHandle, path: String) -> Result<(), String> {
    let canonical_path = validate_proton_dir(&path)?;
    let mut paths = load_custom_proton_paths(&app)?;

    if paths.iter().any(|p| {
        Path::new(p)
            .canonicalize()
            .map(|c| c.to_string_lossy() == canonical_path)
            .unwrap_or(p == &canonical_path)
    }) {
        return Err(format!(
            "Custom Proton path '{}' is already registered.",
            canonical_path
        ));
    }

    paths.push(canonical_path);
    save_custom_proton_paths(&app, &paths)
}

/// Removes a custom Proton installation folder path.
#[tauri::command]
pub fn remove_custom_proton_path(app: AppHandle, path: String) -> Result<(), String> {
    let mut paths = load_custom_proton_paths(&app)?;
    let before = paths.len();

    let target_canonical = Path::new(&path)
        .canonicalize()
        .map(|c| c.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.clone());

    paths.retain(|p| {
        let p_canonical = Path::new(p)
            .canonicalize()
            .map(|c| c.to_string_lossy().into_owned())
            .unwrap_or_else(|_| p.clone());
        p != &path && p_canonical != target_canonical
    });

    if paths.len() == before {
        return Err(format!("Custom Proton path '{path}' was not found."));
    }

    save_custom_proton_paths(&app, &paths)
}

/// Lists all registered custom Proton installation folder paths.
#[tauri::command]
pub fn list_custom_proton_paths(app: AppHandle) -> Result<Vec<String>, String> {
    load_custom_proton_paths(&app)
}
