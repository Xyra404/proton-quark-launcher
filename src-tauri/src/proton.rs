use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ─── Types ────────────────────────────────────────────────────────────────────

/// A discovered Proton installation ready to use with umu-launcher.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProtonInstall {
    /// Human-readable display name derived from the directory name
    /// (e.g. "GE-Proton9-27", "Proton 9.0").
    pub name: String,

    /// Canonicalised absolute path to the Proton installation directory
    /// (the folder that directly contains the `proton` executable).
    pub path: String,
}

// ─── Internal Helpers ─────────────────────────────────────────────────────────

/// Returns `true` only when `dir` contains a *file* (or symlink to one)
/// literally named `proton` that is executable.
fn has_proton_executable(dir: &Path) -> bool {
    let candidate = dir.join("proton");
    if !candidate.exists() {
        return false;
    }
    // Reject directories named "proton" (shouldn't happen, but be safe).
    if candidate.is_dir() {
        return false;
    }
    // On Linux: check that at least one execute bit is set.
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(&candidate) {
        Ok(meta) => meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

/// Scans every direct child of `dir` and collects those that pass
/// `has_proton_executable`. The optional `name_filter` predicate, when
/// supplied, must return `true` for a subfolder to be considered at all
/// (used to restrict `steamapps/common/` to Proton-named entries).
fn scan_dir(
    dir: &Path,
    name_filter: Option<&dyn Fn(&str) -> bool>,
    out: &mut Vec<ProtonInstall>,
    seen_paths: &mut HashSet<PathBuf>,
) {
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return, // directory missing or inaccessible — skip silently
    };

    for entry in read.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_owned(),
            None => continue,
        };

        // Apply caller-supplied name filter (e.g. must contain "proton").
        if let Some(filter) = name_filter {
            if !filter(&name) {
                continue;
            }
        }

        // Must actually contain a `proton` executable.
        if !has_proton_executable(&path) {
            continue;
        }

        // Resolve to canonical path for deduplication.
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.clone(),
        };

        if !seen_paths.insert(canonical.clone()) {
            continue; // already seen via a different search path / symlink
        }

        out.push(ProtonInstall {
            name,
            path: canonical.to_string_lossy().into_owned(),
        });
    }
}

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Discovers every Proton installation available on this system.
///
/// Search order (each directory is skipped if it doesn't exist):
/// 1. `~/.steam/steam/compatibilitytools.d/`                       — all subfolders
/// 2. `~/.local/share/Steam/steamapps/common/`                     — "Proton*" only
/// 3. `/usr/share/steam/compatibilitytools.d/`                     — all subfolders (system-wide)
/// 4. `~/.var/app/com.valvesoftware.Steam/…/steamapps/common/`     — "Proton*" only (Flatpak)
/// 5. `~/.var/app/com.valvesoftware.Steam/…/compatibilitytools.d/` — all subfolders (Flatpak)
/// 6. `/var/lib/flatpak/…/share/steam/compatibilitytools.d/`       — all subfolders (Flatpak system-wide)
#[tauri::command]
pub fn list_proton_versions() -> Result<Vec<ProtonInstall>, String> {
    let home = dirs::home_dir().ok_or("Could not resolve home directory")?;

    // Predicate that passes any subfolder whose name contains "proton"
    // (case-insensitive). Used for steamapps/common/ scans.
    let contains_proton = |name: &str| name.to_ascii_lowercase().contains("proton");

    let search_paths: &[(&Path, Option<&dyn Fn(&str) -> bool>)] = &[
        // 1. Native Steam custom tools
        (
            &home.join(".steam/steam/compatibilitytools.d"),
            None, // accept everything
        ),
        // 2. Native Steam built-in Proton versions
        (
            &home.join(".local/share/Steam/steamapps/common"),
            Some(&contains_proton),
        ),
        // 3. Flatpak Steam built-in Proton versions
        (
            &home.join(
                ".var/app/com.valvesoftware.Steam/\
                 .local/share/Steam/steamapps/common",
            ),
            Some(&contains_proton),
        ),
        // 4. Flatpak Steam custom tools
        (
            &home.join(
                ".var/app/com.valvesoftware.Steam/\
                 .local/share/Steam/compatibilitytools.d",
            ),
            None,
        ),
        // 5. System-wide native tools
        (
            Path::new("/usr/share/steam/compatibilitytools.d"),
            None,
        ),
        // 6. System-wide Flatpak tools
        (
            Path::new("/var/lib/flatpak/app/com.valvesoftware.Steam/current/active/files/share/steam/compatibilitytools.d"),
            None,
        ),
    ];

    let mut installs: Vec<ProtonInstall> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();

    for (dir, filter) in search_paths {
        scan_dir(dir, *filter, &mut installs, &mut seen);
    }

    // Sort alphabetically by name for stable UI ordering.
    installs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(installs)
}

/// Returns `true` if `umu-run` is present and executable on the current PATH.
///
/// Implemented manually by walking PATH entries to avoid adding the `which` crate.
#[tauri::command]
pub fn is_umu_installed() -> bool {
    let path_var = match std::env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };

    use std::os::unix::fs::PermissionsExt;

    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join("umu-run");
        if let Ok(meta) = std::fs::metadata(&candidate) {
            // Must be a file (or symlink to one) with at least one execute bit.
            if !meta.is_dir() && meta.permissions().mode() & 0o111 != 0 {
                return true;
            }
        }
    }

    false
}
