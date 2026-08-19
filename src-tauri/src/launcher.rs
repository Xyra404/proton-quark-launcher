use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use chrono::Utc;
use tauri::{AppHandle, Manager};

use crate::models::{Game, GamePlatform};
use crate::proton::is_umu_installed;
use crate::store::{load_games, save_games};

// ─── Internal Helpers ─────────────────────────────────────────────────────────

/// Resolves and guarantees the wine prefix directory for this game.
/// Returns the `PathBuf` of the (possibly freshly created) prefix directory.
fn resolve_prefix(app: &AppHandle, game: &Game) -> Result<PathBuf, String> {
    if let Some(ref explicit) = game.prefix_path {
        let p = PathBuf::from(explicit);
        fs::create_dir_all(&p)
            .map_err(|e| format!("Failed to create explicit prefix directory '{}': {e}", explicit))?;
        return Ok(p);
    }

    // Default: <app_data_dir>/prefixes/<game.id>
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data directory: {e}"))?;

    let prefix = data_dir.join("prefixes").join(&game.id);
    fs::create_dir_all(&prefix)
        .map_err(|e| format!("Failed to create default prefix directory '{}': {e}", prefix.display()))?;

    Ok(prefix)
}

/// Opens (or creates) the per-game log file and returns a `File` handle
/// suitable for redirecting stdout/stderr.
/// The `logs/` directory is created if it doesn't exist.
fn open_log_file(app: &AppHandle, game: &Game) -> Result<File, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data directory: {e}"))?;

    let logs_dir = data_dir.join("logs");
    fs::create_dir_all(&logs_dir)
        .map_err(|e| format!("Failed to create logs directory '{}': {e}", logs_dir.display()))?;

    let log_path = logs_dir.join(format!("{}.log", game.id));
    File::options()
        .create(true)
        .append(true) // Don't wipe previous sessions; append is friendlier for debugging.
        .open(&log_path)
        .map_err(|e| format!("Failed to open log file '{}': {e}", log_path.display()))
}

/// Resolves the user's Steam installation root (used in the Proton fallback path).
/// Checks the native path first, then the Flatpak path.
fn resolve_steam_root() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let candidates = [
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
        home.join(
            ".var/app/com.valvesoftware.Steam\
             /.local/share/Steam",
        ),
    ];

    candidates.into_iter().find(|p| p.is_dir())
}

/// Splits the optional `launch_args` string on ASCII whitespace into individual
/// arguments, exactly as a shell would. Returns an empty vec if `None` or blank.
fn split_launch_args(args: &Option<String>) -> Vec<String> {
    match args {
        Some(s) => s.split_whitespace().map(str::to_owned).collect(),
        None => Vec::new(),
    }
}

/// Strips environment variables that AppImage's `AppRun` wrapper injects for
/// our OWN bundled runtime (Python, dynamic linker search paths, etc.).
///
/// Without this, any child process we spawn — like `umu-run`, which is
/// itself a separate, system-installed Python program — inherits our
/// AppImage's PYTHONHOME/PYTHONPATH/LD_LIBRARY_PATH pointing at paths
/// *inside our own squashfs mount* (e.g. `/tmp/.mount_XXXXXX/usr/`).
/// That breaks the child's Python interpreter immediately with errors like
/// `ModuleNotFoundError: No module named 'encodings'`, since it goes
/// looking for its standard library inside our app's mount point instead
/// of its real, system location.
///
/// This has no effect when running as a plain binary, .deb, or .rpm
/// install (those variables are never set in the first place) — it only
/// matters for the AppImage build, but it's always safe to call.
fn sanitize_child_env(cmd: &mut Command) {
    for var in [
        "PYTHONHOME",
        "PYTHONPATH",
        "LD_LIBRARY_PATH",
        "APPDIR",
        "APPIMAGE",
        "ARGV0",
        "OWD",
    ] {
        cmd.env_remove(var);
    }
}

// ─── Command ──────────────────────────────────────────────────────────────────

/// Launches a Windows game through umu-launcher (preferred) or a raw Proton
/// fallback, without blocking the Tauri event loop.
///
/// # Error cases — each produces a distinct message
/// - `exe_path` does not exist on disk
/// - `proton_path` does not exist on disk
/// - Neither `umu-run` nor `<proton_path>/proton` can be found
/// - `spawn()` itself fails (e.g. permission denied, binary not executable)
#[tauri::command]
pub async fn launch_game(app: AppHandle, game: Game) -> Result<(), String> {
    // ── Pre-flight checks ────────────────────────────────────────────────────

    let exe_path = Path::new(&game.exe_path);
    if !exe_path.exists() {
        return Err(format!(
            "Game executable not found on disk: '{}'. \
             Has the file been moved or deleted?",
            game.exe_path
        ));
    }

    let log_file = open_log_file(&app, &game)?;
    let log_stderr = log_file
        .try_clone()
        .map_err(|e| format!("Failed to duplicate log file handle: {e}"))?;

    let extra_args = split_launch_args(&game.launch_args);

    let (base_bin, base_args, env_vars) = if game.platform == GamePlatform::Linux {
        // ── Linux Native Launch ──────────────────────────────────────────────
        let metadata = fs::metadata(exe_path)
            .map_err(|e| format!("Failed to read metadata for '{}': {e}", game.exe_path))?;
            
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!(
                "The file '{}' is not marked as executable. \
                 Please ensure it has execute permissions (e.g. chmod +x).",
                game.exe_path
            ));
        }

        (game.exe_path.clone(), extra_args, vec![])
    } else {
        // ── Windows (Proton) Launch ──────────────────────────────────────────
        
        let proton_path = game.proton_path.as_deref().ok_or_else(|| {
            format!("Proton path is missing for Windows game '{}'.", game.name)
        })?;
        
        let proton_version = game.proton_version.as_deref().unwrap_or("Unknown");

        if !Path::new(proton_path).exists() {
            return Err(format!(
                "Proton installation directory not found: '{}'. \
                 The Proton version '{}' may have been uninstalled.",
                proton_path, proton_version
            ));
        }

        let prefix = resolve_prefix(&app, &game)?;

        if is_umu_installed() {
            // Primary: umu-run
            let mut args = vec![game.exe_path.clone()];
            args.extend(extra_args);
            let envs = vec![
                ("WINEPREFIX".to_string(), prefix.to_string_lossy().to_string()),
                ("PROTONPATH".to_string(), proton_path.to_string()),
            ];
            ("umu-run".to_string(), args, envs)
        } else {
            // Fallback: bare Proton binary
            let proton_bin = PathBuf::from(proton_path).join("proton");
            if !proton_bin.exists() {
                return Err(format!(
                    "Cannot launch '{}': neither 'umu-run' was found on PATH \
                     nor the Proton binary exists at '{}'. \
                     Install umu-launcher or verify your Proton installation.",
                    game.name,
                    proton_bin.display()
                ));
            }

            let steam_root = resolve_steam_root().ok_or_else(|| {
                "Proton fallback launch failed: could not locate a Steam installation \
                 directory (checked native and Flatpak paths)."
                    .to_owned()
            })?;

            let mut args = vec!["run".to_string(), game.exe_path.clone()];
            args.extend(extra_args);
            let envs = vec![
                ("STEAM_COMPAT_DATA_PATH".to_string(), prefix.to_string_lossy().to_string()),
                ("STEAM_COMPAT_CLIENT_INSTALL_PATH".to_string(), steam_root.to_string_lossy().to_string()),
            ];
            (proton_bin.to_string_lossy().to_string(), args, envs)
        }
    };

    // ── Apply MangoHud & Feral GameMode wrappers ────────────────────────────
    let mut final_bin = base_bin;
    let mut final_args = base_args;

    if game.enable_mangohud {
        final_args.insert(0, final_bin);
        final_bin = "mangohud".to_string();
    }

    if game.enable_gamemode {
        final_args.insert(0, final_bin);
        final_bin = "gamemoderun".to_string();
    }

    if game.enable_gamescope {
        final_args.insert(0, "--".to_string());
        final_args.insert(1, final_bin);
        final_bin = "gamescope".to_string();
    }

    let mut cmd = Command::new(&final_bin);
    cmd.args(&final_args);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    if game.enable_mangohud {
        cmd.env("MANGOHUD", "1");
    }

    // Strip AppImage-injected runtime variables (PYTHONHOME, PYTHONPATH,
    // LD_LIBRARY_PATH, etc.) so umu-run's/Proton's own Python and dynamic
    // linker aren't hijacked by our AppImage's mount paths. Applied to
    // whichever branch was chosen above, since both spawn external tools
    // that must use their own, correct runtime environment.
    sanitize_child_env(&mut cmd);

    // ── Redirect output to log file ──────────────────────────────────────────

    cmd.stdout(Stdio::from(log_file));
    cmd.stderr(Stdio::from(log_stderr));

    // Detach stdin so the child never blocks waiting for terminal input.
    cmd.stdin(Stdio::null());

    // ── Spawn (non-blocking) ─────────────────────────────────────────────────
    // .spawn() returns as soon as the OS hands the child its PID.
    // We intentionally drop the Child handle — the process runs independently.

    cmd.spawn().map_err(|e| {
        format!(
            "Failed to spawn launcher process for '{}': {e}. \
             Check file permissions and that the binary is executable.",
            game.name
        )
    })?;

    // ── Persist last_played timestamp ─────────────────────────────────────────

    let now_iso = Utc::now().to_rfc3339();
    let mut games = load_games(&app)?;

    if let Some(entry) = games.iter_mut().find(|g| g.id == game.id) {
        entry.last_played = Some(now_iso);
    }
    // If the game isn't in the store for some reason, don't error — the launch
    // already succeeded and we don't want to surface a confusing store error.

    let _ = save_games(&app, &games);

    Ok(())
}
