# Changelog

## 2026-10-24 — Rules shortcut and toolbar order

- Added **F4** to toggle the Rules dialog.
- Moved **Rules (F4)** immediately after **Settings (F3)** in the toolbar.
- Updated Help, Settings shortcut hints, and README.

## 2026-10-24 — Expanded dialogs and Alt-drag random region

- Dialog windows open expanded and no longer offer collapse toggles.
- Alt + left-drag previews a rectangular cell selection and fills it at mouse release using the current Randomize density setting.
- Existing cells outside the rectangle remain unchanged; selections exceeding 2 million cells are rejected to protect UI responsiveness.
- Middle-button drag remains the viewport pan gesture.


Development history of **Game of Life**. **Newest entries first**, oldest at the bottom.

All events below occurred in the project conversation on **2026-10-23**. The supplied 47-page PDF preserves their order but does **not** provide a reliable timestamp or Git tag for each individual change. Entries are therefore numbered in **reverse discussion order**, not represented as distinct formal releases. The wording distinguishes requested work from implemented/claimed changes and later corrections. Page references are to `Describe Conway's Game(1).pdf`. The two warning fixes at the top are from the current build report, after that export.

## 2026-10-24 — Empty board on application startup

### Changed
- New application sessions start with an empty, paused generation-zero board instead of automatically placing a Gosper glider gun.
- The Gosper gun remains available in the Pattern Library for manual placement.
- Added a startup-state regression test covering the empty population, history, forecast, and snapshot.

## 2026-10-23 — Gosper glider gun catalog regression fix

### Fixed
- Corrected the eighth row of the bundled Gosper glider gun RLE: its trailing dead-cell run was 22 instead of 20, exceeding the declared 36-cell width.
- The strict RLE parser correctly rejected the malformed pattern, but catalog construction silently skipped it via `filter_map`; the bundled gun therefore disappeared and its test failed.
- Strengthened the bundled-gun regression test to validate its 36-cell population and 36×9 bounding box.

## 2026-10-23 — egui scroll API compilation fix

### Fixed
- Replaced the unavailable `InputState::raw_scroll_delta` field in the Generations/sec slider handler with `smooth_scroll_delta`, which exists in the pinned egui version.
- Preserved the existing one-unit mouse-wheel speed stepping behavior.

## 2026-10-23 — Rule switching starts a new universe

### Changed
- Selecting a **different** automaton rule now clears the board, all historical/forecast/refractory states, the starting population and the single in-memory snapshot.
- Rule changes reset the generation counter to 0 and pause playback; inactive pattern placement and pending outlier work are discarded.
- Selecting the **already active** rule leaves the current simulation and snapshot untouched.
- Updated Rules dialog explanatory text, README and regression tests for the new behavior.

## 2026-10-23 — Multiple Game of Life rules

### Added
- **Rules** toolbar button and selection dialog with name, notation, and description for Conway's Life (B3/S23), HighLife (B36/S23), Seeds (B2/S), Day & Night (B3678/S34678), and Brian's Brain (three-state).
- Active rule name as the first metric in the left side of the status bar.
- Native rule-based evolution and a refractory state for Brian's Brain; tests cover each rule and switching behavior.

### Changed
- **Superseded by the newer rule-switch behavior above:** the initial implementation preserved cells and generation count when switching rules.
- Brian's Brain refractory cells are displayed using the dying-cell palette in the multi-state view; the living-only view displays ON cells alone.
- Snapshots preserve the selected rule and refractory state. Randomize and Clear preserve the selected rule.

## 2026-10-23 — Nonblocking import chooser, Wikipedia Help link, and glider gun verification

### Fixed
- Moved the native file-selection operation off the Bevy/egui update thread so an unresponsive file picker cannot freeze simulation/rendering. On macOS, use an independent `osascript` chooser process; the main UI polls the selection channel without blocking. Repeated Browse clicks are disabled until the chooser finishes.

### Added
- Linked the Wikipedia article on Conway's Game of Life from the docked Help panel, with a short description.
- Corrected the bundled Gosper glider gun RLE to the canonical 36-cell seed in the **Guns** category and added a regression test confirming its 36-cell configuration and catalog presence.

### Changed
- Corrected outdated Help shortcut descriptions (R rotates; M mirrors; F1/F2/F3 toggle Help/Patterns/Settings).

## 2026-10-23 — Speed wheel, outlier keys, drawing defaults, and grid settings

### Changed
- Factory pattern drag-stamping spacing offset: **15 cells** (previously 5).
- Generations/sec mouse wheel applies exactly **+1 or −1 GPS** from the pre-slider value; prevents logarithmic slider wheel handling from multiplying the change.
- Clear now pauses playback and clears the simulation.
- Default **Fit All** shortcut changed from Home to **Enter**. When Erase Outliers is open, Enter applies the proposed removal instead of fitting.
- Erase Outliers accepts **Esc** to cancel and **Enter** to apply the removal.
- Grid opacity factory default changed to **0.25** (previously 0.22).
- Removed the shape dropdown from the Grid swatch, retaining its color, alpha, and visibility controls.

### Notes
- Previously saved `settings.toml` preferences override new factory defaults until **Restore defaults** is used.

## 2026-10-23 — Continuous pattern stamping and status-bar cleanup

### Added
- Continuous pattern placement while holding and dragging the left mouse button. Stamps are triggered by cursor distance traveled, not time, with interpolation across fast drags.
- Configurable additional stamp spacing in Settings (default **5 cells**). The center-to-center interval is **half the selected pattern bounding-box diagonal plus the extra spacing**.

### Changed
- Removed the separator immediately after the Net Change status-bar metric.
- Updated pattern-placement instructions in Help and README.

## 2026-10-23 — Status telemetry, toolbar grouping, and outlier preview

### Changed
- Anchored **Target / Actual GPS**, **Zoom**, and **RUNNING / PAUSED** at the right side of the fixed-height status bar; left-side metrics scroll independently on narrow windows.
- Reordered toolbar groups to **Clear / Randomize / Erase Outliers**, then **Snapshot / Restore Snapshot**, with a separator between the groups and after Restore Snapshot.
- Opening **Erase Outliers** now requests Fit All on the current viewport to make the entire population visible during preview.
- Added the title-bar close **X** to Erase Outliers; closing it cancels the preview and restores the previous playback state.

## 2026-10-23 — Single-slot snapshots and display settings

### Added
- Added **Snapshot** and **Restore Snapshot** toolbar actions. Snapshot preserves the current generation and all simulation layers; subsequent snapshots replace the previous slot. Restore pauses the simulation and fits the restored population.
- Added inline explanations for Forecast, History, and the two optional history trails in Settings.
- Added a regression test for restoring the simulation's full generation history.

### Changed
- Removed the Reset toolbar action and the separator between Randomize and Erase Outliers.
- Renamed dialogs to **Settings** and **Erase Outliers**.
- Settings now save automatically when closed by the title-bar X, Escape, F3, or the toolbar button; removed **Save preferences**.
- Default Shift brush width is now 5 cells; factory UI scale is now 1.35. Existing user settings remain in effect until defaults are restored.
- Added subtle horizontal padding around the net-change status readout.
- Expanded **Reset history to now** tooltip: it copies the current cell set into both past layers without changing the current generation.

## 2026-10-23 — File import, viewport and interface refinements

### Added
- Native file chooser (Browse…) to the pattern importer for `.rle` and `.mc` files.
- Import de-duplication based on the complete set of living-cell coordinates, irrespective of source filename.
- Settings toggle to display only current living cells or the full birth/living/death visualization.
- Percentage slider for randomized live-cell density, retained across launches.
- Tests for new default/display configuration and simulation beyond the viewport.

### Changed
- Randomize now automatically applies Fit All, using the final laid-out canvas dimensions.
- Made Pattern Library and Help collapse buttons icon-only sideways triangles.
- Made the Erase Outliers dialog collapsible.
- Moved toolbar menu buttons into Help, Patterns, Settings order; removed divider between playback buttons and generation-speed controls.
- Tightened Net Change indicator spacing by using content-width, monospace text instead of a 126-pixel minimum label.
- Removed Undo Outliers toolbar action and obsolete snapshot implementation.
- Clarified that Reset restores the starting snapshot (most recent Randomize pattern or initial state), rather than undoing the last action.

### Notes
- The simulation uses sparse signed 64-bit world coordinates, not a finite board. Camera bounds only affect display, not evolution; checked arithmetic discards neighbor positions beyond the integer coordinate limits.
- Import deduplication compares coordinate-exact patterns. Translated or rotated versions are considered distinct unless they share identical coordinates.

## 2026-10-23 — Latest: warning cleanup and complete changelog reconstruction

### Fixed
- Removed unnecessary parentheses from the Macrocell leaf detection condition in `src/patterns.rs` (`unused_parens`).
- Removed obsolete, uncalled `LifeApp::save_file` from `src/main.rs` (`dead_code`); pattern export remains intentionally absent from the UI.

### Documentation
- Rebuilt this changelog from the start of the supplied 47-page PDF and placed **all entries newest first**. Earlier releases had appended additions to the end or placed foundational milestones above newer work.

## 2026-10-23 — LifeWiki link in the Pattern Library (PDF pp. 46–47)

### Added
- Clickable `https://conwaylife.com/patterns/` link in the Pattern Library, with an explanation of the public downloadable Conway pattern directory and instructions for importing `.rle`/`.mc` files.

## 2026-10-23 — Native Macrocell and LifeWiki pattern imports (PDF pp. 45–46)

### Added
- Native Rust support for Life `RLE` and two-state `[M2]` Macrocell `.mc` pattern imports.
- Hierarchical Macrocell decoding, rule validation, malformed-node checks, memory/expansion limits, tests, and sample Puffer 1 pattern asset.
- Pattern Library integration for local `.rle` and `.mc` catalog entries.

### Limitations
- Macrocell decoding is **not** a Hashlife simulation backend. Large hierarchical patterns are bounded by sparse-cell expansion limits.
- The conversation records that the full LifeWiki pattern collection was **not** bundled; an optional fetch script was provided. A pattern name in catalog metadata is not evidence that its cells are installed.

## 2026-10-23 — Expanded catalog and project-history documentation (PDF pp. 43–45)

### Added
- Catalog metadata for Eaters, Puffers, Rakes, Breeders, additional Methuselahs, and Universal Constructors.
- Optional catalog asset-download script and initial `CHANGELOG.md`.

### Changed
- Grouped the Pattern Library by category headings rather than repeating categories on every pattern row.
- Added vertical scrolling to the pattern list and Settings dialog.
- Moved Import Pattern into the library as a dedicated action, deferring export UI pending further discussion.
- Reorganized the toolbar into playback/speed, Fit All, reset/randomize, outlier tools, and window actions.
- Adjusted net-change spacing and added rendered up/down trend indicators.
- Used separate red outlines for outlying groups while preserving green bounds for retained regions.

## 2026-10-23 — Toolbar and status-bar polish (PDF pp. 43–44)

### Changed
- Separated toolbar groups visually and moved `Generations/sec` immediately after Pause, Step, and +10; Fit All immediately after speed.
- Changed factory-default speed to **5.0 generations/sec** (saved preferences override defaults).
- Automatically queued Fit All after applying Erase Outliers.
- Stabilized net-change display width and tightened whitespace; positive trends green, negative red.
- Added thousands separators to large population and metric counts.
- Removed the last-action/notification segment from the status bar.
- Set OS window title to **Game of Life** (no `Three Generations` suffix).

## 2026-10-23 — Intelligent Erase Outliers (PDF pp. 38–43)

### Added
- Spatial-hash/density-based cluster analysis with configurable radius **15**, minimum local population **5**, retained mass threshold **5%**, and padding **8**.
- Multiple retained masses and forecast births included in detection by default.
- Non-destructive preview with green retained-region outlines and red candidate-removal display; Apply/Cancel confirmation.
- Undo for cleanup, no-major-mass safety behavior, and outlier-analysis unit tests.

## 2026-10-23 — Randomized enlarged drawing brush (PDF pp. 37–38)

### Changed
- Shift+left drag places a probabilistic distribution of live cells rather than filling an entire square; uses the configured random alive probability (default **30%**).
- Shift+right drag continues to erase across the configured enlarged brush.
- Batched brush edits to avoid recomputing next-generation forecasts once for each cell.

## 2026-10-23 — macOS runtime investigation (PDF pp. 35–38)

### Fixed
- Prevented a low-zoom panic caused by `f32::clamp(0.5, side / 2.0)` with a maximum below its minimum when rendering transparent square frames; added regression coverage.

### Investigated
- macOS Apple Silicon Metal log showed `bevy_egui` bindless-texture fallback, long-unrun Bevy ECS system/change-detection warnings, and repeated monitor-removal messages. Root causes for these later warnings were **not established** by the conversation.

## 2026-10-23 — Pattern selection and dialog Escape handling (PDF pp. 34–35)

### Fixed
- Canceling a pattern stamp clears the highlighted selection and hides the associated description/preview, rather than leaving stale pattern details visible.
- Escape closes Settings, File import, and About dialogs as appropriate, and cancels active placement.

## 2026-10-23 — Net population change (PDF pp. 33–34)

### Added
- `Net change` to the status bar after births/deaths: signed `born − dying` and percentage relative to the current population; zero-population percentage is `N/A`.
- Formatting tests for growth, decline, no change, and undefined percentage.

## 2026-10-23 — Pattern descriptions (PDF pp. 33–34)

### Added
- Brief, behavior-focused descriptions for the fifteen built-in patterns, displayed beneath the selected name/category and above Placement preview.
- Generic description for imported RLE patterns.

## 2026-10-23 — UI shortcuts, About dialog, selection and speed wheel (PDF pp. 30–33)

### Changed
- Clicking plain-text `GAME OF LIFE` title opens centered About dialog: `Game Of Life, v1.0`, `(c) 2026 Ren's Games`, application description, X/Escape close.
- Centered the About description, with title hover text simply **About**.
- Added **F1** Help, **F2** Pattern Library, **F3** Settings, and retained **Space** Run/Pause and **Home** Fit All.
- Restored **R** to Rotate Pattern and removed the Reset shortcut after a temporary conflicting assignment; **M** mirrors.
- Removed default pattern highlighting before selection.
- Mouse wheel over the speed slider adjusts by 1.0 GPS.
- Changed the playback status from unsupported glyphs to recognizable running/paused text or glyph equivalents.
- Holding Shift/Alt/Ctrl cancels a selected pattern; closing Patterns also releases the stamp.
- Holding Shift displays the larger drawing-brush preview even when not painting.
- Changed factory appearances to cyan-circle births `RGB(0,214,255)` and orange-red transparent-frame deaths `RGB(255,100,72)`.
- Explained RLE import/export as compact pattern-file interchange; later UI defers export.

## 2026-10-23 — Cell shapes, docks, tooltips and status refinements (PDF pp. 27–30)

### Added
- Configurable shapes per cell state: Filled, Circle, Square Frame, Diamond, Hexagon, Star, Cross.
- Tooltips on toolbar and status metrics, docked Help on right, collapsible side panels, translucent eraser preview.

### Fixed
- Changed square-with-border to a **transparent-centered frame** with colored border; corrected the resulting Rust match-arm type mismatch (`ShapeIdx` versus `()`).
- Tightened Fit All padding and made it account for window resize and visible canvas.
- Adjusted speed preset transitions for sub-1.0 GPS values.

## 2026-10-23 — Mouse-anchored zoom and layout repair (PDF pp. 25–27)

### Fixed
- Mouse-wheel zoom now anchors to the world coordinate under the cursor rather than the window center.
- Fixed footer/status-bar visibility across window sizes by constraining root layout and avoiding height-expanding wraps.
- Fit All uses freshly measured viewport dimensions, correcting resize-dependent offsets.

### Added
- F1 toggle for Help.

## 2026-10-23 — Rectangle erase and enlarged brush (PDF pp. 23–25)

### Added
- Ctrl+left drag rectangle erase with red translucent selection preview and erase on release.
- Configurable Shift enlarged brush (default 3×3, range 1–25); draw/erase remain supported during simulation.
- Moved viewport panning from Shift+drag to middle mouse or Alt+drag.
- Expanded Help instructions for editing gestures and shortcuts.

## 2026-10-23 — Pattern preview and transformation feedback (PDF pp. 22–24)

### Added
- Live selected-pattern preview in library, translucent cursor preview, orientation readout, rotation and mirroring feedback.
- Rotate/Mirror buttons and keyboard controls; changed Mirror from unsupported decorative glyph to readable text, eventually **M**.
- Pattern selection persists after each stamp, permitting repeated placement without returning to the library.

## 2026-10-23 — Help quick reference (PDF pp. 21–22)

### Changed
- Reorganized Help into Mouse controls, Patterns, Simulation controls, and Settings, with grouped sections and more whitespace.

## 2026-10-23 — Speed stepping and Fit All (PDF pp. 20–21)

### Added
- Up/Down keys move to the next lower/higher speed preset; Shift+Up/Down moves to the next integer step even from fractional GPS.
- Toolbar Fit All button in addition to configurable Home shortcut.
- Initial Fit All implementation determines live-cell bounds and centers/scales to show population.

## 2026-10-23 — Status bar and generation metrics (PDF pp. 19–20)

### Added
- Persistent footer showing generation, living-cell count, predicted births/deaths, requested and achieved GPS, zoom, and run state.
- Configurable Home key to fit all living cells.

## 2026-10-23 — Core UX refinements (PDF pp. 18–19)

### Added
- UI-scale slider (applied after closing Settings) and persisted keyboard shortcuts.
- Randomize whole field with configurable X/Y size and alive probability; default **120×90** at **30%**.
- Click-to-select a library pattern without an extra Stamp button.
- Edit/stamp while simulation is running without automatically pausing.

### Fixed
- Replaced unsupported `egui::Context::wants_keyboard_input()` with `egui_wants_keyboard_input()`.

## 2026-10-23 — First compile/API fixes (PDF pp. 12–18)

### Fixed
- Added leading zeroes to float literals (`.75`, `.55`, `.18`, `.08`, `.22`).
- Migrated to compatible `egui` `Panel::top/bottom/left` and `CentralPanel::show(&mut Ui, ...)` APIs.
- Fixed pattern-browser borrow-checker errors (`E0502`) by deferring state mutations until after iterating immutable catalog entries.

## 2026-10-23 — First runnable desktop scaffold (PDF p. 12)

### Added
- Rust/Bevy desktop project with Conway B3/S23, sparse unbounded-world model, pause/play/step/+10/reset/clear/speed controls, camera, drawing and erasing, starter pattern library, RLE import/export, TOML preferences, and tests.
- Initial `README.md`, `DESIGN.md`, and downloadable source ZIP.

### Verification note
- The original package was reported as scaffolded; subsequent user compiler feedback led to corrections above. The PDF does not contain a continuous clean CI history.

## 2026-10-23 — Technical specification and initial concept (PDF pp. 1–11)

### Designed
- Began with an explanation of Conway's Game of Life, original **B3/S23** rules, and still-life/oscillator/spaceship behavior.
- Requested a large navigable 2D field, drag-and-drop library of known patterns, speed/pause controls, and three generations of state.
- Defined Layer 0/current `G`, Layer 1/previous `G−1`, Layer 2/older `G−2`; forecast uses a derived `G+1` but does not change the authoritative state.
- Selected Rust, **Bevy 0.20**, **bevy_egui 0.43.x**, RLE file support, and configurable per-cell birth/living/death color and alpha.
- Specified an extensible pattern catalog, persistence, performance strategy, tests, and separation of simulation from rendering/UI.

---

**Source:** `Describe Conway's Game(1).pdf` (47 pages), supplied with the request to reconstruct the entire history. Page ranges are approximate milestones in the exported discussion, not formal release dates. Implementation claims in this document describe the reported project development; they are not a guarantee of independently verified runtime behavior.
