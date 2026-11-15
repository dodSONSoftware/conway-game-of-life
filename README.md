![Game of Life glider icon and wordmark](docs/game-of-life-wordmark.png)

# Game of Life

**Release:** Aluminum Ferret — version 0.2.0.

A native desktop cellular automaton simulator written in Rust with Bevy and egui. Explore a sparse, editable world, stamp classic Life patterns, and visualize predicted births and deaths alongside two generations of history.

The runnable Rust crate is in [`game-of-life-bevy/`](game-of-life-bevy/); the repository root is not a Cargo workspace. The application starts with an **empty, paused board at generation 0**, using Conway's Life.

## Install Game of Life

Choose the version 0.2.0 installer for your computer.

| Platform | Installer |
| --- | --- |
| macOS on Apple Silicon (M-series) | [Download the Apple Silicon package](https://github.com/dodSONSoftware/conway-game-of-life/releases/download/v0.2.0/Game-of-Life-0.2.0-Apple-Silicon.pkg) |
| macOS on Intel | [Download the Intel Mac package](https://github.com/dodSONSoftware/conway-game-of-life/releases/download/v0.2.0/Game-of-Life-0.2.0-Apple-Intel-Silicon.pkg) |
| Windows x64 (Intel or AMD) | [Download Windows Setup](https://github.com/dodSONSoftware/conway-game-of-life/releases/download/v0.2.0/Game-of-Life-0.2.0-Windows-x64-Setup.exe) |

Download [SHA256SUMS](https://github.com/dodSONSoftware/conway-game-of-life/releases/download/v0.2.0/SHA256SUMS) to the same folder as your installer. Before opening the installer, check it from that folder:

- **macOS (Terminal):** Run `shasum -a 256 -c --ignore-missing SHA256SUMS`. Confirm that your downloaded package is listed with `OK`.
- **Windows (PowerShell):** Run `Get-FileHash .\Game-of-Life-0.2.0-Windows-x64-Setup.exe -Algorithm SHA256`. Compare the full hash with the line for that filename in `SHA256SUMS`. Uppercase and lowercase letters are equivalent.

On **macOS**, open the downloaded package, follow the installer prompts, then launch **Game of Life** from Applications. If you’re unsure which Mac package to choose, open **Apple menu → About This Mac** and check the chip or processor.

On **Windows**, run Setup, then open **Game of Life** from the Start menu. Setup installs the app and its bundled patterns for your Windows user account.

These installers are unsigned. macOS and Windows may display a security warning when you open them.

## Build and run

Requirements:

- A Rust toolchain supporting edition 2024 and the locked dependencies. No minimum Rust version is declared or toolchain version pinned.
- A desktop display and graphics environment supported by Bevy.
- Platform development tools and native libraries. On macOS, install Xcode Command Line Tools. Debian/Ubuntu setup notes are in the [crate README](game-of-life-bevy/README.md#build-and-run-debian--ubuntu).

From the repository root:

```sh
cd game-of-life-bevy
cargo build --locked
cargo run --locked --release
```

On macOS, launch the app bundle to use the glider icon in the Dock. Run these commands from `game-of-life-bevy/`:

```sh
./scripts/package-macos.sh
open "target/release/Game of Life.app"
```

To build the Windows x64 installer from macOS, install the MinGW-w64 and NSIS tools, then run:

```sh
rustup target add x86_64-pc-windows-gnu
./scripts/package-windows.sh
```

Run the script from `game-of-life-bevy/`. It writes the versioned setup executable to the repository’s `installers/` directory. Setup installs Game of Life for the current Windows user, adds a Start menu shortcut, and places the bundled patterns in `assets/patterns/` beside the executable.

For development, use `cargo run --locked`. Cargo downloads dependencies on the first build; downloading additional pattern packs is optional.

The manifest pins Bevy to `0.20.0` and `bevy_egui` to `0.43.1`. Other direct dependencies are `serde`, `toml`, and `rfd`; `Cargo.lock` records resolved versions.

## Quick start

1. Select **Glider**, **Blinker**, or **Gosper glider gun** in the Pattern Library.
2. Click the field to place it. The pattern stays selected for repeated stamping; press **Esc** to return to drawing.
3. Press **Space** to run or pause. Use **Step** or **+10** to pause and advance a fixed number of generations.
4. Press **Enter** to fit all living cells, scroll to zoom toward the pointer, and drag the middle mouse button to pan.
5. Open **Settings (F3)** to adjust colors, shapes, history trails, and random drawing density.

Drawing, erasing, random-region editing, and stamping can be used during playback.

## Rules

Open **Rules (F4)** to select an automaton:

| Rule | Birth / survival or state transition |
| --- | --- |
| Conway's Life | B3/S23 |
| HighLife | B36/S23 |
| Seeds | B2/S; no living cell survives |
| Day & Night | B3678/S34678 |
| Brian's Brain | OFF cells with exactly two ON neighbors become ON; ON → refractory → OFF |

**Selecting a different rule clears the board, history, refractory state, and saved snapshot, then pauses at generation 0.** Selecting the active rule leaves the simulation unchanged. Imports supply seed coordinates without changing the active rule. The selected rule is session state and is not saved in preferences.

## Controls

These are factory defaults; Run/Pause, speed keys, and Fit All can be reassigned in Settings.

| Action | Control |
| --- | --- |
| Run / pause | Space or toolbar |
| Advance one / ten generations and pause | Step / +10 |
| Increase / decrease speed preset | Up / Down |
| Snap to the next / previous whole-number speed | Shift + Up / Down |
| Fit all living cells | Enter or Fit All |
| Draw / erase | Left / right mouse drag |
| Randomized enlarged brush | Shift + left-drag |
| Erase with enlarged brush | Shift + right-drag |
| Erase an inclusive rectangle on release | Ctrl + left-drag |
| Randomize a rectangle on release | Alt + left-drag |
| Pan | Middle-button drag |
| Zoom toward the pointer | Mouse wheel over the field |
| Repeated pattern placement | Select a pattern, then click or left-drag in the field |
| Rotate / mirror active pattern | R / M |
| Cancel placement or rectangle selection; close dialogs | Esc |
| Help / Pattern Library / Settings / Rules | F1 / F2 / F3 / F4 |
| About | Click GAME OF LIFE |

Holding Shift, Alt, or Ctrl cancels the active stamp. Closing the Pattern Library also cancels placement. Stamping adds cells to the existing population. Drag spacing is half the pattern bounding-box diagonal plus the configured extra spacing, initially **15 cells**.

Requested speed ranges from **0.25 to 240 generations/sec**, initially **5**. The footer reports requested and measured speed separately; achieved speed depends on workload. Scrolling over the speed slider adjusts the requested speed by one GPS, within those limits.

**Randomize** replaces the entire population with a random rectangle centered on the viewport, resets generation/history, and fits the new population. Defaults are **120 × 90 cells at 30% density**. Alt-drag replaces only the selected rectangle and accepts up to **2,000,000 cell positions**. The Shift brush defaults to **5 × 5** and uses the same density; unsampled positions within its footprint are cleared.

**Clear** pauses and empties the universe at generation 0 while preserving the active rule. The toolbar has no Reset action.

## Forecast, history, and appearance

The simulation maintains current cells (`G`), previous cells (`G−1`), older cells (`G−2`), and a cached next-generation forecast. Historical layers affect visualization only.

- **Forecast:** classifies predicted births, survivors, and deaths between `G` and `G+1`.
- **History:** classifies births, survivors, and deaths between `G−1` and `G`.
- **History trails:** optionally show footprints from the previous two generations.
- **Show only currently living cells:** displays the current ON population using the Living style, hiding transitions and trails.

Factory appearance uses cyan circles for births, green filled cells for survivors, and translucent orange-red square frames for deaths. Settings provides independent colors, opacity, visibility, and seven cell shapes, plus background/grid controls. Brian's Brain refractory cells use the dying style in the full visualization.

The footer includes active rule, generation, living population, forecast births/deaths, net change, speed, zoom, and playback state. Net change is births minus deaths; its percentage is relative to the current living population and is `N/A` when that population is zero.

Manual edits retain prior history and mark it as edited. **Reset history to now** copies current cells into both historical layers without changing the current population or generation.

## Snapshots and outlier cleanup

**Snapshot** copies the complete simulation into one in-memory slot, including generation, rule, historical layers, forecast, and refractory state. A new snapshot replaces the previous one. **Restore Snapshot** pauses, restores that state, and fits the restored population. Snapshots disappear when the process exits; changing rules clears the slot.

**Erase Outliers** pauses playback and previews density-based cleanup. Red cells/frames identify proposed removals; green rectangles identify retained regions. Adjust detection radius, local population, retained-mass percentage, padding, multiple-mass retention, and whether forecast births participate in detection. Defaults are radius **15**, local population **5**, mass threshold **5%**, padding **8**, with both checkboxes enabled.

Use **Apply removal** or **Enter** to apply; **Cancel**, **Esc**, or the close button cancels. Applying fits the result; either outcome restores the prior playback state. Only current living cells are removed, and no cells are removed if no qualifying mass is detected. There is no dedicated undo; take a Snapshot before cleanup if needed. Analysis runs synchronously, so large populations may stall the UI.

## Pattern library and imports

The code embeds 15 starter patterns across still lifes, oscillators, spaceships, guns, and methuselahs. A sourced **Puffer 1** RLE file is also included. Search filters names and categories; selecting a pattern exposes its description, preview, and rotation/mirroring controls.

To import a local file:

1. Open **Patterns → Import Pattern…**.
2. Use **Browse…**, or enter an absolute path or a path relative to the process working directory.
3. Select **Import into library**, then place the pattern in the field.

The chooser runs off the UI thread; macOS uses a separate AppleScript chooser process. Coordinate-identical imports select the existing pattern rather than adding a duplicate. Imported library entries exist only for the current session.

Supported formats:

- **RLE (`.rle`):** two-state coordinates, declared width/height from 1 to 100,000, at most 2,000,000 living cells. The parser does not apply the file's rule declaration.
- **Macrocell (`.mc`):** Golly two-state `[M2]` quadtrees, expanded into sparse coordinates; explicit rules must describe Conway Life. Expansion is capped at 2,000,000 living cells and 3,000,000 nodes. Compressed `.mc.gz` and multistate Macrocell files are unsupported.

Macrocell import is not a Hashlife simulation backend. Large constructors may exceed expansion limits or be impractical to simulate.

### Optional catalog downloads

From the crate directory, with Python 3 and internet access:

```sh
python3 scripts/fetch_patterns.py
```

The script uses only the Python standard library. It reads `assets/patterns/catalog.json`, checks the [ConwayLife pattern directory](https://conwaylife.com/patterns/), and downloads available files into `assets/patterns/`. Existing matching files may be overwritten. Missing/invalid downloads are reported; restart the app to load installed assets.

Runtime loading uses the compiled-in `catalog.tsv` and reads matching files beneath the build-time `CARGO_MANIFEST_DIR`. Missing or unsupported assets are skipped. Keep JSON and TSV entries aligned when extending the catalog; changing TSV metadata requires rebuilding. Retain source attribution when redistributing patterns.

## Preferences

Settings save automatically when the Settings dialog closes. Appearance previews immediately; UI scale changes apply after closing the dialog. Factory UI scale is **1.35**. Existing saved preferences override defaults until **Restore defaults** is used.

Preferences are stored as `settings.toml`:

| Environment | Location |
| --- | --- |
| `XDG_CONFIG_HOME` set, on any platform | `$XDG_CONFIG_HOME/game-of-life-bevy/settings.toml` |
| macOS | `~/Library/Application Support/game-of-life-bevy/settings.toml` |
| Windows, with `APPDATA` set | `%APPDATA%\game-of-life-bevy\settings.toml` |
| Other / fallback | `~/.config/game-of-life-bevy/settings.toml` |

Missing or unreadable/invalid preferences fall back to defaults. Population, imported library entries, snapshots, viewport, selected rule, and outlier options are not persisted in this file.

## Development and tests

Run from `game-of-life-bevy/`:

```sh
cargo test --locked
```

The existing unit and integration tests cover Conway evolution, alternative rules, refractory state, history rotation, transition counts, snapshots, rule-switch resets, rectangle erasing, RLE/Macrocell parsing, pattern transforms, catalog presence, settings compatibility, speed controls, zoom anchoring, rendering geometry helpers, formatting, and outlier detection. These checks do not establish interactive desktop or cross-platform acceptance.

Simulation is CPU-based using sparse hash sets of signed 64-bit coordinates. Only current cells and their neighborhoods participate in evolution; cells outside the viewport continue to evolve. Rendering uses viewport-clipped egui painting through Bevy, without one ECS entity per cell. Playback advances at most eight generations per frame and limits accumulated lag.

## Repository layout

```text
.
├── README.md                       # Current repository overview and usage
├── GAME_OF_LIFE_DESIGN.md           # Original design specification
├── game-of-life-bevy-rules-f4.zip    # Source archive matching the included crate files
└── game-of-life-bevy/
    ├── Cargo.toml / Cargo.lock      # Rust manifest and resolved dependencies
    ├── README.md                    # Earlier usage notes and incremental updates
    ├── DESIGN.md                    # Copy of the original design specification
    ├── CHANGELOG.md                 # Development history
    ├── src/
    │   ├── main.rs                  # Bevy app, UI, input, painting, dialogs, playback
    │   ├── lib.rs                   # Public library modules
    │   ├── simulation.rs            # Rules, sparse world, history, forecasts, metrics
    │   ├── patterns.rs              # Catalog, parsers, encoder, transforms
    │   ├── settings.rs              # Preferences, defaults, TOML persistence
    │   └── outliers.rs              # Density analysis and removal-region bounds
    ├── tests/core.rs                # Integration tests; modules also contain unit tests
    ├── assets/patterns/             # JSON/TSV catalogs and bundled Puffer 1 RLE
    └── scripts/fetch_patterns.py    # Optional catalog downloader
```

The design documents describe broader goals and proposed architecture. Earlier README/changelog entries retain superseded controls and behavior; the current source defines actual behavior.

## Current scope

There is no general undo/redo, disk-based world save/load, export action in the UI, automatic persistence of imported patterns, Hashlife backend, chunked GPU simulation, or benchmark/CI configuration. An RLE encoder exists in the library but is not exposed as a desktop export action. Practical population size is limited by CPU time and memory; 64-bit simulation coordinates do not imply precise camera positioning at extreme distances because viewport calculations use floating-point values.
