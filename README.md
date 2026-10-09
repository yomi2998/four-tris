# four-tris
This is the source code for four-tris, an open source training tool for block-stacking games, built to allow you to quickly explore different situations and test different options and freely train in a Tetris-like environment.

This repository is a fork of [github.com/fiorescarlatto/four-tris](https://github.com/fiorescarlatto/four-tris).
It carries out-of-tree modifications, most notably a cross-platform Rust port by [github.com/yomi2998](https://github.com/yomi2998) (see `src/`),
and is distributed under the same GNU General Public License as the original, either version 3 of the License, or (at your option) any later version.
Contributions to this fork are offered under that same license.

## Creating custom Skins

You can add your own custom skins inside the `textures` folder.
All custom skins must follow these requirements:
-   Must be a .png file.
-   Must have a resolution of 300 x 30 or higher as long as it keeps the same aspect ratio.
-   Must have a bit depth of 32 (transparency)

You can follow this template for the placement of each different 'piece' (ZLOSIJT) and 'ghost piece'
The last black square represents the color of an empty cell.

<img src="https://i.imgur.com/8GRRW6f.png" alt="template skin" width="300">


## Reporting issues, suggestions, feedback, bugs

This fork ships the Rust port; the Rust version is what its issues cover.

1. Check [the issue tracker](https://github.com/yomi2998/four-tris/issues) for duplicates first.
2. Open a new issue here for anything about the Rust build (`src/`), the
   snapshot tool, backends (Wayland/X11), HiDPI, or crashes on Linux,
   Windows or macOS. Include OS + window backend, version (`Cargo.toml`
   `version` or release tag), and steps to reproduce.
3. Bugs in the original AutoIt/Windows client are not tracked here; that
   version lives in the [upstream repository](https://github.com/fiorescarlatto/four-tris)
   and its [Discord](https://discord.gg/UhbnyAzWfw).

## Building

### Rust version (cross-platform, recommended)

The AutoIt original has been ported to Rust. The Rust version runs natively on
Linux (Wayland, X11 and HiDPI displays) and on Windows.

-   You will need [Rust](https://rustup.rs/) (stable toolchain).
-   Run `git clone https://github.com/yomi2998/four-tris.git`
-   Run the game from the repository root so it can find `settings.ini`,
    `colors.ini`, `se/` and `textures/`:
    -   Debug build: `cargo run`
    -   Release build: `cargo run --release`

On Linux the sound output goes through your sound server (PipeWire, PulseAudio
or ALSA) via `rodio`, the window/input layer uses `winit` (Wayland and X11 are
both supported, HiDPI scale factors are picked up automatically), and all
rendering is done on the CPU with `softbuffer`.

Notes:

-   The game picks its window backend automatically (Wayland preferred, X11 as
    fallback). Set `FOUR_TRIS_BACKEND=x11` or `FOUR_TRIS_BACKEND=wayland` to
    force one.
-   The window content scales to fit the window size; `SCALE` in
    `settings.ini` stores the current zoom level, and the `RESET SIZE` button
    in settings restores it.
-   The snapshot tool works like a regular screenshot tool: on Wayland it
    uses `slurp` + `grim` so you can drag-select a region on any monitor
    (nothing is hidden, multi-monitor friendly, niri included), falling back
    to a `grim` full capture or the XDG screenshot portal when `slurp` is
    missing; on X11 (XWayland included) it captures the monitor under the
    cursor and lets you select the region in-app. If no capture method is
    available it will tell you so.
-   Clipboard game-state sharing is compatible with the original Windows
    client (the board payload uses the same LZNT1 wire format).
-   Game logic is covered by unit tests: `cargo test`.

### AutoIt version (Windows, legacy)

The original AutoIt client (`Tetris.au3` and its `lib/`, `ai/`, BASS
binaries, `icon.ico`, `Compression.rtf`) has been removed from this fork.
It is still available in the [upstream repository](https://github.com/fiorescarlatto/four-tris)
history, or on [the original author's Discord](https://discord.gg/UhbnyAzWfw)
as a compiled standalone executable.

### Code

The Rust port lives in `src/`:

-   `game.rs` – playfield, piece shapes, SRS kicks (+180°), bags, scoring, modes
-   `app.rs` – application state, input (DAS/ARR/soft drop), screen flows
-   `ui.rs` – HUD and settings drawing (ports of the original Draw* functions)
-   `render.rs` – software renderer (rects, alpha blends, XOR icon blits, text)
-   `gui.rs` – winit event loop and presentation
-   `state.rs` – undo/redo ring and clipboard state codec (base64 + LZNT1)
-   `config.rs` / `ini.rs` – settings and skin handling
-   `audio.rs` – sound effects via `rodio`
-   `snap.rs` – screen capture for the snapshot tool (slurp+grim, grim, XDG portal, X11)

For features and larger changes, open an issue here first to discuss the
approach; pull requests against this repository are welcome. Contributions
are offered under the same GPL-3.0-or-later license as the project.


## License
    Copyright (C) 2020  github.com/fiorescarlatto
    Rust port Copyright (C) 2026  github.com/yomi2998

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/gpl.html>.
