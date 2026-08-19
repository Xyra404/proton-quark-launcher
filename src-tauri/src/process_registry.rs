use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Payload sent to the frontend when a game process stops.
#[derive(Clone, Serialize, Deserialize)]
pub struct GameStoppedPayload {
    pub game_id: String,
    pub exit_success: bool,
    pub session_seconds: u64,
    pub total_playtime_seconds: u64,
}

pub fn accumulate_playtime_and_emit(
    app: &AppHandle,
    game_id: &str,
    started_at: DateTime<Utc>,
    exit_success: bool,
) {
    let session_seconds = (Utc::now() - started_at).num_seconds().max(0) as u64;
    let mut total_playtime_seconds = 0;

    if let Ok(mut games) = crate::store::load_games(app) {
        if let Some(game) = games.iter_mut().find(|g| g.id == game_id) {
            game.total_playtime_seconds += session_seconds;
            total_playtime_seconds = game.total_playtime_seconds;
        }
        let _ = crate::store::save_games(app, &games);
    }

    let _ = app.emit(
        "game-stopped",
        GameStoppedPayload {
            game_id: game_id.to_string(),
            exit_success,
            session_seconds,
            total_playtime_seconds,
        },
    );
}

pub struct RunningProcess {
    pub child: Child,
    #[allow(dead_code)]
    pub game_id: String,
    pub started_at: DateTime<Utc>,
    /// The PID of the immediate child, captured at spawn time via
    /// `child.id()`. Used as a fallback/secondary cleanup signal — see
    /// `wine_prefix` below for why this alone isn't enough for Proton games.
    pub pid: u32,
    /// The WINEPREFIX this game was launched with, if it's a Proton/Windows
    /// game. `None` for Linux-native games.
    ///
    /// This is the actual fix for force-quit on Proton games:
    /// pressure-vessel's sandboxing (srt-bwrap, pv-adverb) calls setsid()
    /// at nearly every layer of the launch chain, putting wineserver,
    /// winedevice.exe, and the game's own .exe into entirely separate
    /// process groups/sessions from the top-level umu-run process. A plain
    /// `kill(-pid, SIGKILL)` on the top-level PID never reaches any of
    /// them (confirmed via `ps --forest`: each layer gets its own PGID and
    /// SID). `wineserver -k` sidesteps this completely — Wine tracks every
    /// process under a given prefix through its own internal server,
    /// independent of OS process groups.
    ///
    /// For Linux-native games, there's no Wine layer and no sandboxing, so
    /// the plain process-group `kill()` below is sufficient on its own —
    /// this field being `None` skips the wineserver step entirely and
    /// falls straight through to that same kill() call as before.
    pub wine_prefix: Option<PathBuf>,
}

/// The global registry tracking all currently running games.
pub struct ProcessRegistry(pub Mutex<HashMap<String, RunningProcess>>);

impl Default for ProcessRegistry {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// Command: Check if a specific game is currently running.
#[tauri::command]
pub fn is_game_running(registry: State<'_, ProcessRegistry>, game_id: String) -> bool {
    registry.0.lock().unwrap().contains_key(&game_id)
}

/// Command: List all currently running game IDs (useful for UI initialization).
#[tauri::command]
pub fn list_running_game_ids(registry: State<'_, ProcessRegistry>) -> Vec<String> {
    registry.0.lock().unwrap().keys().cloned().collect()
}

/// Command: Force quit a running game.
///
/// For Proton/Windows games (`wine_prefix` is `Some`): primarily uses
/// `wineserver -k` scoped to the game's WINEPREFIX, which asks Wine's own
/// server to cleanly terminate every Windows process it's tracking for
/// that prefix — the actual game .exe, winedevice.exe helpers, xalia.exe,
/// etc. — regardless of what process group or session pressure-vessel's
/// sandboxing put them in.
///
/// For Linux-native games (`wine_prefix` is `None`): there's no Wine layer
/// to delegate to, so this skips straight to signaling the process group
/// directly, exactly like before — native games have no sandboxing forking
/// children into separate sessions, so a plain process-group SIGKILL on
/// the PID they were spawned with (via `process_group(0)` in launcher.rs)
/// reaches the whole tree just fine.
///
/// In both cases, the process-group SIGKILL also runs as a secondary
/// cleanup pass, to catch the umu-run/gamescope/mangohud wrapper processes
/// themselves — `wineserver -k` only terminates Wine-tracked Windows
/// processes, not the Linux-side wrapper chain that launched them, though
/// that chain typically exits on its own shortly after wineserver reports
/// the prefix is empty.
#[tauri::command]
pub fn force_quit_game(
    app: AppHandle,
    registry: State<'_, ProcessRegistry>,
    game_id: String,
) -> Result<(), String> {
    let mut map = registry.0.lock().unwrap();

    if let Some(mut rp) = map.remove(&game_id) {
        let pid = rp.pid as i32;

        if let Some(prefix) = &rp.wine_prefix {
            // Primary mechanism for Proton games: ask Wine itself to kill
            // every process it's tracking under this prefix.
            let _ = Command::new("wineserver")
                .arg("-k")
                .env("WINEPREFIX", prefix)
                .status();

            // Give the wrapper chain (umu-run -> pressure-vessel -> proton
            // script) a brief moment to notice wineserver has shut down
            // and exit on its own, since "waitforexitandrun" mode returns
            // once no Wine processes remain.
            std::thread::sleep(Duration::from_millis(800));
        }

        // Secondary/fallback cleanup: signal the top-level process's own
        // group directly.
        // - Linux-native games: this is the ONLY mechanism, unchanged from
        //   before, since `wine_prefix` is `None` and the block above is
        //   skipped entirely.
        // - Proton games: mainly catches umu-run/gamescope/mangohud if
        //   they haven't already exited on their own after the
        //   wineserver -k pass above.
        //
        // Safety: `kill()` with a negative PID is a standard POSIX signal
        // call; we're only ever passing a PID we captured ourselves at
        // spawn time, never user-controlled input.
        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }

        // Reap the immediate child directly via our own Child handle so it
        // doesn't linger as a zombie. Other processes in the group that
        // aren't our direct child get reparented to PID 1 (or the nearest
        // subreaper) once killed, which reaps them automatically — we
        // don't need to (and can't) wait() on processes we didn't spawn
        // ourselves.
        let _ = rp.child.wait();

        accumulate_playtime_and_emit(&app, &game_id, rp.started_at, false);

        Ok(())
    } else {
        Err(format!(
            "Game with ID {} is not currently running.",
            game_id
        ))
    }
}
