import { invoke } from '@tauri-apps/api/core';
import type { Game, ProtonInstall, ProtonRelease, Collection } from './types';

/**
 * Thin invoke wrapper: runs `fn`, catches any Tauri/Rust error string,
 * and rethrows it as a proper `Error` so callers can `catch (e)` with `e.message`.
 */
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    // Tauri surfaces Rust `Err(String)` as a plain string rejection.
    throw new Error(typeof e === 'string' ? e : String(e));
  }
}

// ─── Game CRUD ───────────────────────────────────────────────────────────────

export async function listGames(): Promise<Game[]> {
  return call<Game[]>('list_games');
}

export async function addGame(game: Game): Promise<void> {
  return call<void>('add_game', { game });
}

export async function removeGame(id: string): Promise<void> {
  return call<void>('remove_game', { id });
}

export async function updateGame(game: Game): Promise<void> {
  return call<void>('update_game', { game });
}

// ─── Proton Discovery ────────────────────────────────────────────────────────

export async function listProtonVersions(): Promise<ProtonInstall[]> {
  return call<ProtonInstall[]>('list_proton_versions');
}

export async function isUmuInstalled(): Promise<boolean> {
  return call<boolean>('is_umu_installed');
}

// ─── Launcher ────────────────────────────────────────────────────────────────

export async function launchGame(game: Game): Promise<void> {
  return call<void>('launch_game', { game });
}

// ─── Custom Proton Paths ────────────────────────────────────────────────────

export async function listCustomProtonPaths(): Promise<string[]> {
  return call<string[]>('list_custom_proton_paths');
}

export async function addCustomProtonPath(path: string): Promise<void> {
  return call<void>('add_custom_proton_path', { path });
}

export async function removeCustomProtonPath(path: string): Promise<void> {
  return call<void>('remove_custom_proton_path', { path });
}

// ─── Proton Downloader ───────────────────────────────────────────────────────

export async function listAvailableProtonDownloads(): Promise<ProtonRelease[]> {
  return call<ProtonRelease[]>('list_available_proton_downloads');
}

export async function downloadProtonVersion(release: ProtonRelease): Promise<void> {
  return call<void>('download_proton_version', { release });
}

export async function deleteProtonVersion(path: string): Promise<void> {
  return call<void>('delete_proton_version', { path });
}

// ─── Collections ───────────────────────────────────────────────────────────────

export async function listCollections(): Promise<Collection[]> {
  return call<Collection[]>('list_collections');
}

export async function createCollection(name: string): Promise<Collection> {
  return call<Collection>('create_collection', { name });
}

// Tauri v2 auto-converts Rust snake_case params to camelCase on the JS side.
// The Rust command is `rename_collection(id: String, new_name: String)`, so
// the expected invoke() key is `newName`, not `new_name`.
export async function renameCollection(id: string, newName: string): Promise<void> {
  return call<void>('rename_collection', { id, newName });
}

export async function deleteCollection(id: string): Promise<void> {
  return call<void>('delete_collection', { id });
}

// Rust command is `add_game_to_collection(collection_id: String, game_id: String)`
// → expected invoke() keys are `collectionId` and `gameId`.
export async function addGameToCollection(collectionId: string, gameId: string): Promise<void> {
  return call<void>('add_game_to_collection', { collectionId, gameId });
}

// Same fix: `remove_game_from_collection(collection_id: String, game_id: String)`
// → expected invoke() keys are `collectionId` and `gameId`.
export async function removeGameFromCollection(collectionId: string, gameId: string): Promise<void> {
  return call<void>('remove_game_from_collection', { collectionId, gameId });
}
