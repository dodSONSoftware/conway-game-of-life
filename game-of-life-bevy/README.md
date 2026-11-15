# Game of Life

Native desktop Conway B3/S23 simulator written in Rust with Bevy 0.20 and bevy_egui 0.43.1. It stores **G, G−1, G−2**, computes a noncommitted G+1 forecast, and renders configurable birth/living/death transitions over a navigable sparse world.

## Build and run (Debian / Ubuntu)

```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libx11-dev libxi-dev libxcursor-dev libxrandr-dev libxinerama-dev libgl1-mesa-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
rustup update stable
cargo build
cargo test
cargo run --release
```

macOS: install the Rust stable toolchain with rustup, ensure Xcode Command Line Tools are installed, then run `cargo run --release`.


### Pattern imports and reference links

Use **Patterns → Import Pattern… → Browse…** to choose a `.rle` or two-state `.mc` Macrocell file. On macOS the chooser runs in a separate AppleScript process, and the game remains responsive while the chooser is displayed; after selecting a file, choose **Import into library**. The Browse button is temporarily disabled while a chooser is active. You can also enter a path directly.

The application starts with an **empty, paused board at generation 0**. The built-in **Gosper glider gun** appears under **Guns** and contains 36 living cells. The Help panel includes a link to the [Wikipedia Game of Life article](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life); the Pattern Library links to the [LifeWiki pattern directory](https://conwaylife.com/patterns/).

## Features

- Sparse, logically unbounded Conway Life, with generation tracking and deterministic B3/S23 behavior.
- Run/pause, single-step, ten-step, reset, clear, **randomize**; 0.25–240 requested generations/sec with achieved speed readout.
- Pan (middle-drag or Shift-drag), zoom (wheel), **Fit All** (Home) to frame every living cell, click/drag draw and erase.
- Persistent bottom status bar: generation, population, predicted births/deaths, target/actual speed, zoom, and run state.
- Three generation snapshots and selectable **Forecast** or **History** view; optional independent history ghosts.
- Settings window with RGB swatches, opacity sliders, visibility switches, background and grid controls; persistent TOML settings.
- Adjustable **UI scale** (applied after closing Settings), random area X/Y dimensions and fill density.
- Configurable keyboard bindings (defaults: Space run/pause, Up faster, Down slower, **Home fit all living cells**).
- Editing/stamping never pauses a running universe. Hold left-drag with an active pattern to place repeated stamps at intervals of half the pattern bounding-box diagonal plus the configurable extra-cell spacing (default 5).
- Searchable bundled pattern browser with classic Life patterns; click a pattern to activate placement, then click the field, or drag directly from the catalog to the field.
- Import `.rle` patterns from a typed path and export the current world as RLE.

### Limitations / scope

This is an **initial release**, not a full realization of every feature in the design specification. The bundled collection is curated, not every known pattern. Pattern placement is click-to-stamp or list-to-field drag initiation, rather than native OS file drag-and-drop. Save/load is RLE and does not preserve generation history. Undo/redo, package catalog indexing, advanced mesh-instancing / GPU rendering and user-configurable placement policies are not implemented. Rendering uses egui's viewport-clipped, batched painting via Bevy's renderer, not one ECS entity per cell; dense, high-population worlds may require a chunked GPU renderer.

**Build verification:** Rust/Cargo were unavailable in the generation environment. Code and tests are included, but compilation has not been verified here.

## Controls

| Action | Interaction |
|---|---|
| Run/pause | Toolbar or Space (configurable) |
| Fit all cells | Home or Fit All toolbar button; centers and zooms to include all living cells |
| Speed up/down | Up / Down arrows (configurable) |
| Advance | Step / +10 |
| Randomize | Toolbar; region and density configured in Settings |
| Draw | Left mouse drag |
| Erase | Right mouse drag |
| Pan | Middle mouse drag or Shift + drag |
| Zoom | Mouse wheel |
| Stamp | Click to place once; hold left-drag to place repeatedly at distance-based spacing |
| Rotate/mirror | Buttons below pattern browser while a stamp is active |
| Import/export | File… dialog (textual file path) |
| Preferences | Settings dialog, then Save preferences |

## Files and source layout

- `Cargo.toml` — dependency pins and build profiles.
- `src/main.rs` — Bevy application, interactive desktop UI, viewport painting, file actions.
- `src/lib.rs` — re-exports testable library modules.
- `src/simulation.rs` — pure Life evolution, history snapshots, transition classification.
- `src/patterns.rs` — built-in catalog, standard RLE parser/writer, rotation/mirroring.
- `src/settings.rs` — RGBA style model, default palette and TOML persistence.
- `tests/core.rs` — core simulation, history, pattern and settings tests.
- `DESIGN.md` — original full requirements and proposed future architecture.
- `.gitignore` — common Rust, TypeScript/Node and Python exclusions.

The source of each listed file is included in this project directory and is the authoritative code listing. To view any file's exact contents:

```bash
cat Cargo.toml
cat src/main.rs
cat src/lib.rs
cat src/simulation.rs
cat src/patterns.rs
cat src/settings.rs
cat tests/core.rs
```

## Prioritized enhancements

1. Compile and run on Debian/macOS; fix any Bevy platform-specific regressions, then add CI.
2. Replace per-frame candidate set allocations with spatial chunks, mesh batches and dirty-region updates.
3. Add OS-native drag/drop, hotkey rotation/mirroring, live transparent placement preview and replace/XOR stamping.
4. Add full pattern metadata, thumbnails, favorites and large catalog-pack indexing.
5. Add undo/redo and a versioned binary world format preserving history, viewport and settings.
6. Add benchmarks and stress tests for large populations and 240 GPS scheduling.


### Compatibility update (2026-10-23)

The desktop UI uses the egui 0.36 `Panel` API via a root `Area`; previous `TopBottomPanel` / `SidePanel` calls and `CentralPanel::show(&Context, ...)` are no longer valid.

## Repeatable stamping and additional shortcuts

- Selecting a pattern arms it for repeated placement. Click or drag into the field repeatedly; the selected pattern is preserved until you choose another one or press **Cancel placement**.
- The toolbar has a **🔍 Fit All (Enter)** action that fits the entire live population into the viewport. The shortcut is configurable in Settings.
- **Up/Down** uses the existing speed presets (e.g., 2, 4, 8), snapping to the next preset.
- **Shift+Up/Down** adjusts speed to the next/previous whole-number generations per second (e.g., 1.3 → 2.0 with Shift+Up, 4.5 → 4.0 with Shift+Down), limited to 0.25–240 GPS.
- Editing and pattern placement do not pause simulation playback.

### Input refinements

- **M** mirrors the active pattern horizontally; **R** rotates it.
- **Shift + left-drag/right-drag** draws/erases with a square brush (default **3 × 3** cells). Configure its width under Settings → Drawing brush.
- Pan via middle-drag or **Alt + drag** (Shift is reserved for the larger brush).
- The bottom status bar uses a single non-wrapping line to maintain a stable height.

### Rectangular erase

Hold **Control** and drag with the **left mouse button** to select a rectangle. A translucent red overlay previews the selection. Release the mouse button to erase all living cells in that inclusive area. This works while the simulation is running, and it takes precedence over pattern stamping.

## Help and viewport fixes

- Press **F1** to toggle the Help dialog.
- The status footer has a fixed height and horizontally scrolls if the window is too narrow.
- **Fit All** uses the measured viewport after the current frame’s panels are arranged, including after window resizing.

### Mouse-centered zoom

Mouse-wheel zoom keeps the cell beneath the pointer at the same screen position. Zoom is anchored to the pointer within the field, not the window center. Fit All continues to frame the full living population.

## Interface refinements (October 2026)

- **Fit All** now leaves a larger border (76% fit fraction) around the living population.
- **Status bar:** compact, fixed-height footer with hover descriptions on each metric.
- **Speed keys:** Up/Down work while navigating the pattern panel, including fractional speeds (0.25 → 0.5 → 1); Shift+Up/Down snaps to integer steps.
- **Cell shapes:** choose Filled, Circle, Square + border, Diamond, Hexagon, Star, or Cross independently for birthing, living, and dying cells in Settings.
- **Tooltips:** primary toolbar, pattern actions, status metrics, and library items provide hover descriptions.
- **Docked Help:** F1 toggles a collapsible right-hand panel, while the pattern library can be collapsed using its upper-right arrow.
- **Erasing:** a translucent red brush footprint appears while right-dragging; Shift uses the configured larger brush.

Appearance shape preferences are saved with existing colors and alpha values in `settings.toml`. Previous settings files remain compatible through Serde defaults.

## v1.0 UI and shortcuts

- **R** rotates the active pattern, **F2** toggles Pattern Library, and **F3** toggles Settings. Reset has no shortcut.
- **T** rotates a selected pattern (90 degrees), **M** mirrors it. Selecting Shift, Alt, or Ctrl cancels the active stamp. Closing Pattern Library cancels it as well.
- **Shift** previews the larger brush even when no mouse button is pressed; Shift-drag draws or erases with the configured brush width.
- Click **GAME OF LIFE** to show the centered About dialog; close with **Esc** or the window's X.
- **RLE** means Run Length Encoded, the standard compact Life pattern interchange format. The File dialog imports RLE patterns into the library and exports the active universe.
- Playback status uses text RUNNING/PAUSED instead of potentially unsupported font symbols.
- Factory defaults: births use cyan (`#00D6FF`) circles; deaths use orange-red (`#FF6448`) transparent square frames. Existing saved preferences remain in effect until restored to defaults.

### Net change status metric

The status bar reports the forecast net change as `Born − Dying`, followed by the percentage relative to the current living population. Positive values indicate growth, negative values decline. A zero current population displays `N/A` for the rate to avoid division by zero.

## Randomized Shift brush

Hold **Shift + left mouse** and drag to paint a sparse random patch rather than a filled rectangle. The brush width is controlled by **Settings → Drawing brush** (default 3 cells); its probability of living cells uses **Settings → Randomize → Alive probability** (default 30%). Unsampled cells inside the brush footprint are made dead. **Shift + right mouse** continues to erase the complete brush footprint. Regular left mouse drawing is unchanged.


## Erase Outliers (density-based population cleanup)

The **Erase Outliers** toolbar button pauses simulation, analyzes current living cells, and opens a preview dialog. Red overlay cells are proposed for removal; green outlines mark protected population regions. Change the controls to recalculate the preview, then choose **Apply removal** or **Cancel / Esc**. Applying returns to the prior run/pause state and enables **Undo Outliers** (one-level current-population snapshot).

Default configuration: detection radius **15 cells**, minimum local population **5** (includes the cell itself), minimum retained mass **5%** of living cells, region padding **8 cells**, preserve multiple masses **on**, and include next-generation forecast births in *detection* **on**. Only currently living cells are deleted.

The clustering algorithm is in `src/outliers.rs`. It uses spatial bins, Euclidean distance, density-connectivity expansion, and a minimum population threshold. It retains the bounding region plus padding for each significant mass. If no qualifying mass exists, the operation removes nothing as a precaution. For disjoint dense groups, multiple-mass mode retains all that meet the threshold; single-mass mode retains only the most populous.

**Important:** This is deliberately an editorial cleanup operation, not a biological inference. Gliders, sparse streams, and isolated still lifes may be marked for removal. Inspect the preview. The scan runs synchronously when invoked or settings change; particularly large universes may briefly pause the UI during analysis. Settings are adjustable for each operation (not yet saved to TOML).

## Interface refinements (2026-10-23)

- Pattern library inserts whitespace and separators between pattern categories.
- Toolbar actions are grouped by function: playback, population editing, fit, outlier cleanup, and UI/dialogs.
- Factory-default simulation speed is 5.0 generations/sec. Existing saved preferences retain their previous speed until changed or reset.
- Applying outlier removal requests Fit All on the updated living population. The removal preview highlights each proposed deletion with a red square and retains green region outlines.
- The status bar groups large integer values with commas, shows a colored direction arrow for net change, reserves a fixed width for the signed change, and no longer shows the last application message.
- Native window title is **Game of Life**.

## Expanded Pattern Catalog

Additional authentic patterns are cataloged in `assets/patterns/catalog.json` (Eaters, Puffers, Rakes, Breeders, Methuselahs and Universal Constructors). They are **not synthetic placeholders**. Since the upstream RLE content cannot be bundled without retrieving it, run this one-time command with internet access:

```bash
python3 scripts/fetch_patterns.py
```

Restart the application to load successfully downloaded `.rle` assets. Any pattern missing from the upstream directory is reported as unavailable and is not displayed. Extremely large or non-RLE constructors may require future Macrocell support; the current parser accepts RLE patterns up to 100,000 by 100,000 coordinates and 2 million living cells.

The Pattern Library's **Import Pattern** button accepts a local `.rle` file path. Export was intentionally removed pending further design. See `CHANGELOG.md` for the history of requested features.

## Native RLE and Macrocell imports

The Pattern Library **Import Pattern** action supports the standard two-state
Conway Life formats `.rle` and Golly `.mc` (`[M2]` Macrocell). Enter the path of
a downloaded file; it is parsed locally in Rust and added to the library. The
included `assets/patterns/puffer1.rle` is a sourced example. Files already in
`assets/patterns/` matching the catalog manifest are loaded automatically.

The Macrocell importer expands compressed quadtrees into living-cell coordinates;
it is **not** a Hashlife implementation. Expansion is limited to 2 million live
cells and unsupported rules are rejected. Extremely large structures such as
Gemini require a hierarchical/Hashlife simulation backend for efficient playback.
Macrocell `.mc.gz` and multi-state Macrocells are not yet supported.

LifeWiki pattern source: https://conwaylife.com/patterns/ . Pattern asset
provenance should be retained when redistributing catalog packs. Not every
catalogued pattern is included in this archive; the program never substitutes
synthetic patterns under authentic names.

### LifeWiki pattern directory

The Pattern Library panel includes a clickable [LifeWiki pattern directory](https://conwaylife.com/patterns/) link. LifeWiki hosts Conway’s Game of Life pattern files; download a `.rle` or `.mc` file and use **Import Pattern…** in the panel to load it. Browsing the directory is optional and does not change the simulation.


### Recent interface changes

- `Browse…` opens a native file chooser for RLE (`.rle`) and Macrocell (`.mc`) pattern imports. Identical imported cell layouts are not duplicated.
- The Pattern Library and Help sidebars use icon-only collapse controls; Help (`F1`), Patterns (`F2`) and Settings (`F3`) are the final toolbar group.
- Settings includes **Show only currently living cells** and **Live cells (%)** (used by Randomize and the Shift brush).
- Randomize clears the universe, generates a new starting population and automatically fits the living population to the canvas.
- Erase Outliers analysis can be collapsed before applying or canceling. No dedicated outlier undo is provided.
- **Reset** restores generation zero of the current starting snapshot, which is replaced on Randomize; it does not undo edits. **Clear** also clears that starting snapshot.
- World coordinates are signed 64-bit integers; the map is not bounded by the window. Cells outside the viewport continue to evolve. Only coordinates at numeric overflow are excluded.

### Snapshot and restore

**Snapshot** stores the complete current simulation state (living cells, two historical generations, forecast and generation count) in one in-memory slot. Taking another snapshot overwrites that slot. **Restore Snapshot** pauses playback, restores the saved state and frames the restored population. Snapshots are not written to disk and disappear when the application exits.

**Settings** save automatically when the window closes. The default UI scale is 1.35 and the default enlarged Shift brush width is 5 cells; existing saved preferences take precedence. **Reset history to now** copies the current living population into both past generations, clearing ghost trails/edited-history flags without changing the simulation's current generation.

**Forecast G → G+1** categorizes cells by predicted next-generation fate. **History G−1 → G** categorizes cells by their most recently completed transition. Optional **G−1** and **G−2** trails show faint past locations; they never influence Game of Life evolution.

### Status bar and outlier preview

Target/Actual GPS, zoom, and playback state remain aligned to the right of the status bar. The left-side population information scrolls horizontally when space is limited. Opening Erase Outliers fits the visible population; closing the dialog with the X cancels the proposed removals.

### Latest controls and defaults

- **Enter**: Fit All (unless Erase Outliers is open, in which case it applies the removal).
- **Esc**: Cancel Erase Outliers preview.
- **Clear**: Pause and clear all cells.
- **Generations/sec wheel**: Increase/decrease by exactly 1 GPS per wheel event, even on the logarithmic slider.
- **Pattern drag extra spacing**: 15 cells by default.
- **Grid**: RGB/opacity/visibility controls, but no cell shape control; default opacity 0.25.

Factory defaults apply to new settings or after choosing **Restore defaults**.


## Simulation rules

Use **Rules (F4)** after **Settings (F3)** on the toolbar, or press **F4**, to switch between Conway's Life (B3/S23), HighLife (B36/S23), Seeds (B2/S), Day & Night (B3678/S34678), and Brian's Brain. The active rule name appears first on the left of the status bar. Changing rules clears the entire board (including refractory cells), both historical buffers, the starting state, and the in-memory snapshot. Generation returns to 0 and the simulation pauses. Selecting the already-active rule makes no changes. Brian's Brain uses ON, refractory, and OFF states; refractory is shown with the dying-cell palette, unless **Show only currently living cells** is enabled. RLE and Macrocell pattern imports provide seed coordinates and do not change the active automaton rule.

### Random rectangle drawing

Hold **Alt + left mouse button**, drag out a rectangular boundary, and release to populate that region using **Settings → Random field** density. Existing cells outside the boundary remain unchanged; cells inside are replaced according to the selected probability. A live preview outlines the rectangle. Maximum selection: 2,000,000 cells to keep the interface responsive. Use middle-button drag to pan. Dialogs open expanded and cannot be collapsed.
