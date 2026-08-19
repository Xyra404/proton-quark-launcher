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
  /** ISO 8601 UTC timestamp of the last launch, or undefined if never played */
  last_played?: string;
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
