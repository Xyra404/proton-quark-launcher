import { invoke } from '@tauri-apps/api/core';
import type { Game, ProtonInstall } from './types';

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

