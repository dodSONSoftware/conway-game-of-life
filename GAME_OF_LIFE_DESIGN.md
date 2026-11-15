# Game of Life — Technical Design Document

**Status:** Initial design / implementation specification\
**Target:** Native desktop, Rust, Bevy 0.20, `bevy_egui` 0.43.x\
**Rules:** Conway's Game of Life (B3/S23)\
**Revision:** 0.1 — 2026-10-23

## 1. Overview

Build a high-performance, interactive Conway's Game of Life desktop application with a large navigable 2D universe, precise simulation controls, a searchable drag-and-drop pattern library, and an expressive three-generation visualization. Users can pause, single-step, run faster or slower, edit living cells, place library patterns, customize colors and transparency, and save/reload their work.

**Design principles:** deterministic simulation; separation of simulation, history, visualization and UI; zero per-cell ECS entities; responsiveness when paused or under heavy load; transparent and configurable rendering; straightforward offline use.

### 1.1 Goals

1. Simulate the standard B3/S23 rules correctly, with all cells advancing simultaneously.
2. Allow panning, zooming, editing, and placing patterns over a large, conceptually unbounded plane.
3. Show current living cells alongside birth and death transition information, using three generation buffers.
4. Provide a settings window for independently adjusting cell-state colors and opacities.
5. Support browsing, searching, previewing, rotating, mirroring, and placing a comprehensive, extensible pattern catalog.
6. Make simulation cadence adjustable independently from rendering frame rate.
7. Persist settings, universe snapshots and user-imported patterns.

### 1.2 Non-goals for the initial release

- Multiplayer/network simulation.
- Biological metabolism, genetics, evolutionary mutation, or non-B3/S23 rules.
- Guaranteeing that **all** known Life patterns are shipped: the catalog is continuously evolving and external collections can be enormous. The app will support full import/indexing, rather than claim a permanently complete bundled database.
- Three-dimensional cellular automata. The three *layers* are generations of a **2D** world, not spatial Z-layers used in the Life rules.

## 2. Technology

- **Language:** Rust (stable toolchain).
- **Game engine:** Bevy `0.20` (2D graphics, input, ECS scheduling, assets and window management).
- **GUI:** `bevy_egui` `0.43.x` for docked toolbars, floating Settings dialog, pattern browser, searchable catalogs, RGBA color inputs and sliders.
- **Serialization:** `serde` with `toml` for preferences, and RLE or a versioned binary/sparse format for worlds.
- **Catalog:** built-in metadata/index plus on-disk `.rle` pattern files; additional collections can be imported.
- **Simulation:** CPU-based initially, independent of Bevy ECS entity counts. GPU compute is a possible later optimization.
- **Performance instrumentation:** Bevy diagnostic plugins and application-level counters.

Version choices were current when this specification was written (October 23, 2026). Pin and validate dependencies when scaffolding the app.

## 3. Interface and user workflow

### 3.1 Main window

```text
+---------------------------------------------------------------------------------+
| File  Edit  Simulation  Patterns  View  Settings  Help                          |
+---------------------------------------------------------------------------------+
| [Run/Pause] [Step] [Step ×10] [Reset]  Speed [-----|----]  30 gen/s  G: 12,480   |
+----------------------------+----------------------------------------------------+
| PATTERN LIBRARY            |                                                    |
| Search [____________]      |               INFINITE FIELD                       |
| Category: Spaceships       |                                                    |
|  > Glider                  |       pan / zoom / click to edit                   |
|  > Lightweight spaceship   |       drag pattern here to place                  |
|  > ...                     |       births / alive / deaths colored             |
|                            |                                                    |
| Preview / RLE metadata     |                                                    |
+----------------------------+----------------------------------------------------+
| Alive: 28,410 | Born: 712 | Dying: 706 | FPS: 60 | gen/s: 30 | Zoom: 75%        |
+---------------------------------------------------------------------------------+
```

The pattern browser can collapse to maximize viewport area. Settings is a movable window (not a separate operating-system process) accessible from the menu, a toolbar icon, and a shortcut.

### 3.2 Simulation controls

| Control | Behavior |
|---|---|
| Run / Pause | Toggle simulation progression without stopping camera or UI updates. |
| Single Step | Advance exactly one generation while paused. |
| Step ×10 | Advance ten deterministic generations, then remain paused. |
| Speed slider | Set desired generations per second (GPS); logarithmic or tiered ranges, e.g. 0.25–240 GPS. |
| Speed +/- | Increment/decrement among human-friendly presets. |
| Reset | Restore last starting snapshot, after confirmation for unsaved edits. |
| Clear | Empty the world; optionally clear/reset historical buffers. |
| Pan / Zoom | Middle-button or space-drag to pan; scroll to zoom centered on pointer; optional fit-to-pattern. |
| Draw / Erase | Left-click/drag to add living cells, right-click/drag to erase; edits work while paused. |
| Pattern placement | Drag a preview from the library, move a translucent ghost into the world, click/release to stamp. |

UI must show **requested GPS** and **actual achieved GPS** separately; avoid claiming that a slow device is running faster than it is.

### 3.3 Placement operations

- Preview ghost follows cursor and snaps to integer grid coordinates.
- `R` rotates by 90°, `H`/`V` mirror horizontally/vertically, `Esc` cancels.
- Placement policy defaults to **union/additive** (existing live cells remain); optional **replace footprint** and **XOR/toggle** modes.
- Placement while running: default behavior pauses before stamping, allowing an intentional and deterministic edit. A configurable live-stamp mode can be added later.
- Undo/redo of user edits, not arbitrary simulation generations, is targeted for the first release.

## 4. Three-generation model

### 4.1 Authoritative state

Three discrete state snapshots are maintained:

| Layer | Symbol | Meaning |
|---|---|---|
| **Layer 0** | `current` / `G` | Authoritative live/dead population now. |
| **Layer 1** | `previous` / `G-1` | Population one generation earlier. |
| **Layer 2** | `older` / `G-2` | Population two generations earlier. |

**Each generation proceeds in this order:**

```rust
let next = evolve(&current); // calculate from current only
older = previous;
previous = current;
current = next;
generation += 1;
```

Actual buffers are rotated or swapped rather than deep-copied every generation. The simulation rules *read only `current`*. Neither historical layer can affect births or survival.

### 4.2 Birth, living and death visualization

Because “about to” implies a prediction, two distinct views are provided:

**Default: Forecast mode** — derive a noncommitted `next = evolve(current)` and classify cells using `current` and `next`:

| Cell state | Condition | Default color | Default opacity |
|---|---|---|---:|
| **Birthing** | `!current && next` | Electric cyan `#00D9FF` | 0.75 |
| **Living (surviving)** | `current && next` | Emerald green `#36E57A` | 1.00 |
| **Dying** | `current && !next` | Vivid orange-red `#FF6548` | 0.55 |
| **Dead/background** | `!current && !next` | Near-black `#10141D` | 1.00 |

This shows an impending birth as an illuminated empty location, surviving living cells as green, and currently living cells that will die on the next step as fading orange-red. In this mode, `next` is a **derived prediction**, not a fourth history layer and never advances the simulation by itself.

**Alternate: History mode** — use `previous` and `current`:

| Cell state | Condition | Interpretation |
|---|---|---|
| Born | `!previous && current` | Became alive in the most recent step. |
| Survived | `previous && current` | Remained alive. |
| Died | `previous && !current` | Died in the most recent step; rendered as an overlay/ghost. |

`older` enables a secondary **history-trail** option: render `G-1` and `G-2` footprints with separately configurable ghost opacity (e.g. 0.18 and 0.08) to clarify motion paths, including gliders. Historical ghosts do **not** count as living cells.

**Important drawing policy:** A currently live cell that is predicted to die is colored **dying**, not both living and dying. A predicted birth is drawn as a translucent glyph in an otherwise dead location. Classification precedence is explicit; overlay order must not cause a misread of the underlying binary population.

### 4.3 Editing and initial-generation semantics

- After a fresh load, `previous` and `older` are initially identical to `current` by default, so a loaded pattern does not erroneously appear to have just been born.
- When paused and edited, recompute forecast immediately; leave historical snapshots intact unless the user chooses **Reset history to current**.
- History transitions after external edits may reflect those edits rather than a natural B3/S23 step. The UI marks the history as *edited* until the next simulation generation, and the user may reset it.
- Predictions and coloration are deterministic and recomputed only when current-state changes.

## 5. Settings window

Settings can be opened without stopping simulation and uses immediate preview. Changes to color or opacity should affect the viewport without forcing a simulation recalculation.

### 5.1 Appearance → Cell states

| Type | Color control | Alpha slider | Visibility |
|---|---|---|---|
| Birthing | RGBA/hex color picker | 0–100% (default 75%) | Toggle |
| Living | RGBA/hex color picker | 0–100% (default 100%) | Toggle |
| Dying | RGBA/hex color picker | 0–100% (default 55%) | Toggle |
| G-1 history trail | RGBA/hex color picker (default muted blue `#648DFF`) | default 18% | Toggle (off by default) |
| G-2 history trail | RGBA/hex color picker (default muted purple `#9B80E8`) | default 8% | Toggle (off by default) |
| Grid lines | RGBA/hex color picker (default `#354050`) | default 22% | Toggle |
| Background | RGB/hex picker (default `#10141D`) | 100% | Always |

UI controls:

- A color swatch plus RGB/HSV picker and editable hex field.
- Independent alpha slider and numeric percentage field for each state.
- Instant preview of sample glider, still life and predicted birth/death composition.
- **Reset individual state** and **Restore default palette** actions.
- A **Visualization mode** selector: Forecast / History.
- A **Show older history trails** toggle with independent alpha.
- Optional **Colorblind-friendly palette** presets in a later release.
- Persist changes across launches in a local `settings.toml`; allow Import/Export preset in a later release.

### 5.2 Other settings

- Simulation: default GPS, max catch-up steps/frame, auto-pause on placement, run-on-load.
- Rendering: grid visibility and minimum zoom threshold, cell gap/border, interpolation disabled for authoritative state, vertical sync and FPS display.
- Camera: zoom sensitivity and initial viewport center.
- Library: user-pattern directories, search/indexing preferences.
- Persistence: autosave interval and last opened universe.

### 5.3 Example configuration

```toml
[simulation]
target_gps = 30.0
max_steps_per_frame = 8
pause_on_stamp = true

[visualization]
mode = "forecast"
show_history_trails = false

[visualization.birth]
color = "#00D9FF"
alpha = 0.75
visible = true

[visualization.living]
color = "#36E57A"
alpha = 1.0
visible = true

[visualization.dying]
color = "#FF6548"
alpha = 0.55
visible = true

[visualization.history_g1]
color = "#648DFF"
alpha = 0.18
visible = false

[visualization.history_g2]
color = "#9B80E8"
alpha = 0.08
visible = false

[visualization.grid]
color = "#354050"
alpha = 0.22
visible = true

[visualization.background]
color = "#10141D"
```

## 6. Large-universe simulation and rendering

### 6.1 Coordinate model

Use signed integer world coordinates `(i64, i64)`, with independent camera pan/zoom. The universe is logically unbounded; practical limits are dictated by memory and computation. The screen only renders the currently visible bounds.

### 6.2 Simulation representation

**Recommended MVP:** `HashSet<IVec2-like coordinate>` for living cells, or a custom `CellPos { x: i64, y: i64 }` hash set. Each generation builds neighbor counts around active cells only. This is straightforward, deterministic in outcome, and ideal for sparse glider/oscillator patterns.

**Scale-up path:** sparse fixed-size chunks (e.g. 64×64 bitsets) keyed by chunk coordinates, storing `current`, `previous`, and `older` per active chunk. Implement neighbor-border handling carefully and discard empty chunks. This improves locality and makes rendering changed chunks efficient.

Never spawn one Bevy ECS entity or one scene object per living cell. Use batched quads/instances or a texture-backed tile renderer; only viewport-visible chunks are drawn. Renderer implementation details should be validated against Bevy 0.20's render APIs.

### 6.3 Update scheduling

Maintain a simulation-time accumulator independent of wall-clock rendering:

```text
accumulator += delta_seconds
step_duration = 1 / target_generations_per_second
while running && accumulator >= step_duration && steps < max_steps_per_frame:
    advance_one_generation()
    accumulator -= step_duration
```

Define an explicit catch-up policy when overloaded (e.g. clamp lag or slow simulated time), and expose measured GPS. Single-step and pattern stamping bypass the accumulator. Rendering remains responsive even when paused.

### 6.4 Rendering pipeline

1. Determine visible world coordinates from camera transform/zoom.
2. Obtain `current`, `previous`, `older` and forecast `next` snapshots or caches.
3. Classify visible cells by selected visualization mode.
4. Generate batched geometry/instance data for living, birthing and dying cells.
5. Optionally render history ghosts *behind* foreground classifications.
6. Draw grid at zoom levels where it is legible.
7. Draw pattern placement ghost and selection tools.
8. Render egui toolbars/windows over the simulation.

State changes mark relevant chunks dirty; palette-only changes update materials/uniforms/instance appearance without rerunning B3/S23.

## 7. Pattern library

### 7.1 Scope

Ship a curated starter collection, organized by **still lifes, oscillators, spaceships, guns, puffers, rakes, breeders, methuselahs, logic/circuit structures** and **user patterns**. Include known classics such as block, beehive, blinker, toad, beacon, pulsar, pentadecathlon, glider, lightweight spaceship, Gosper glider gun and R-pentomino.

The phrase “all known shapes” is treated as a **library coverage goal**, not a finite hardcoded list. Import catalog packages and individual `.rle` files; retain provenance/license/attribution information supplied with the source. Very large patterns are loaded on demand rather than all kept expanded in memory. Periodic catalog refresh is a future capability, not an automatic requirement for offline operation.

### 7.2 Pattern metadata

```rust
struct PatternMetadata {
    id: String,
    name: String,
    category: String,
    description: Option<String>,
    width: u32,
    height: u32,
    period: Option<u64>,
    population: usize,
    author: Option<String>,
    source: Option<String>,
    tags: Vec<String>,
}
```

Use a parsed sparse cell-set plus optional cached thumbnail, with original `.rle` bytes available for round-trip export.

### 7.3 Browser behavior

Search by name, class, period, population and tags; preview patterns; inspect metadata; favorite; stamp repeatedly; rotate/mirror; import/export standard RLE. Show placement bounding box and current coordinate before drop.

## 8. Rust module boundaries

```text
src/
  main.rs                 # Bevy application and plugin registration
  app.rs                  # High-level scheduling and state
  simulation/
    mod.rs
    world.rs              # Coordinate representation and world access
    rules.rs              # Pure B3/S23 evolution function
    history.rs            # Three-snapshot ring rotation
    clock.rs              # Generation timing
  rendering/
    mod.rs
    viewport.rs           # Camera, culling and visible bounds
    classifier.rs         # Forecast/history display classification
    cells.rs              # Batched cell rendering
    palette.rs            # Theme data and alpha
    overlays.rs           # Grid, history trails and stamp ghost
  ui/
    mod.rs
    toolbar.rs
    settings.rs
    pattern_browser.rs
    status.rs
  patterns/
    mod.rs
    catalog.rs
    rle.rs
    placement.rs
  persistence/
    mod.rs
    settings.rs
    world_file.rs
  input/
    mod.rs
    camera.rs
    editing.rs
assets/
  patterns/               # Curated RLE patterns and catalog manifest
  fonts/
tests/
  rules.rs
  history.rs
  placement.rs
  rle.rs
  settings.rs
.gitignore
Cargo.toml
README.md
```

Use plugins/resources as orchestration tools, not as substitutes for pure, testable core logic. The simulation library must be usable without a window, GPU or Bevy renderer.

## 9. Behavioral acceptance criteria

1. A blinker alternates orientation after one generation and returns after two.
2. A block remains unchanged for thousands of generations.
3. A glider translates diagonally by one cell every four generations on an unbounded world.
4. Pausing stops generation increments without stopping camera and UI input.
5. Step performs precisely one generation, regardless of selected speed.
6. Three snapshots rotate correctly: after stepping from G to G+1, they contain G+1, G, G-1.
7. Forecast classification matches exact expected birth/survival/death sets computed from B3/S23.
8. History classification matches the actual `previous → current` transition and does not influence future simulation.
9. Updating colors and alpha changes only appearance, not population or generation count.
10. Color/opacity settings survive application restart and Restore Defaults restores documented values.
11. Drag/drop placement respects snapping, rotation, mirroring and selected stamp mode.
12. An imported RLE pattern produces the same relative living-cell layout when exported again.
13. Zooming and panning never alter simulation state; only visible chunks are rendered.
14. A paused editor remains responsive while the viewport contains a large population.
15. Requested and achieved simulation speeds are shown separately.

## 10. Delivery phases

**Phase 1 — Working simulator:** B3/S23 core, generation clock, three history buffers, default color classification, camera pan/zoom, draw/erase, and pause/step/speed controls.

**Phase 2 — Settings and visuals:** Floating settings window, per-state RGB/hex and alpha inputs, background/grid controls, instant preview, persisted settings, forecast/history switch, historical trails.

**Phase 3 — Pattern library:** Browser, bundled catalog, RLE import, previews, drag-and-drop placement, rotate/mirror and search.

**Phase 4 — Scalability and usability:** Sparse chunk/bitset representation, chunk culling and dirty updates, world save/load, undo/redo, statistics and diagnostics.

**Phase 5 — Advanced capabilities:** Large catalog package indexing, time travel/replay, editable rule sets, advanced performance features, export of animation/video, and GPU compute experimentation.

## 11. Design decisions needing empirical validation

- Sparse hash sets versus chunked bitsets should be benchmarked with empty, dense, clustered and wide-ranging worlds.
- For very large populations, forecasting must avoid redundant computation. Consider caching one step ahead and reusing it during the next authoritative advancement.
- Transparency must use premultiplied alpha or a consistent blend model with well-defined overlay order, especially when history ghosts and foreground markers overlap.
- Consider per-cell borders or distinct glyph shapes as an accessibility alternative to relying solely on color.
- Floating egui window focus must prevent edits from leaking through the settings interface into the world underneath.

## 12. Reference dependencies

- Bevy 0.20 release: https://bevy.org/news/bevy-0-20/
- `bevy_egui` compatibility / changelog: https://docs.rs/crate/bevy_egui/latest/source/CHANGELOG.md
- LifeWiki pattern reference: https://conwaylife.com/wiki/
- Golly cellular automata tool: https://golly.sourceforge.io/

---

**Core principle:** The engine simulates a single Conway universe; it remembers two preceding generations; and it renders historical or predicted transitions as configurable visual annotations. Color, opacity and history never modify the Game of Life rules.
