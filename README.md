# Proton Quark Launcher

A lightweight Linux desktop launcher for Windows games through Proton, plus native Linux games. Add your games once, choose how they should run, and launch them from one library.

Built with SvelteKit, TypeScript, Tauri, and Rust. Game settings are stored locally, and Proton installations are discovered from Steam, Flatpak, system locations, and registered custom folders.

## Features

- Add, edit, and remove Windows or native Linux games.
- Detect Steam, Flatpak, system, and custom Proton installations, including GE-Proton.
- Browse and download available GE-Proton releases from the settings panel.
- Launch through `umu-launcher` when available, with a raw Proton fallback.
- Configure per-game Wine prefixes, launch arguments, MangoHud, GameMode, and Gamescope.
- Organize games into custom collections, including an uncategorized view.
- Track last-played time and total playtime, see running games, and force-quit them.
- Keep per-game launch logs in the application data directory.

## Requirements

- Linux
- Node.js and npm
- Rust and Cargo
- Tauri's Linux development dependencies
- Steam or another source of Proton builds for Windows games
- `umu-launcher` is recommended for the best compatibility, but raw Proton is used as a fallback.
- Optional: MangoHud, Feral GameMode, and Gamescope for the matching per-game options

Tauri's platform prerequisites are listed in the [Tauri Linux prerequisites guide](https://tauri.app/start/prerequisites/).

## Development

Install the JavaScript dependencies:

```sh
npm install
```

Run the frontend in a browser:

```sh
npm run dev
```

Run the desktop application with Tauri:

```sh
npm run tauri dev
```

Check the Svelte and TypeScript code:

```sh
npm run check
```

## Build

Build the frontend:

```sh
npm run build
```

Build installable Tauri bundles for the current platform:

```sh
npm run tauri build
```

The generated bundles are placed under `src-tauri/target/release/bundle/`.

## Using the launcher

1. Start the application.
2. Choose **Add Game** and select a Windows or Linux executable.
3. For Windows games, choose an installed Proton version.
4. Open **Advanced options** to set a Wine prefix, launch arguments, or performance tools.
5. Launch the game from the library.

Use the sidebar to create and manage collections. Open **Settings** to install or remove downloadable GE-Proton versions and register Proton folders outside the standard Steam locations.

The launcher checks that the executable and selected Proton installation still exist before starting a game. Native Linux games must be executable files. If `umu-launcher` is not installed, Windows games use the raw Proton fallback and the app shows a status notice.

## Project layout

- `src/` contains the SvelteKit frontend and UI components.
- `src/lib/api/` contains the frontend API wrappers.
- `src-tauri/src/` contains the Rust launcher, Proton discovery, storage, and Tauri commands.
- `src-tauri/target/` contains generated Rust build output and is not source code.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).
