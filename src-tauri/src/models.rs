use serde::{Deserialize, Serialize};

/// Represents a manually added Windows game managed by Proton Quark Launcher.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Game {
    /// Stable unique identifier (UUID v4), generated once at creation time.
    pub id: String,

    /// Human-readable display name of the game (e.g. "Half-Life 2").
    pub name: String,

    /// Absolute path to the Windows executable (e.g. "/mnt/games/hl2.exe").
    pub exe_path: String,

    /// Display name of the selected Proton build (e.g. "GE-Proton9-27").
    pub proton_version: String,

    /// Absolute path to the Proton installation directory
    /// (i.e. the folder containing `proton` binary and `files/`).
    pub proton_path: String,

    /// Optional explicit Wine prefix directory.
    /// If `None`, umu-launcher uses its own managed prefix.
    pub prefix_path: Option<String>,

    /// Optional extra arguments appended to the game launch command.
    pub launch_args: Option<String>,

    /// ISO 8601 timestamp of the last launch, e.g. "2025-08-19T12:00:00Z".
    /// `None` if the game has never been launched.
    pub last_played: Option<String>,
}
