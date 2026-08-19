import { invoke } from '@tauri-apps/api/core';

// ─── Types ────────────────────────────────────────────────────────────────────

/**
 * Mirrors the Rust `Game` struct in `src-tauri/src/models.rs` exactly.
 * All optional Rust fields (`Option<String>`) become optional TypeScript fields.
 */
export interface Game {
  /** UUID v4 stable identifier — generated on the Rust side via `uuid::Uuid::new_v4()`. */
  id: string;
  /** Human-readable display name, e.g. "Half-Life 2". */
  name: string;
  /** Absolute path to the Windows .exe, e.g. "/mnt/games/hl2/hl2.exe". */
  exe_path: string;
  /** Display name of the selected Proton build, e.g. "GE-Proton9-27". */
  proton_version: string;
  /** Absolute path to the Proton installation directory. */
  proton_path: string;
  /** Optional explicit Wine prefix directory. */
  prefix_path?: string;
  /** Optional extra arguments appended to the launch command. */
  launch_args?: string;
  /** ISO 8601 timestamp of the last launch, e.g. "2025-08-19T12:00:00Z". */
  last_played?: string;
}

// ─── API Functions ────────────────────────────────────────────────────────────

/**
 * Adds a new game to the persistent store.
 * The caller must supply a pre-generated UUID v4 `id`.
 *
 * @throws {string} Error message from Rust if the game already exists or the store fails.
 */
export async function addGame(game: Game): Promise<void> {
  await invoke<void>('add_game', { game });
}

/**
 * Returns the full list of persisted games, or an empty array on first run.
 *
 * @throws {string} Error message from Rust if the store cannot be read.
 */
export async function listGames(): Promise<Game[]> {
  return invoke<Game[]>('list_games');
}

/**
 * Removes the game identified by `id` from the persistent store.
 *
 * @throws {string} Error message from Rust if no game with that id exists.
 */
export async function removeGame(id: string): Promise<void> {
  await invoke<void>('remove_game', { id });
}

/**
 * Replaces an existing game entry (matched by `game.id`) with the updated struct.
 * Use this for editing name, proton version, paths, or launch args.
 *
 * @throws {string} Error message from Rust if no game with that id exists.
 */
export async function updateGame(game: Game): Promise<void> {
  await invoke<void>('update_game', { game });
}

// ─── Utility ──────────────────────────────────────────────────────────────────

/**
 * Generates a UUID v4 string on the frontend using the Web Crypto API.
 * Avoids a round-trip to Rust just for ID generation.
 */
export function generateId(): string {
  return crypto.randomUUID();
}

/**
 * Returns the current UTC time as an ISO 8601 string, suitable for `last_played`.
 */
export function nowIso(): string {
  return new Date().toISOString();
}
