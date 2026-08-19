use serde::{Deserialize, Serialize};

/// Supported target platforms for a game.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum GamePlatform {
    #[default]
    Windows,
    Linux,
}

/// Represents a manually added game managed by Proton Quark Launcher.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Game {
    /// Stable unique identifier (UUID v4), generated once at creation time.
    pub id: String,

    /// Human-readable display name of the game (e.g. "Half-Life 2").
    pub name: String,

    /// Target platform for this game (defaults to Windows for backwards compatibility).
    #[serde(default)]
    pub platform: GamePlatform,

    /// Absolute path to the game executable.
    pub exe_path: String,

    /// Display name of the selected Proton build (e.g. "GE-Proton9-27").
    /// `None` for Linux native games.
    pub proton_version: Option<String>,

    /// Absolute path to the Proton installation directory.
    /// `None` for Linux native games.
    pub proton_path: Option<String>,

    /// Optional explicit Wine prefix directory.
    /// If `None`, umu-launcher uses its own managed prefix.
    pub prefix_path: Option<String>,

    /// Optional extra arguments appended to the game launch command.
    pub launch_args: Option<String>,

    /// ISO 8601 timestamp of the last launch, e.g. "2025-08-19T12:00:00Z".
    /// `None` if the game has never been launched.
    pub last_played: Option<String>,
}

/// A user-defined collection grouping multiple games together.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Collection {
    /// Stable unique identifier (UUID v4).
    pub id: String,

    /// Human-readable display name of the collection (e.g. "Favorites").
    pub name: String,

    /// List of game IDs belonging to this collection.
    pub game_ids: Vec<String>,
}
