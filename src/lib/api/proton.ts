import { invoke } from '@tauri-apps/api/core';

// ─── Types ────────────────────────────────────────────────────────────────────

/**
 * Mirrors the Rust `ProtonInstall` struct in `src-tauri/src/proton.rs`.
 */
export interface ProtonInstall {
  /**
   * Human-readable display name derived from the directory name,
   * e.g. "GE-Proton9-27" or "Proton 9.0".
   *
   * Store this as `Game.proton_version` — it is the *label* shown in the UI
   * and logged in play history. It is stable across reboots.
   */
  name: string;

  /**
   * Canonicalised absolute path to the Proton directory (the folder that
   * directly contains the `proton` executable), e.g.
   * "/home/user/.local/share/Steam/compatibilitytools.d/GE-Proton9-27".
   *
   * Store this as `Game.proton_path` — it is what the launch command
   * (`PROTONPATH=...`) actually consumes. If the user reinstalls Proton,
   * the path should be re-validated, but the name remains the same.
   */
  path: string;
}

// ─── API Functions ─────────────────────────────────────────────────────────────

/**
 * Returns every Proton installation found across native Steam, Flatpak Steam,
 * and custom compatibilitytools.d directories.
 *
 * Results are sorted alphabetically by name and deduplicated by resolved path.
 * Returns an empty array (not an error) when no installs are found.
 *
 * @throws {string} Only if the home directory cannot be resolved.
 */
export async function listProtonVersions(): Promise<ProtonInstall[]> {
  return invoke<ProtonInstall[]>('list_proton_versions');
}

/**
 * Returns `true` if `umu-run` is found and executable on the current PATH.
 *
 * Use this on app startup to gate the launch UI — if `false`, show a warning
 * that umu-launcher must be installed before games can be launched.
 *
 * Never throws; returns `false` on any failure.
 */
export async function isUmuInstalled(): Promise<boolean> {
  return invoke<boolean>('is_umu_installed');
}
