import { invoke } from '@tauri-apps/api/core';
import type { Game } from './games';

/**
 * Launches a game through umu-launcher (preferred) or bare Proton (fallback).
 *
 * The command returns as soon as the child process is successfully spawned —
 * it does NOT wait for the game to exit. `last_played` is updated in the store
 * automatically on the Rust side after a successful spawn.
 *
 * @throws {string} One of the following distinct error messages:
 *   - "Game executable not found on disk: …"
 *   - "Proton installation directory not found: …"
 *   - "Cannot launch '…': neither 'umu-run' was found on PATH nor the Proton binary exists at …"
 *   - "Failed to spawn launcher process for '…': …"
 */
export async function launchGame(game: Game): Promise<void> {
  await invoke<void>('launch_game', { game });
}
