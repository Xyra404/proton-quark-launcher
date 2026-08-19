# Proton Quark Launcher

Proton Quark Launcher is a lightweight Linux desktop app for launching Windows games with Proton. Add games manually, choose the Proton installation to use, and start each game from one place.

The app is built with SvelteKit, TypeScript, Tauri, and Rust. It automatically looks for Proton installations in common Steam, custom compatibility-tool, and Flatpak locations. Game settings are saved locally, and each game can use its own Wine prefix and launch arguments.

## Features

- Add and remove Windows games by selecting their executable files.
- Detect installed Proton and custom Proton builds, including GE-Proton.
- Launch games through `umu-launcher` when it is available.
- Fall back to running Proton directly when `umu-launcher` is not installed.
- Configure a per-game Wine prefix and optional launch arguments.
- Keep launch logs in the application data directory.

## Requirements

- Linux
- Node.js and npm
- Rust and Cargo
- Tauri's Linux development dependencies
- Steam or another source of Proton builds
- `umu-launcher` is recommended for the best compatibility, but the app can use raw Proton as a fallback.

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

## Adding a game

1. Start the application.
2. Choose **Add game**.
3. Select the game's Windows executable.
4. Choose an installed Proton version.
5. Optionally set a custom Wine prefix or launch arguments.
6. Launch the game from the game list.

The launcher checks that both the executable and selected Proton installation still exist before starting a game.

## Project layout

- `src/` contains the SvelteKit frontend and UI components.
- `src/lib/api/` contains the frontend API wrappers.
- `src-tauri/src/` contains the Rust launcher, Proton discovery, storage, and Tauri commands.
- `src-tauri/target/` contains Rust build output and is not source code.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).
