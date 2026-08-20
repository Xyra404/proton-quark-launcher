export type GamePlatform = 'Windows' | 'Linux';

/**
 * Mirrors `src-tauri/src/models.rs` → `Game` exactly.
 * All Rust `Option<String>` fields are `?: string` here.
 */
export interface Game {
  /** UUID v4 stable identifier */
  id: string;
  /** Human-readable game name */
  name: string;
  /** Target platform (defaults to Windows if omitted) */
  platform?: GamePlatform;
  /** Absolute path to the game executable */
  exe_path: string;
  /** Display name of the selected Proton build, e.g. "GE-Proton9-27" (None for Linux) */
  proton_version?: string;
  /** Absolute path to the Proton installation directory (None for Linux) */
  proton_path?: string;
  /** Optional explicit Wine prefix directory */
  prefix_path?: string;
  /** Optional extra CLI arguments appended at launch */
  launch_args?: string;
  /** ISO 8601 UTC timestamp of when the game was added to the library */
  date_added?: string;
  /** ISO 8601 UTC timestamp of the last launch, or undefined if never played */
  last_played?: string;
  /** Total playtime in seconds */
  total_playtime_seconds: number;
  /** Toggle for MangoHud performance overlay & FPS limiter */
  enable_mangohud?: boolean;
  /** Toggle for Feral GameMode CPU/GPU performance optimizer */
  enable_gamemode?: boolean;
  /** Toggle for Gamescope micro-compositor */
  enable_gamescope?: boolean;
}

/**
 * Mirrors `src-tauri/src/proton.rs` → `ProtonInstall` exactly.
 */
export interface ProtonInstall {
  /** Directory name used as the display label, e.g. "GE-Proton9-27" */
  name: string;
  /** Canonicalised absolute path to the Proton installation directory */
  path: string;
}

/**
 * Mirrors `src-tauri/src/proton_downloader.rs` → `ProtonRelease` exactly.
 */
export interface ProtonRelease {
  /** Release tag, e.g. "GE-Proton9-27" */
  tag: string;
  /** URL to the .tar.gz download asset */
  download_url: string;
  /** Size of the archive in bytes, if available */
  size_bytes?: number;
  /** ISO 8601 timestamp of when this release was published */
  published_at: string;
}

/**
 * Mirrors `src-tauri/src/models.rs` → `Collection` exactly.
 */
export interface Collection {
  /** Stable unique identifier (UUID v4). */
  id: string;
  /** Human-readable display name of the collection (e.g. "Favorites"). */
  name: string;
  /** List of game IDs belonging to this collection. */
  game_ids: string[];
}
