#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use game_of_life_bevy::{patterns::{self, Pattern}, settings::{Settings,Swatch,CellShape}, simulation::{Cell,CellKind,Cells,Universe,LifeRule}};
use std::{path::PathBuf, sync::{mpsc, Mutex}, time::{SystemTime, UNIX_EPOCH}};
use game_of_life_bevy::outliers::{self, OutlierOptions, OutlierAnalysis};

fn random_seed() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xA0761D6478BD642F) | 1
}

fn random_next(seed: &mut u64) -> f64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed as f64 / u64::MAX as f64
}

#[cfg(target_os = "macos")]
fn set_dock_icon() {
    use objc2::{AnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let main_thread = MainThreadMarker::new().expect("application startup requires the main thread");
    let data = NSData::with_bytes(include_bytes!("../assets/app-icon.png"));
    let image = NSImage::initWithData(NSImage::alloc(), &data).expect("invalid embedded app icon");
    let app = NSApplication::sharedApplication(main_thread);
    // SAFETY: Called on the main thread with a valid, decoded AppKit image.
    unsafe { app.setApplicationIconImage(Some(&image)) };
}

fn main() {
    #[cfg(target_os = "macos")]
    set_dock_icon();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin { primary_window: Some(Window {
            title: "Game of Life".into(),
            resolution: (1500, 950).into(), ..default()
        }), ..default() }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(LifeApp::default())
        .add_systems(Startup, |mut commands: Commands| { commands.spawn(Camera2d); })
        .add_systems(EguiPrimaryContextPass, interface)
        .run();
}

#[derive(Resource)]
struct LifeApp {
    show_license: bool,
    fit_about: bool,
    toolbar_brand: Option<egui::TextureHandle>,
    universe: Universe, initial: Cells, snapshot: Option<Universe>, settings: Settings, running: bool,
    accumulator: f64, actual_gps: f64, measured_steps: u64, measure_time: f64,
    center: egui::Vec2, zoom: f32, patterns: Vec<Pattern>, selected: usize,
    search: String, settings_open: bool, help_open: bool, file_open: bool, about_open: bool, rules_open: bool,
    path_input: String, message: String, stamp: Option<Vec<Cell>>, stamp_rotation: u16, stamp_mirrored: bool,
    stamp_drag_last: Option<(f64, f64)>, drawing: bool, erasing: bool, brush_rng: u64, rectangle_erase: Option<(Cell, Cell)>, rectangle_random: Option<(Cell, Cell)>, applied_ui_scale: f32, viewport_size: egui::Vec2, pending_fit: bool,
    outlier_options: OutlierOptions, outlier_preview: Option<OutlierAnalysis>, outlier_was_running: bool,
    file_picker: Mutex<Option<mpsc::Receiver<Option<PathBuf>>>>,
}
impl Default for LifeApp {
    fn default() -> Self {
        let patterns = patterns::builtin();
        let start = Cells::new();
        let settings = Settings::load();
        let applied_ui_scale = settings.ui_scale.clamp(0.75, 2.5);
        Self { fit_about: true, show_license: false, toolbar_brand: None, universe: Universe::new(start.clone()), initial:start,snapshot:None,settings,
            running:false,accumulator:0.,actual_gps:0.,measured_steps:0,measure_time:0.,
            center:egui::Vec2::ZERO,zoom:15.,patterns,selected:usize::MAX,search:String::new(),
            settings_open:false,help_open:false,file_open:false,about_open:false,rules_open:false,path_input:patterns::pattern_directory().join("puffer1.rle").display().to_string(),
            message:String::new(),stamp:None,stamp_rotation:0,stamp_mirrored:false,stamp_drag_last:None,drawing:false,erasing:false,brush_rng:random_seed(),rectangle_erase:None,rectangle_random:None,applied_ui_scale, viewport_size: egui::vec2(1000., 700.), pending_fit: false, outlier_options: OutlierOptions::default(), outlier_preview: None, outlier_was_running: false, file_picker: Mutex::new(None) }
    }
}
// The native chooser must not block the Bevy/egui frame loop.
// On macOS, osascript runs its own dialog process on a worker thread.
// Other platforms use rfd from the worker thread.
fn choose_pattern_file() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let result = std::process::Command::new("osascript")
            .args(["-e", "POSIX path of (choose file with prompt \"Import a Game of Life pattern (.rle or .mc)\")"])
            .output().ok()?;
        if !result.status.success() { return None; }
        let path = String::from_utf8(result.stdout).ok()?;
        let path = path.trim_end_matches(['\r', '\n']);
        if path.is_empty() { None } else { Some(PathBuf::from(path)) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        rfd::FileDialog::new()
            .add_filter("Life patterns", &["rle", "mc"])
            .pick_file()
    }
}

impl LifeApp {
    fn start_pattern_picker(&mut self) {
        let Ok(mut picker) = self.file_picker.lock() else { return; };
        if picker.is_some() { return; }
        let (sender, receiver) = mpsc::channel();
        *picker = Some(receiver);
        std::thread::spawn(move || {
            let result = choose_pattern_file();
            let _ = sender.send(result);
        });
    }
    fn poll_pattern_picker(&mut self) {
        let result = {
            let Ok(mut picker) = self.file_picker.lock() else { return; };
            let Some(rx) = picker.as_ref() else { return; };
            let polled = rx.try_recv();
            match polled {
                Ok(value) => { *picker = None; Some(value) }
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => { *picker = None; Some(None) }
            }
        };
        if let Some(Some(path)) = result {
            self.path_input = path.to_string_lossy().into_owned();
        }
    }
    fn pattern_picker_open(&self) -> bool {
        self.file_picker.lock().is_ok_and(|picker| picker.is_some())
    }
    // A rule change starts an entirely new universe, not a conversion of existing cells.
    fn switch_rule(&mut self, rule: LifeRule) {
        if rule == self.universe.rule { return; }
        self.running = false;
        self.accumulator = 0.0;
        self.actual_gps = 0.0;
        self.measured_steps = 0;
        self.measure_time = 0.0;
        self.universe = Universe::with_rule(Cells::new(), rule);
        self.initial.clear();
        self.snapshot = None;
        self.outlier_preview = None;
        self.outlier_was_running = false;
        self.stamp = None;
        self.selected = usize::MAX;
        self.stamp_rotation = 0;
        self.stamp_mirrored = false;
        self.stamp_drag_last = None;
        self.drawing = false;
        self.erasing = false;
        self.rectangle_erase = None;
        self.rectangle_random = None;
        self.pending_fit = false;
    }

    fn begin_outlier_preview(&mut self) {
        self.outlier_was_running = self.running;
        self.running = false;
        self.accumulator = 0.0;
        self.outlier_preview = Some(outliers::analyze(&self.universe.current, &self.universe.forecast, self.outlier_options));
    }
    fn cancel_outlier_preview(&mut self) {
        self.outlier_preview = None;
        self.running = self.outlier_was_running;
        self.accumulator = 0.0;
    }
    fn apply_outliers(&mut self) {
        if let Some(result) = self.outlier_preview.take() {
            let removed = result.remove.len();
            if removed > 0 {
                self.universe.current.retain(|p| !result.remove.contains(p));
                self.universe.recompute_forecast();
                self.universe.edited_history = true;
            }
            self.message = format!("Outliers removed: {removed}; retained {} regions", result.regions.len());
            self.pending_fit = true;
        }
        self.running = self.outlier_was_running;
        self.accumulator = 0.0;
    }
    fn fit_all_cells(&mut self) {
        match game_of_life_bevy::simulation::bounding_box(&self.universe.current) {
            None => self.message = "No living cells to fit".into(),
            Some(((min_x, min_y), (max_x, max_y))) => {
                // Calculate with f64 to avoid integer overflow on large coordinates.
                let width = (max_x as f64 - min_x as f64 + 1.0).max(1.0);
                let height = (max_y as f64 - min_y as f64 + 1.0).max(1.0);
                let view_w = f64::from(self.viewport_size.x).max(1.0);
                let view_h = f64::from(self.viewport_size.y).max(1.0);
                const PADDING: f64 = 0.76;
                self.center = egui::vec2(
                    ((min_x as f64 + max_x as f64 + 1.0) / 2.0) as f32,
                    ((min_y as f64 + max_y as f64 + 1.0) / 2.0) as f32,
                );
                self.zoom = ((view_w / width).min(view_h / height) * PADDING)
                    .clamp(0.01, 70.0) as f32;
                self.message = format!("Fit {} living cells", self.universe.current.len());
            }
        }
    }
    fn advance(&mut self, n: usize) { for _ in 0..n { self.universe.advance();self.measured_steps+=1; } }
    fn pause_for_edit(&mut self){self.running=false;self.accumulator=0.;}
    fn randomize(&mut self) {
        let width = self.settings.random_width;
        let height = self.settings.random_height;
        let mut seed = SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_nanos() as u64).unwrap_or(0xCAFE_BABE) | 1;
        let mut cells = Cells::new();
        let origin_x = self.center.x.floor() as i64 - i64::from(width / 2);
        let origin_y = self.center.y.floor() as i64 - i64::from(height / 2);
        for y in 0..height { for x in 0..width {
            seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
            if (seed as f64 / u64::MAX as f64) < f64::from(self.settings.random_density) {
                cells.insert((origin_x + i64::from(x), origin_y + i64::from(y)));
            }
        }}
        self.initial = cells.clone();
        self.universe = Universe::with_rule(cells,self.universe.rule);
        self.accumulator = 0.;
        self.message = format!("Randomized {width} × {height} area");
        self.pending_fit = true;
    }
    fn rotate_stamp(&mut self) {
        if let Some(cells)=self.stamp.as_mut(){*cells=patterns::rotate(cells);self.stamp_rotation=(self.stamp_rotation+90)%360;}
    }
    fn mirror_stamp(&mut self) {
        if let Some(cells)=self.stamp.as_mut(){*cells=patterns::mirror(cells);self.stamp_mirrored=!self.stamp_mirrored;}
    }
    fn stamp_spacing(&self) -> f64 {
        self.stamp.as_deref().map(|cells| {
            let min_x=cells.iter().map(|p|p.0).min().unwrap_or(0);
            let max_x=cells.iter().map(|p|p.0).max().unwrap_or(0);
            let min_y=cells.iter().map(|p|p.1).min().unwrap_or(0);
            let max_y=cells.iter().map(|p|p.1).max().unwrap_or(0);
            let w=(max_x as f64-min_x as f64)+1.0;
            let h=(max_y as f64-min_y as f64)+1.0;
            0.5*w.hypot(h)+f64::from(self.settings.stamp_spacing_extra)
        }).unwrap_or(1.0).max(0.01)
    }
    fn place(&mut self, at:Cell) {
        if let Some(cells)=self.stamp.as_ref() {
            for &(dx,dy) in cells { if let (Some(x),Some(y))=(at.0.checked_add(dx),at.1.checked_add(dy)) {
                self.universe.current.insert((x,y));
            }}
            self.universe.recompute_forecast();
            self.universe.edited_history=true;
        }
    }
    fn read_file(&mut self) {
        match patterns::parse_pattern_file(std::path::Path::new(&self.path_input)) {
            Ok(cells) => {
                let set: Cells = cells.into_iter().collect();
                if let Some((idx, existing)) = self.patterns.iter().enumerate()
                    .find(|(_, p)| p.cells.len() == set.len() && p.cells.iter().all(|c| set.contains(c))) {
                    let name = existing.name.clone();
                    self.selected = idx;
                    self.stamp = Some(existing.cells.clone());
                    self.stamp_rotation = 0;
                    self.stamp_mirrored = false;
                    self.message = format!("Pattern already exists: {name}");
                    self.file_open = false;
                    return;
                }
                let as_pattern = Pattern {
                    name: PathBuf::from(&self.path_input).file_stem().unwrap_or_default().to_string_lossy().into_owned(),
                    category: "Imported".into(),
                    description: "An imported Life pattern. Select it to preview and place its cells on the field.".into(),
                    cells: set.into_iter().collect(),
                };
                self.patterns.push(as_pattern);
                self.selected = self.patterns.len() - 1;
                self.stamp = Some(self.patterns[self.selected].cells.clone());
                self.stamp_rotation = 0;
                self.stamp_mirrored = false;
                self.message = "Imported pattern; click on the field to place it".into();
                self.file_open = false;
            }
            Err(e) => self.message = format!("Import error: {e}"),
        }
    }

}

const SHORTCUT_CHOICES: &[&str] = &["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "Enter", "A", "D", "E", "F", "J", "K", "L", "M", "P", "R", "S", "T", "W"];
fn key_from_name(name: &str) -> Option<egui::Key> {
    Some(match name {
        "Space"=>egui::Key::Space,"ArrowUp"=>egui::Key::ArrowUp,"ArrowDown"=>egui::Key::ArrowDown,
        "ArrowLeft"=>egui::Key::ArrowLeft,"ArrowRight"=>egui::Key::ArrowRight,"Home"=>egui::Key::Home,"Enter"=>egui::Key::Enter,
        "A"=>egui::Key::A,"D"=>egui::Key::D,"E"=>egui::Key::E,"F"=>egui::Key::F,
        "J"=>egui::Key::J,"K"=>egui::Key::K,"L"=>egui::Key::L,"P"=>egui::Key::P,
        "M"=>egui::Key::M,"R"=>egui::Key::R,"S"=>egui::Key::S,"T"=>egui::Key::T,"W"=>egui::Key::W,
        _=>return None,
    })
}
fn wheel_speed(current:f32,up:bool)->f32 {
    let value=if up {current+1.0}else{current-1.0};
    value.clamp(0.25,240.0)
}
fn next_speed(current: f32, faster: bool) -> f32 {
    const SPEEDS: [f32; 14] = [0.25,0.5,1.,2.,4.,8.,12.,15.,20.,30.,60.,90.,120.,240.];
    if faster { SPEEDS.iter().copied().find(|s| *s > current+0.001).unwrap_or(240.) }
    else { SPEEDS.iter().rev().copied().find(|s| *s < current-0.001).unwrap_or(0.25) }
}
// Shift+Up moves to the next integer; Shift+Down to the previous integer.
// Using floor/ceil also makes fractional speeds snap correctly: 1.3 -> 2 or 1.
fn next_unit_speed(current: f32, faster: bool) -> f32 {
    (if faster { (current + 0.0001).floor() + 1.0 }
    else { (current - 0.0001).ceil() - 1.0 }).clamp(0.25, 240.0)
}

fn shortcut_editor(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(label).selected_text(value.as_str()).show_ui(ui, |ui| {
            for &choice in SHORTCUT_CHOICES { ui.selectable_value(value,choice.to_owned(),choice); }
        });
    });
}

fn handle_game_shortcuts(ctx: &egui::Context, state: &mut LifeApp) {
    // Pattern modifiers always cancel placement, allowing brush and navigation gestures.
    let modifiers = ctx.input(|i| i.modifiers);
    if modifiers.shift || modifiers.alt || modifiers.ctrl {
        state.stamp = None;
        state.selected = usize::MAX;
        state.stamp_rotation = 0;
        state.stamp_mirrored = false;
    }
    let typing = ctx.egui_wants_keyboard_input();
    let keys = state.settings.keys.clone();
    let pressed = |name: &str| key_from_name(name)
        .is_some_and(|key| ctx.input(|i| i.key_pressed(key)));
    if !typing && !modifiers.ctrl && !modifiers.alt && !modifiers.command {
        if pressed(&keys.toggle_run) { state.running = !state.running; state.accumulator = 0.; }
        if pressed(&keys.faster) {
            state.settings.speed = if modifiers.shift { next_unit_speed(state.settings.speed, true) }
                else { next_speed(state.settings.speed, true) };
        }
        if pressed(&keys.slower) {
            state.settings.speed = if modifiers.shift { next_unit_speed(state.settings.speed, false) }
                else { next_speed(state.settings.speed, false) };
        }
        if state.outlier_preview.is_none() && pressed(&keys.fit_all) { state.pending_fit = true; }
        if state.stamp.is_some() {
            if ctx.input(|i| i.key_pressed(egui::Key::R)) { state.rotate_stamp(); }
            if ctx.input(|i| i.key_pressed(egui::Key::M)) { state.mirror_stamp(); }
        }
    }
    if state.outlier_preview.is_some() && !typing && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        state.apply_outliers();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        if state.settings_open { if let Err(e)=state.settings.save(){eprintln!("Failed to save settings: {e}");} }
        if state.outlier_preview.is_some() { state.cancel_outlier_preview(); }
        state.stamp = None;
        state.selected = usize::MAX;
        state.stamp_rotation = 0;
        state.stamp_mirrored = false;
        state.rectangle_random = None;
        state.rectangle_erase = None;
        state.settings_open = false;
        state.file_open = false;
        state.rules_open = false;
    }
    // F1 remains available even when a text field currently has focus.
    if ctx.input(|i| i.key_pressed(egui::Key::F1)) { state.help_open = !state.help_open; }
    if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
        state.settings.show_library = !state.settings.show_library;
        if !state.settings.show_library { state.stamp = None; state.selected = usize::MAX; }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::F3)) { state.settings_open = !state.settings_open; if !state.settings_open { if let Err(e)=state.settings.save(){eprintln!("Failed to save settings: {e}");} } }
    if ctx.input(|i| i.key_pressed(egui::Key::F4)) { state.rules_open = !state.rules_open; }
}

fn interface(mut contexts:EguiContexts, mut state:ResMut<LifeApp>, time:Res<Time>) -> Result {
    let ctx=contexts.ctx_mut()?;
    state.poll_pattern_picker();
    if !state.settings_open && (state.applied_ui_scale - state.settings.ui_scale).abs() > 0.001 {
        state.applied_ui_scale = state.settings.ui_scale.clamp(0.75, 2.5);
    }
    ctx.set_zoom_factor(state.applied_ui_scale);
    if !state.about_open {
        handle_game_shortcuts(ctx, &mut state);
    }
    let dt=time.delta_secs_f64().min(0.25);
    if state.running && state.outlier_preview.is_none() {
        state.accumulator+=dt;
        let interval=1.0 / state.settings.speed.clamp(0.25,240.) as f64;
        let mut steps=0;
        while state.accumulator>=interval && steps<8 {
            state.universe.advance();state.accumulator-=interval;steps+=1;state.measured_steps+=1;
        }
        if steps==8 {state.accumulator=state.accumulator.min(interval);}
    }
    state.measure_time+=dt;
    if state.measure_time>=1.0 {
        state.actual_gps=state.measured_steps as f64/state.measure_time;
        state.measured_steps=0;state.measure_time=0.;
    }
    if state.running {ctx.request_repaint();}

    // Constrain the root layout to the current window, rather than permitting a
    // content-sized Area to extend below the bottom edge when resized.
    let window_rect = ctx.content_rect();
    egui::Area::new(egui::Id::new("game_root"))
        .fixed_pos(window_rect.min)
        .constrain(false)
        .order(egui::Order::Background)
        .show(ctx, |root| {
    root.set_min_size(window_rect.size());
    root.set_max_width(window_rect.width());
    root.set_max_height(window_rect.height());
    egui::Panel::top("top").show(root,|ui|{
        ui.horizontal_wrapped(|ui|{
            let brand = state.toolbar_brand.get_or_insert_with(|| {
                let rgba = image::load_from_memory(include_bytes!("../assets/toolbar-brand.png"))
                    .expect("invalid embedded toolbar artwork").into_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                ui.ctx().load_texture("toolbar-brand", egui::ColorImage::from_rgba_unmultiplied(size, &rgba), egui::TextureOptions::LINEAR)
            });
            let image = egui::Image::new((brand.id(), egui::vec2(170.0, 40.0)));
            if ui.add(egui::Button::image(image.alt_text("Game of Life")).frame(false))
                .on_hover_text("About Game of Life").clicked() {
                state.fit_about = true;
                state.show_license = false;
                state.about_open = true;
            }
            ui.separator();
            if ui.button(if state.running{"⏸ Pause"}else{"▶ Run"}).on_hover_text("Toggle playback (Space)").clicked(){state.running=!state.running;state.accumulator=0.;}
            if ui.button("Step").on_hover_text("Advance one generation").clicked(){state.pause_for_edit();state.advance(1);}
            if ui.button("+10").on_hover_text("Advance ten generations").clicked(){state.pause_for_edit();state.advance(10);}
            ui.label("Generations/sec").on_hover_text("Requested simulation speed");
            let before_slider = state.settings.speed;
            let speed_response=ui.add(egui::Slider::new(&mut state.settings.speed,0.25..=240.).logarithmic(true).fixed_decimals(1)).on_hover_text("Scroll to change speed by exactly 1 generation/sec");
            if speed_response.hovered() && !speed_response.dragged() {
                let scroll=ui.input(|i|i.smooth_scroll_delta.y);
                if scroll.abs()>0.1 {
                    // The logarithmic Slider may consume scrolling itself. Ignore that
                    // adjustment and use the speed from before rendering the widget.
                    state.settings.speed=wheel_speed(before_slider,scroll>0.0);
                }
            }
            ui.separator();
            if ui.button("Fit All (Enter)").on_hover_text("Fit the entire living population").clicked(){state.pending_fit=true;}
            ui.separator();
            if ui.button("Clear").on_hover_text("Pause and remove all living cells").clicked(){state.running=false;state.accumulator=0.0;state.universe=Universe::with_rule(Cells::new(),state.universe.rule);state.initial=Cells::new();}
            if ui.button("Randomize").on_hover_text("Replace world with random cells").clicked(){state.randomize();}
            if ui.add_enabled(state.outlier_preview.is_none(),egui::Button::new("Erase Outliers")).on_hover_text("Preview outlier cleanup and fit the population").clicked(){state.begin_outlier_preview();state.pending_fit=true;}
            ui.separator();
            if ui.button("Snapshot").on_hover_text("Save the current universe in the single snapshot slot, replacing its previous contents").clicked(){state.snapshot=Some(state.universe.clone());}
            if ui.add_enabled(state.snapshot.is_some(),egui::Button::new("Restore Snapshot")).on_hover_text("Restore saved universe and pause playback").clicked(){
                if let Some(snapshot)=state.snapshot.clone(){state.running=false;state.accumulator=0.0;state.universe=snapshot;state.pending_fit=true;}
            }
            ui.separator();
            if ui.button("Help (F1)").on_hover_text("Toggle help panel").clicked(){state.help_open=!state.help_open;}
            if ui.button("Patterns (F2)").on_hover_text("Toggle pattern library").clicked(){state.settings.show_library=!state.settings.show_library;if !state.settings.show_library{state.stamp=None;state.selected=usize::MAX;}}
            if ui.button("Settings (F3)").on_hover_text("Toggle settings; saves preferences on close").clicked(){state.settings_open=!state.settings_open; if !state.settings_open {if let Err(e)=state.settings.save(){eprintln!("Failed to save settings: {e}");}}}
            if ui.button("Rules (F4)").on_hover_text("Select the cellular automaton rule set (F4)").clicked(){state.rules_open=!state.rules_open;}
        });
    });
    egui::Panel::bottom("status_bar").resizable(false).default_size(27.).show(root,|ui|{
        ui.set_min_height(23.);
        ui.set_max_height(23.);
        ui.painter().rect_filled(ui.max_rect(),0.,egui::Color32::from_rgb(29,37,49));
        let (born,dying)=state.universe.transition_counts();
        let net_change = game_of_life_bevy::simulation::net_change(born, dying, state.universe.current.len());
        // Reserve the right-side telemetry before giving the population stats the remaining width.
        // This keeps Target/Actual GPS, Zoom and Run state anchored during number changes.
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x = 5.0;
            let right_width = 365.0_f32.min(ui.available_width());
            let left_width = (ui.available_width() - right_width - 6.0).max(0.0);
            ui.allocate_ui_with_layout(egui::vec2(left_width,23.0),egui::Layout::left_to_right(egui::Align::Center),|ui|{
                egui::ScrollArea::horizontal().id_salt("status_scroll").show(ui,|ui|{
                    ui.horizontal(|ui|{
                ui.strong(state.universe.rule.name()).on_hover_text(state.universe.rule.notation()); ui.separator();
                ui.label(egui::RichText::new(format!("GEN {}",group_int(state.universe.generation))).strong()).on_hover_text("Current generation index");ui.separator();
                ui.label(format!("Alive {}",group_int(state.universe.current.len()))).on_hover_text("Current living cell population");ui.separator();
                ui.label(format!("Born {} · Dying {}",group_int(born),group_int(dying))).on_hover_text("Predicted transitions in the next generation");ui.separator();
                let net_color=if net_change.delta>0 {egui::Color32::LIGHT_GREEN} else if net_change.delta<0 {egui::Color32::from_rgb(255,100,72)} else {egui::Color32::LIGHT_GRAY};
                let net_label=match net_change.rate_percent {
                    Some(rate)=>format!("{} ({:+.2}%)",signed_group_int(net_change.delta),rate),
                    None=>format!("{} (N/A)",signed_group_int(net_change.delta)),
                };
                ui.horizontal(|ui|{
                    ui.add_space(4.0);
                    ui.label("Net change");
                    ui.allocate_ui_with_layout(egui::vec2(14.0,18.0),egui::Layout::left_to_right(egui::Align::Center),|ui|{
                        // Geometric arrow: reliable on macOS regardless of the selected UI font.
                        let (r,_) = ui.allocate_exact_size(egui::vec2(14.0,14.0),egui::Sense::hover());
                        let c=r.center();
                        let points=if net_change.delta>0 {
                            vec![egui::pos2(c.x,c.y-5.0),egui::pos2(c.x-5.0,c.y+4.0),egui::pos2(c.x+5.0,c.y+4.0)]
                        }else if net_change.delta<0 {
                            vec![egui::pos2(c.x,c.y+5.0),egui::pos2(c.x-5.0,c.y-4.0),egui::pos2(c.x+5.0,c.y-4.0)]
                        }else {
                            vec![egui::pos2(c.x+5.0,c.y),egui::pos2(c.x-4.0,c.y-5.0),egui::pos2(c.x-4.0,c.y+5.0)]
                        };
                        ui.painter().add(egui::Shape::convex_polygon(points,net_color,egui::Stroke::NONE));
                    });
                    ui.add(egui::Label::new(egui::RichText::new(net_label).color(net_color).monospace()))
                        .on_hover_text("Forecast: births minus deaths; percent of current living population");
                    ui.add_space(4.0);
                });
                        if state.universe.edited_history {ui.label("History edited").on_hover_text("History may reflect manual edits");}
                    });
                });
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                ui.colored_label(if state.running{egui::Color32::LIGHT_GREEN}else{egui::Color32::YELLOW},if state.running{"▶ RUNNING"}else{"⏸ PAUSED"}).on_hover_text("Simulation playback state");
                ui.separator();
                ui.label(format!("Zoom {:.2} px/cell",state.zoom)).on_hover_text("Current viewport scale");
                ui.separator();
                ui.label(format!("Target {:.1} · Actual {:.1} GPS",state.settings.speed,state.actual_gps)).on_hover_text("Requested and achieved generations per second");
            });
        });
    });
    if state.settings.show_library {
        egui::Panel::left("catalog").resizable(true).default_size(245.).show(root,|ui|{
            ui.horizontal(|ui|{ui.heading("Pattern library");ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                if ui.small_button("◀").on_hover_text("Collapse the pattern library").clicked(){state.settings.show_library=false;state.stamp=None;state.selected=usize::MAX;}
            });});ui.text_edit_singleline(&mut state.search).on_hover_text("Filter patterns by name or category");
            ui.label("Select once; click or drag into the field to stamp repeatedly. Esc cancels placement.");
            ui.separator();
            egui::ScrollArea::vertical().id_salt("pattern_browser_scroll").auto_shrink([false,false]).show(ui,|ui|{
                let query = state.search.to_lowercase();
                let mut action: Option<(usize, bool)> = None;
                let mut last_category: Option<&str> = None;
                for (idx, pattern) in state.patterns.iter().enumerate() {
                    if !format!("{} {}", pattern.name, pattern.category)
                        .to_lowercase()
                        .contains(&query)
                    {
                        continue;
                    }
                    if last_category != Some(pattern.category.as_str()) {
                        if last_category.is_some(){ui.add_space(8.0);ui.separator();}
                        ui.label(egui::RichText::new(&pattern.category).strong().color(egui::Color32::from_rgb(155,196,232)));
                        ui.add_space(4.0);
                        last_category=Some(pattern.category.as_str());
                    }
                    let response = ui.selectable_label(
                        idx == state.selected,
                        pattern.name.as_str(),
                    ).on_hover_text(format!("Select {} for repeated placement",pattern.name));
                    if response.drag_started() || response.clicked() {
                        action = Some((idx, true));
                    }
                }
                // Apply state changes after the immutable catalog iteration ends.
                if let Some((idx, start_stamp)) = action {
                    state.selected = idx;
                    if start_stamp {
                        state.stamp = Some(state.patterns[idx].cells.clone());
                        state.stamp_rotation=0;state.stamp_mirrored=false;
                    }
                }
            ui.separator();
            if ui.button("Import Pattern…").on_hover_text("Import an RLE or Macrocell (.mc) pattern into the library").clicked(){state.file_open=true;}
            ui.add_space(8.0);
            ui.label(egui::RichText::new("More patterns online").strong());
            ui.label("LifeWiki maintains a public collection of Conway's Game of Life patterns. Browse and download .rle or .mc files, then use Import Pattern to add them here.");
            ui.hyperlink_to("Browse LifeWiki patterns", "https://conwaylife.com/patterns/")
                .on_hover_text("Open https://conwaylife.com/patterns/ in your web browser");
            ui.separator();
            if state.stamp.is_some() { if let Some(p)=state.patterns.get(state.selected){
                ui.strong(&p.name);
                ui.label(format!("{} · {} cells",p.category,p.cells.len()));
                ui.add_space(7.0);
                ui.label(egui::RichText::new(&p.description).color(egui::Color32::from_gray(190)));
                ui.add_space(5.0);
            }}
            if let Some(cells)=state.stamp.as_ref(){
                ui.add_space(8.0);
                ui.strong("Placement preview");
                ui.label(format!("Rotation: {}°  |  Mirrored: {}",state.stamp_rotation,if state.stamp_mirrored{"Yes"}else{"No"}));
                let preview_cells=cells.clone();
                pattern_preview(ui,&preview_cells);
                ui.add_space(6.0);
                ui.horizontal(|ui|{
                    if ui.button("Rotate 90° (R)").on_hover_text("Rotate the active stamp clockwise by 90 degrees").clicked(){state.rotate_stamp();}
                    if ui.button("Mirror (M)").on_hover_text("Mirror the active stamp horizontally").clicked(){state.mirror_stamp();}
                });
                if ui.button("Cancel placement (Esc)").on_hover_text("Exit placement mode, leaving the simulation unchanged").clicked(){state.stamp=None;state.selected=usize::MAX;state.stamp_rotation=0;state.stamp_mirrored=false;}
            }
            });
        });
    }
    if state.help_open {
        egui::Panel::right("help_dock").resizable(true).default_size(325.).show(root,|ui|{
            ui.horizontal(|ui|{
                if ui.small_button("▶").on_hover_text("Collapse Help (F1)").clicked(){state.help_open=false;}
                ui.heading("Help");
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui,|ui|{help_contents(ui);});
        });
    }
    egui::CentralPanel::default().frame(egui::Frame::NONE).show(root,|ui|{ field(ui,&mut state); });
    });
    if state.outlier_preview.is_some() {outlier_window(ctx,&mut state);}
    if state.rules_open {rules_window(ctx,&mut state);}
    if state.settings_open {settings_window(ctx,&mut state);}
    if state.file_open {file_window(ctx,&mut state);}
    if state.about_open {about_window(ctx,&mut state);}

    Ok(())
}

fn pattern_preview(ui:&mut egui::Ui,cells:&[Cell]) {
    let size=egui::vec2(ui.available_width().min(230.0).max(90.0),150.0);
    let (rect,_)=ui.allocate_exact_size(size,egui::Sense::hover());
    let painter=ui.painter_at(rect);
    painter.rect_filled(rect,4.0,egui::Color32::from_rgb(20,26,36));
    if cells.is_empty(){return;}
    let (mut min_x,mut min_y,mut max_x,mut max_y)=(i64::MAX,i64::MAX,i64::MIN,i64::MIN);
    for &(x,y) in cells {min_x=min_x.min(x);min_y=min_y.min(y);max_x=max_x.max(x);max_y=max_y.max(y);}
    let width=(max_x as f64-min_x as f64+1.0) as f32;
    let height=(max_y as f64-min_y as f64+1.0) as f32;
    let cell_size=((rect.width()-16.0)/width).min((rect.height()-16.0)/height).clamp(0.25,18.0);
    let x0=rect.center().x-width*cell_size*0.5;
    let y0=rect.center().y-height*cell_size*0.5;
    for &(x,y) in cells {
        let pos=egui::pos2(x0+(x-min_x) as f32*cell_size,y0+(y-min_y) as f32*cell_size);
        painter.rect_filled(egui::Rect::from_min_size(pos,egui::vec2((cell_size-0.5).max(0.25),(cell_size-0.5).max(0.25))),0.0,egui::Color32::from_rgb(0,217,255));
    }
}

fn color(s:&Swatch)->egui::Color32{egui::Color32::from_rgba_unmultiplied(s.rgb[0],s.rgb[1],s.rgb[2],(s.alpha.clamp(0.,1.)*255.) as u8)}
// Keep the world coordinate under the pointer fixed when the zoom level changes.
fn zoom_center_at_pointer(center: egui::Vec2, old_zoom: f32, new_zoom: f32, pointer: egui::Pos2, viewport_center: egui::Pos2) -> egui::Vec2 {
    if (new_zoom - old_zoom).abs() < f32::EPSILON { return center; }
    let offset = pointer - viewport_center;
    center + offset * (1.0 / old_zoom - 1.0 / new_zoom)
}

// Border width must remain within the cell, including at minimum zoom.
fn square_frame_thickness(side: f32) -> f32 {
    (side * 0.22).max(0.0).min(side * 0.5)
}

fn paint_cell(painter:&egui::Painter,origin:egui::Pos2,zoom:f32,color:egui::Color32,shape:CellShape){
    let side=(zoom-0.7).max(0.5);
    let r=egui::Rect::from_min_size(origin,egui::vec2(side,side));
    let center=r.center();
    let radius=side*0.46;
    match shape {
        CellShape::Filled=>{painter.rect_filled(r,0.,color);},
        CellShape::Circle=>{painter.circle_filled(center,radius,color);},
        CellShape::OutlinedSquare=>{
            // Draw only the frame; the center remains transparent.
            let thickness=square_frame_thickness(side);
            painter.rect_stroke(r,0.,egui::Stroke::new(thickness,color),egui::StrokeKind::Inside);
        },
        CellShape::Diamond=>{
            let pts=vec![egui::pos2(center.x,center.y-radius),egui::pos2(center.x+radius,center.y),egui::pos2(center.x,center.y+radius),egui::pos2(center.x-radius,center.y)];
            painter.add(egui::Shape::convex_polygon(pts,color,egui::Stroke::NONE));
        },
        CellShape::Hexagon=>{
            let pts=(0..6).map(|i|{let a=(i as f32)*std::f32::consts::TAU/6.;egui::pos2(center.x+radius*a.cos(),center.y+radius*a.sin())}).collect();
            painter.add(egui::Shape::convex_polygon(pts,color,egui::Stroke::NONE));
        },
        CellShape::Star=>{
            let pts:Vec<_>=(0..10).map(|i|{let a=(i as f32)*std::f32::consts::PI/5.-std::f32::consts::FRAC_PI_2;let rad=if i%2==0 {radius} else {radius*0.42};egui::pos2(center.x+rad*a.cos(),center.y+rad*a.sin())}).collect();
            for i in 0..10 {painter.add(egui::Shape::convex_polygon(vec![center,pts[i],pts[(i+1)%10]],color,egui::Stroke::NONE));}
        },
        CellShape::Cross=>{
            let thickness=(side*0.26).max(0.5);
            painter.rect_filled(egui::Rect::from_center_size(center,egui::vec2(thickness,side)),0.,color);
            painter.rect_filled(egui::Rect::from_center_size(center,egui::vec2(side,thickness)),0.,color);
        },
    }
}

fn field(ui:&mut egui::Ui,state:&mut LifeApp){
    let available=ui.available_size();
    let (rect,response)=ui.allocate_exact_size(available,egui::Sense::click_and_drag());
    state.viewport_size = rect.size();
    // Perform fit after top/side/bottom panels have been laid out. This makes
    // the centering and zoom use the current viewport, including after resize.
    if state.pending_fit {
        state.pending_fit = false;
        state.fit_all_cells();
    }
    let painter=ui.painter_at(rect);
    let bg=state.settings.background;
    painter.rect_filled(rect,0.,egui::Color32::from_rgb(bg[0],bg[1],bg[2]));
    let hovered=ui.input(|i|i.pointer.hover_pos()).filter(|p|!state.about_open && rect.contains(*p));
    if let Some(pointer) = hovered.filter(|_| response.hovered()) {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.01 {
            let new_zoom = (state.zoom * (scroll * 0.0015).exp()).clamp(0.01, 70.0);
            state.center = zoom_center_at_pointer(state.center, state.zoom, new_zoom, pointer, rect.center());
            state.zoom = new_zoom;
        }
    }
    // Use updated camera values for both input hit-testing and drawing this frame.
    let zoom=state.zoom;
    let viewport_center=state.center;
    let to_screen=move |p:Cell|egui::pos2(rect.center().x + (p.0 as f32-viewport_center.x)*zoom,rect.center().y + (p.1 as f32-viewport_center.y)*zoom);
    let from_screen=move |pos:egui::Pos2| -> Cell {(((pos.x-rect.center().x)/zoom+viewport_center.x).floor() as i64,((pos.y-rect.center().y)/zoom+viewport_center.y).floor() as i64)};
    let modifiers=ui.input(|i|i.modifiers);
    let middle=ui.input(|i|i.pointer.middle_down());
    if middle && response.hovered() {
        let delta=ui.input(|i|i.pointer.delta());
        state.center-=delta/state.zoom;
    }
    let moving=middle||modifiers.alt;
    if moving || state.stamp.is_none() {state.stamp_drag_last=None;}
    let primary=ui.input(|i|i.pointer.primary_down());
    let secondary=ui.input(|i|i.pointer.secondary_down());
    // Ctrl + primary drag takes precedence over stamping and the normal brush.
    // Keep the selection in world coordinates so camera changes do not affect it.
    let pressed=ui.input(|i|i.pointer.primary_pressed());
    let released=ui.input(|i|i.pointer.primary_released());
    let pointer=ui.input(|i|i.pointer.interact_pos());
    // Alt + primary drag defines a rectangular region in world-cell coordinates.
    // The region is filled on release using the Randomize density preference.
    if state.outlier_preview.is_none() && state.rectangle_random.is_none()
        && response.hovered() && pressed && primary && modifiers.alt {
        if let Some(pos) = pointer {
            let cell = from_screen(pos);
            state.rectangle_random = Some((cell, cell));
        }
    }
    if let Some((start, _)) = state.rectangle_random {
        if let Some(pos) = pointer {
            let clamped = egui::pos2(pos.x.clamp(rect.left(), rect.right() - 0.01),
                pos.y.clamp(rect.top(), rect.bottom() - 0.01));
            state.rectangle_random = Some((start, from_screen(clamped)));
        }
        if released || !primary {
            if let Some((a, b)) = state.rectangle_random.take() {
                let width = a.0.abs_diff(b.0).saturating_add(1);
                let height = a.1.abs_diff(b.1).saturating_add(1);
                // Avoid locking the UI if a huge rectangle is selected at low zoom.
                const MAX_RANDOM_CELLS: u64 = 2_000_000;
                if width.saturating_mul(height) > MAX_RANDOM_CELLS {
                    state.message = format!("Random region too large (maximum {MAX_RANDOM_CELLS} cells)");
                } else {
                    let mut updates = Vec::with_capacity((width * height) as usize);
                    for y in a.1.min(b.1)..=a.1.max(b.1) {
                        for x in a.0.min(b.0)..=a.0.max(b.0) {
                            let live = random_next(&mut state.brush_rng) < f64::from(state.settings.random_density);
                            updates.push(((x,y),live));
                        }
                    }
                    state.universe.paint_cells(updates);
                    state.message = format!("Randomized {width} × {height} cell region");
                }
            }
        }
    }
    if state.outlier_preview.is_none() && state.rectangle_erase.is_none() && state.rectangle_random.is_none() && response.hovered() && pressed && primary && modifiers.ctrl && !moving {
        if let Some(pos)=pointer {
            let cell=from_screen(pos);
            state.rectangle_erase=Some((cell,cell));
        }
    }
    if state.outlier_preview.is_none() {
    if let Some((start, _))=state.rectangle_erase {
        if let Some(pos)=pointer {
            // Clamp outside-window drags to the visible field bounds.
            let pos=egui::pos2(pos.x.clamp(rect.left(),rect.right()-0.01),pos.y.clamp(rect.top(),rect.bottom()-0.01));
            state.rectangle_erase=Some((start,from_screen(pos)));
        }
        if released || !primary {
            if let Some((a,b))=state.rectangle_erase.take() {
                let removed=state.universe.erase_rectangle(a,b);
                state.message=format!("Erased {removed} living cells in rectangle");
            }
        }
    } else if state.rectangle_random.is_none() && response.hovered()&&!moving {
        if let Some(pointer)=hovered {
            let at=from_screen(pointer);
            if state.stamp.is_some(){
                // Stamp once on press, then at distance-based intervals along the drag.
                // Interpolate between samples so fast mouse movement does not skip stamps.
                if primary {
                    let current=(at.0 as f64,at.1 as f64);
                    if pressed || state.stamp_drag_last.is_none() {
                        state.place(at);
                        state.stamp_drag_last=Some(current);
                    } else if let Some(last)=state.stamp_drag_last {
                        let spacing=state.stamp_spacing();
                        let dx=current.0-last.0;
                        let dy=current.1-last.1;
                        let dist=dx.hypot(dy);
                        let steps=(dist/spacing).floor().min(1024.0) as usize;
                        for j in 1..=steps {
                            let t=(j as f64*spacing)/dist;
                            let x=last.0+dx*t;
                            let y=last.1+dy*t;
                            state.place((x.round() as i64,y.round() as i64));
                        }
                        if steps>0 {
                            let t=(steps as f64*spacing)/dist;
                            state.stamp_drag_last=Some((last.0+dx*t,last.1+dy*t));
                        }
                    }
                }
            } else if primary || secondary {
                let write=primary;
                if state.drawing != primary || state.erasing != secondary || response.dragged() || response.clicked(){
                    let size=if modifiers.shift {state.settings.shift_brush_size.clamp(1,25) as i64} else {1};
                    let start=-(size/2);
                    let mut updates = Vec::with_capacity((size * size) as usize);
                    for dy in start..start+size {
                        for dx in start..start+size {
                            if let (Some(x),Some(y))=(at.0.checked_add(dx),at.1.checked_add(dy)) {
                                // Shift + left-drag paints a random distribution, not a solid square.
                                // Empty samples clear existing cells so density is meaningful.
                                let alive = if write && modifiers.shift {
                                    random_next(&mut state.brush_rng) < f64::from(state.settings.random_density)
                                } else { write };
                                updates.push(((x,y),alive));
                            }
                        }
                    }
                    state.universe.paint_cells(updates);
                }
                state.drawing=primary;state.erasing=secondary;
            }
        }
    }
    }
    if !primary {state.stamp_drag_last=None;}
    if !primary&&!secondary {state.drawing=false;state.erasing=false;}
    let origin_x=(state.center.x-(rect.width()/zoom)/2.).floor() as i64-1;
    let max_x=(state.center.x+(rect.width()/zoom)/2.).ceil() as i64+1;
    let origin_y=(state.center.y-(rect.height()/zoom)/2.).floor() as i64-1;
    let max_y=(state.center.y+(rect.height()/zoom)/2.).ceil() as i64+1;
    // Avoid allocating/rendering a dense grid when zoomed far out. Cells remain visible.
    {
        if state.settings.grid.visible && zoom>=7. && (max_x-origin_x).saturating_mul(max_y-origin_y) <= 450_000 {
            let gc=color(&state.settings.grid);
            for x in origin_x..=max_x {let px=to_screen((x,0)).x;painter.line_segment([egui::pos2(px,rect.top()),egui::pos2(px,rect.bottom())],egui::Stroke::new(1.,gc));}
            for y in origin_y..=max_y {let py=to_screen((0,y)).y;painter.line_segment([egui::pos2(rect.left(),py),egui::pos2(rect.right(),py)],egui::Stroke::new(1.,gc));}
        }
        let draw=|p:Cell,c:egui::Color32,shape:CellShape|{
            if p.0<origin_x||p.0>max_x||p.1<origin_y||p.1>max_y{return;}
            let point=to_screen(p);
            paint_cell(&painter,point,zoom,c,shape);
        };
        if state.settings.live_only {
            if state.settings.living.visible {
                for &p in &state.universe.current {draw(p,color(&state.settings.living),state.settings.living.shape);}
            }
        } else {
        if state.universe.rule==LifeRule::BriansBrain && state.settings.dying.visible {
            for &p in &state.universe.refractory {draw(p,color(&state.settings.dying),state.settings.dying.shape);}
        }
        if state.settings.trail_2.visible {for &p in &state.universe.older{draw(p,color(&state.settings.trail_2),CellShape::Filled);}}
        if state.settings.trail_1.visible {for &p in &state.universe.previous{draw(p,color(&state.settings.trail_1),CellShape::Filled);}}
        let candidates:Cells=if state.settings.history_mode{
            state.universe.current.union(&state.universe.previous).copied().collect()
        }else{state.universe.current.union(&state.universe.forecast).copied().collect()};
        for p in candidates {
            let kind=if state.settings.history_mode{state.universe.history_kind(&p)}else{state.universe.forecast_kind(&p)};
            let swatch=match kind{Some(CellKind::Birth)=>&state.settings.birth,Some(CellKind::Living)=>&state.settings.living,Some(CellKind::Dying)=>&state.settings.dying,None=>continue};
            if swatch.visible{draw(p,color(swatch),swatch.shape);}
        }
        }
        // Proposed removals are red. The analysis is non-destructive until confirmed.
        if let Some(preview) = &state.outlier_preview {
            for &cell in &preview.remove { draw(cell,egui::Color32::from_rgba_unmultiplied(255, 75, 65, 215),CellShape::Filled); }
            // Draw per-cell red frames around all cells proposed for deletion.
            for &cell in &preview.remove {
                let pt=to_screen(cell);
                let box_rect=egui::Rect::from_min_size(pt,egui::vec2(zoom.max(0.5),zoom.max(0.5)));
                painter.rect_stroke(box_rect,0.,egui::Stroke::new(1.25,egui::Color32::from_rgb(255,70,70)),egui::StrokeKind::Inside);
            }
            for (lo,hi) in outliers::removal_group_bounds(&preview.remove, state.outlier_options.radius) {
                let bounds=egui::Rect::from_min_max(to_screen(lo),to_screen(hi)+egui::vec2(zoom,zoom));
                painter.rect_stroke(bounds,0.,egui::Stroke::new(2.0,egui::Color32::from_rgb(255,75,65)),egui::StrokeKind::Outside);
            }
            for region in &preview.regions {
                let (lo,hi)=region.bounds;
                let bounds=egui::Rect::from_min_max(to_screen(lo),to_screen(hi) + egui::vec2(zoom,zoom));
                painter.rect_stroke(bounds,0.,egui::Stroke::new(1.5,egui::Color32::from_rgb(90,225,150)),egui::StrokeKind::Inside);
            }
        }
        // An erase cursor is rendered even before dragging to show the active footprint.
        if state.rectangle_erase.is_none() && (secondary || modifiers.shift) && !moving {
            if let Some(pointer)=hovered {
                let at=from_screen(pointer);
                let size=if modifiers.shift {state.settings.shift_brush_size.clamp(1,25) as i64} else {1};
                let start=-(size/2);
                let top_left=to_screen((at.0+start,at.1+start));
                let brush_rect=egui::Rect::from_min_size(top_left,egui::vec2(zoom*size as f32,zoom*size as f32));
                painter.rect_filled(brush_rect,0.,egui::Color32::from_rgba_unmultiplied(if secondary {255} else {80},if secondary {85} else {175},if secondary {70} else {255},43));
                painter.rect_stroke(brush_rect,0.,egui::Stroke::new(1.5,egui::Color32::from_rgba_unmultiplied(if secondary {255} else {80},if secondary {126} else {175},if secondary {106} else {255},210)),egui::StrokeKind::Inside);
            }
        }
        if let Some((a,b))=state.rectangle_random {
            let top_left=to_screen((a.0.min(b.0),a.1.min(b.1)));
            let bottom_right=to_screen((a.0.max(b.0),a.1.max(b.1)));
            let box_rect=egui::Rect::from_min_max(top_left,bottom_right+egui::vec2(zoom,zoom));
            painter.rect_filled(box_rect,0.,egui::Color32::from_rgba_unmultiplied(60,185,255,35));
            painter.rect_stroke(box_rect,0.,egui::Stroke::new(1.5,egui::Color32::from_rgb(80,190,255)),egui::StrokeKind::Inside);
        }
        if let Some((a,b))=state.rectangle_erase {
            let min_x=a.0.min(b.0); let max_x=a.0.max(b.0);
            let min_y=a.1.min(b.1); let max_y=a.1.max(b.1);
            let top_left=to_screen((min_x,min_y));
            let bottom_right=to_screen((max_x,max_y));
            let box_rect=egui::Rect::from_min_max(top_left, bottom_right + egui::vec2(zoom,zoom));
            painter.rect_filled(box_rect,0.,egui::Color32::from_rgba_unmultiplied(255,85,70,32));
            painter.rect_stroke(box_rect,0.,egui::Stroke::new(1.5,egui::Color32::from_rgb(255,115,85)),egui::StrokeKind::Inside);
        }
        if state.rectangle_erase.is_none() && state.rectangle_random.is_none() {
        if let (Some(cells),Some(pos))=(&state.stamp,hovered){
            let at=from_screen(pos);
            for &(dx,dy) in cells {
                if let (Some(x),Some(y))=(at.0.checked_add(dx),at.1.checked_add(dy)){draw((x,y),egui::Color32::from_rgba_unmultiplied(250,245,150,120),CellShape::Filled);}
            }
        }
        }
    }
}

fn group_int<T: std::fmt::Display>(number:T)->String {
    let s=number.to_string();
    let mut output=String::new();
    for (i,c) in s.chars().rev().enumerate(){
        if i>0 && i%3==0 {output.push(',');}
        output.push(c);
    }
    output.chars().rev().collect()
}
fn signed_group_int(value:i128)->String {
    if value>=0 {format!("+{}",group_int(value))} else {format!("-{}",group_int(value.unsigned_abs()))}
}
fn swatch_editor(ui:&mut egui::Ui,label:&str,s:&mut Swatch){
    ui.horizontal(|ui|{
        ui.checkbox(&mut s.visible,label);
        ui.color_edit_button_srgb(&mut s.rgb);
        ui.add(egui::Slider::new(&mut s.alpha,0.0..=1.0).text("opacity").fixed_decimals(2));
        if label != "Grid" {
            egui::ComboBox::from_id_salt(format!("cell-shape-{label}"))
                .selected_text(s.shape.label()).show_ui(ui,|ui|{
                    for shape in CellShape::ALL {ui.selectable_value(&mut s.shape,shape,shape.label());}
                });
        }
    });
}
fn outlier_window(ctx:&egui::Context,state:&mut LifeApp){
    let mut apply = false;
    let mut cancel = false;
    let mut refresh = false;
    let mut open = true;
    egui::Window::new("Erase Outliers")
        .open(&mut open).collapsible(false).resizable(true).default_width(410.)
        .show(ctx,|ui|{
            ui.label("The simulation is paused during analysis. Red cells will be removed; green outlines show retained masses.");
            ui.separator();
            ui.heading("Detection settings");
            refresh |= ui.add(egui::Slider::new(&mut state.outlier_options.radius,2..=64).text("Radius (cells)")).changed();
            refresh |= ui.add(egui::Slider::new(&mut state.outlier_options.min_neighbors,2..=20).text("Local population")).changed();
            refresh |= ui.add(egui::Slider::new(&mut state.outlier_options.min_mass_percent,1.0..=30.0).text("Minimum mass %")).changed();
            refresh |= ui.add(egui::Slider::new(&mut state.outlier_options.padding,0..=32).text("Padding (cells)")).changed();
            refresh |= ui.checkbox(&mut state.outlier_options.multiple_masses,"Preserve multiple major masses").changed();
            refresh |= ui.checkbox(&mut state.outlier_options.include_births,"Include predicted births in detection").changed();
            if let Some(result)=&state.outlier_preview {
                ui.separator();
                ui.strong(format!("{} regions retained",result.regions.len()));
                ui.label(format!("Remove {} of {} living cells ({:.2}%)",result.remove.len(),result.population,
                    100.0 * result.remove.len() as f64 / result.population.max(1) as f64));
                if result.regions.is_empty() { ui.colored_label(egui::Color32::YELLOW,"No major mass detected; nothing will be removed."); }
            }
            ui.add_space(8.0);
            ui.horizontal(|ui|{
                if ui.button("Apply removal").clicked(){apply=true;}
                if ui.button("Cancel (Esc)").clicked(){cancel=true;}
            });
        });
    if refresh {state.outlier_preview=Some(outliers::analyze(&state.universe.current,&state.universe.forecast,state.outlier_options));}
    if apply {state.apply_outliers();}
    if cancel || !open {state.cancel_outlier_preview();}
}

fn rules_window(ctx:&egui::Context,state:&mut LifeApp) {
    let mut open=state.rules_open;
    egui::Window::new("Rules").collapsible(false).open(&mut open).resizable(false).default_width(440.).show(ctx,|ui|{
        ui.label("Select the cellular automaton used for future generations.");
        ui.small("Changing rules clears the board, generation history, and saved snapshot. The new rule starts paused at generation 0.");
        ui.separator();
        let mut chosen=None;
        for rule in LifeRule::ALL {
            ui.horizontal(|ui|{
                if ui.radio(state.universe.rule==rule,format!("{} ({})",rule.name(),rule.notation())).clicked(){chosen=Some(rule);}
            });
            ui.indent(format!("desc_{rule:?}"),|ui|{ui.small(rule.description());});
            ui.add_space(7.0);
        }
        if let Some(rule)=chosen {
            state.switch_rule(rule);
        }
        if state.universe.rule==LifeRule::BriansBrain {
            ui.separator();ui.small("Brian's Brain: ON → refractory → OFF. Refractory cells are colored using the dying-cell palette.");
        }
    });
    state.rules_open=open;
}

fn settings_window(ctx:&egui::Context,state:&mut LifeApp){
    let mut open=state.settings_open;
    egui::Window::new("Settings").collapsible(false).open(&mut open).resizable(true).default_width(430.).default_height(620.).show(ctx,|ui|{
        egui::ScrollArea::vertical().id_salt("settings_scroll").auto_shrink([false,false]).show(ui,|ui|{
        ui.heading("Cell-state colors and transparency");
        swatch_editor(ui,"Birthing",&mut state.settings.birth);
        swatch_editor(ui,"Living",&mut state.settings.living);
        swatch_editor(ui,"Dying",&mut state.settings.dying);
        ui.separator();ui.heading("Three-generation display");
        ui.checkbox(&mut state.settings.live_only, "Show only currently living cells").on_hover_text("Disable colored birth/death forecasts and history trails; draw current live cells using the Living swatch");
        ui.horizontal(|ui|{
            ui.radio_value(&mut state.settings.history_mode,false,"Forecast G → G+1").on_hover_text("Predicted next step: cyan = will be born, green = will survive, red = will die");
            ui.radio_value(&mut state.settings.history_mode,true,"History G−1 → G").on_hover_text("Completed previous step: cells born, survived or died between G−1 and G");
        });
        ui.small("Forecast shows what WILL happen next. History shows what JUST happened.");
        ui.small("G−1 and G−2 trails overlay faint cell positions from one and two generations ago; they do not affect evolution.");
        swatch_editor(ui,"G−1 trail",&mut state.settings.trail_1);
        swatch_editor(ui,"G−2 trail",&mut state.settings.trail_2);
        swatch_editor(ui,"Grid",&mut state.settings.grid);
        ui.horizontal(|ui|{ui.label("Background");ui.color_edit_button_srgb(&mut state.settings.background);});
        ui.separator();ui.heading("Interface scale");
        ui.add(egui::Slider::new(&mut state.settings.ui_scale,0.75..=2.5).text("UI scale").fixed_decimals(2));
        ui.small("Applies when Settings is closed. The dialog stays at its current scale while you adjust it.");
        ui.separator();ui.heading("Random field");
        ui.add(egui::Slider::new(&mut state.settings.random_width,8..=512).text("Width (X)"));
        ui.add(egui::Slider::new(&mut state.settings.random_height,8..=512).text("Height (Y)"));
        let mut percent = state.settings.random_density * 100.0;
        if ui.add(egui::Slider::new(&mut percent,0.0..=100.0).text("Live cells (%)").fixed_decimals(1)).changed() {
            state.settings.random_density = percent / 100.0;
        }
        ui.separator();ui.heading("Pattern drag stamping");
        ui.add(egui::Slider::new(&mut state.settings.stamp_spacing_extra,0..=100).text("Extra spacing (cells)"));
        ui.small("Left-drag stamps repeatedly every half pattern diagonal + extra spacing cells. Default extra: 15.");
        ui.separator();ui.heading("Drawing brush");
        ui.add(egui::Slider::new(&mut state.settings.shift_brush_size,1..=25).text("Shift brush width (cells)"));
        ui.small("Shift + left-drag uses the Randomize alive probability above.");
        ui.small("Shift + left-drag paints random live cells; Shift + right-drag erases. Alt + left-drag randomizes a selected rectangle; middle-drag pans.");
        ui.small("Randomize clears the entire universe and populates a rectangle centered on the viewport.");
        ui.separator();ui.heading("Keyboard shortcuts");
        shortcut_editor(ui,"Run / pause",&mut state.settings.keys.toggle_run);
        shortcut_editor(ui,"Increase speed",&mut state.settings.keys.faster);
        shortcut_editor(ui,"Decrease speed",&mut state.settings.keys.slower);
        shortcut_editor(ui,"Fit all living cells",&mut state.settings.keys.fit_all);

        ui.label("Pattern library: F2");
        ui.label("Settings: F3");
        ui.label("Rules: F4");
        ui.separator();
        ui.horizontal(|ui|{
            if ui.button("Restore defaults").on_hover_text("Restore application settings to defaults").clicked(){state.settings=Settings::default();}
            if ui.button("Reset history to now").on_hover_text("Copy the current living cells into both history layers (G−1 and G−2), clearing historical trails and edited-history status without changing the current population or generation").clicked(){state.universe.reset_history();}
        });
        ui.label("Color changes take effect immediately. Preferences save automatically when Settings closes.");
        });
    });
    if !open { if let Err(e)=state.settings.save(){eprintln!("Failed to save settings: {e}");} }
    state.settings_open=open;
}
fn file_window(ctx:&egui::Context,state:&mut LifeApp){
    let mut open=state.file_open;
    egui::Window::new("Import Pattern (RLE / Macrocell)").collapsible(false).open(&mut open).show(ctx,|ui|{
        ui.label("Import standard Conway Life patterns (.rle or .mc Macrocell).");
        ui.label("Macrocell is a compressed quadtree format for large patterns.");
        ui.add_space(6.);
        ui.label("Absolute or working-directory-relative .rle / .mc file path:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.path_input);
            if ui.add_enabled(!state.pattern_picker_open(), egui::Button::new("Browse…"))
                .on_hover_text("Open a native file chooser without blocking the simulation").clicked() {
                state.start_pattern_picker();
            }
        });
        if state.pattern_picker_open() { ui.label("File chooser open…"); }
        ui.horizontal(|ui|{
            if ui.button("Import into library").on_hover_text("Load a .rle or .mc pattern from the specified file").clicked(){state.read_file();}

        });
        ui.label("Imported patterns are added to the browser for placement.");
    });
    state.file_open &= open;
}
fn about_window(ctx: &egui::Context, state: &mut LifeApp) {
    let mut open = state.about_open;
    let mut dismissed = false;
    let id = egui::Id::new("about_modal");
    let layer = egui::LayerId::new(egui::Order::Foreground, id);
    ctx.memory_mut(|memory| memory.set_modal_layer(layer));
    ctx.layer_painter(layer).rect_filled(ctx.content_rect(), 0.0, egui::Color32::from_black_alpha(95));
    let bounds = ctx.content_rect();
    let fit = std::mem::take(&mut state.fit_about);
    let compact = egui::vec2(450.0, 412.0).min(bounds.size());
    let centered = bounds.center() - compact * 0.5;
    let previous_rect = ctx.memory(|memory| memory.area_rect(id));
    let was_showing_license = state.show_license;
    let mut license_height = 0.0;
    let mut window = egui::Window::new("About Game of Life")
        .id(id)
        .order(layer.order)
        .pivot(egui::Align2::LEFT_TOP)
        .default_pos(centered)
        .default_size(compact)
        .min_size([265.0, 160.0])
        .collapsible(false)
        .movable(true)
        .resizable(true)
        .vscroll(true)
        .open(&mut open);
    if fit {
        window = window.current_pos(centered).fixed_size(compact);
    } else if let Some((origin, size)) = ctx.data_mut(|data| data.remove_temp::<(egui::Pos2, egui::Vec2)>(id.with("expanded_rect"))) {
        window = window.current_pos(origin).fixed_size(size);
    }
    window.show(ctx, |ui| {
            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                if let Some(brand) = &state.toolbar_brand {
                    let width = ui.available_width().min(300.0);
                    ui.image((brand.id(), egui::vec2(width, width * 40.0 / 170.0)));
                }
            });
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            ui.heading("Game of Life");
            ui.weak(concat!("Version ", include_str!("../../VERSION")).trim_end());
            ui.add_space(10.0);
            ui.label("Draw cells or place patterns and watch them evolve under the rules of John Horton Conway’s 1970 classic, the Game of Life, or one of four other cellular automata.");
            ui.add_space(12.0);
            egui::Grid::new("about_details").spacing([18.0, 5.0]).show(ui, |ui| {
                for (label, value) in [("Developer", "Randel Dodson"), ("Copyright", "© 2026 Randel Dodson"), ("License", "MIT License")] {
                    ui.weak(label);
                    ui.label(value);
                    ui.end_row();
                }
            });
            ui.add_space(16.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button(if state.show_license { "Hide license" } else { "View license…" }).clicked() {
                    state.show_license = !state.show_license;
                    state.fit_about = !state.show_license;
                }
                ui.hyperlink_to("Project repository…", env!("CARGO_PKG_REPOSITORY"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    dismissed = ui.add_sized([76.0, ui.spacing().interact_size.y], egui::Button::new("OK")).clicked();
                });
            });
            if state.show_license {
                ui.add_space(8.0);
                license_height = ui.group(|ui| { ui.monospace(include_str!("../../LICENSE")); }).response.rect.height();
            }
        });
    if !was_showing_license && state.show_license {
        if let Some(rect) = previous_rect {
            let available_height = bounds.bottom() - rect.top();
            let height = (rect.height().max(compact.y) + license_height + 12.0).min(available_height);
            let expanded = (rect.min, egui::vec2(rect.width(), height));
            ctx.data_mut(|data| data.insert_temp(id.with("expanded_rect"), expanded));
            ctx.request_repaint();
        }
    }
    state.about_open = open && !dismissed && !ctx.input(|i| i.key_pressed(egui::Key::Escape));
}

fn help_contents(ui:&mut egui::Ui) {
            ui.set_width(ui.available_width());
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Quick reference for navigating, editing, and controlling the simulation.").italics());

            ui.add_space(10.0);
            ui.group(|ui|{
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Mouse controls").strong());
                ui.add_space(6.0);
                ui.label("• Left-click / drag: draw live cells");
                ui.label("• Right-click / drag: erase cells");
                ui.label("• Ctrl + left-drag: preview a rectangle; release to erase all cells inside");
                ui.label("• Shift + left-drag: random cells within the larger brush; Shift + right-drag: erase the full area");
                ui.label("• Alt + left-drag: select a rectangle and randomize its cells on release");
                ui.label("• Middle drag: pan the viewport");
                ui.label("• Mouse wheel: zoom toward pointer");
                ui.label("• Erasing displays a red translucent brush preview");
            });

            ui.add_space(10.0);
            ui.group(|ui|{
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Patterns").strong());
                ui.add_space(6.0);
                ui.label("• Click a library pattern to make it the active stamp");
                ui.label("• Hold left mouse and drag to repeat stamps at distance-based intervals");
                ui.label("• The selection remains active for repeated placement");
                ui.label("• Press Esc to cancel placement");
                ui.label("• R rotates 90°; M mirrors horizontally (also available as buttons)");
            });

            ui.add_space(10.0);
            ui.group(|ui|{
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Simulation controls").strong());
                ui.add_space(6.0);
                ui.label("• Use the toolbar to run/pause, step, +10, clear, or randomize");
                ui.label("• Fit All is available from the toolbar or by pressing Enter");
                ui.label("• Up / Down: move to the next higher or lower speed preset");
                ui.label("• Shift + Up / Shift + Down: adjust speed by 1, snapping fractional speeds to whole numbers");
                ui.label("• Editing and stamping continue while the simulation is running");
            });

            ui.add_space(10.0);
            ui.group(|ui|{
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Settings").strong());
                ui.add_space(6.0);
                ui.label("• F1 toggles this docked Help panel");
                ui.label("• F1: Help; F2: Pattern Library; F3: Settings; F4: Rules");
                ui.label("• Click the Game of Life logo for About; Esc closes About or cancels a stamp");
                ui.label("• R: rotate selected pattern; M: mirror selected pattern");
                ui.label("• Shift / Alt / Ctrl cancels any selected stamp");
                ui.label("• Keyboard shortcuts are customizable in Settings");
                ui.label("• Cell shapes can be selected individually for births, living, and dying");
                ui.label("• Colors, transparency, trails, visualization mode, UI scale, and randomize area are configurable");
                ui.label("• Settings affect appearance and controls without changing the Game of Life rules");
                ui.add_space(10.0);
                ui.separator();
                ui.hyperlink_to("Conway's Game of Life — Wikipedia", "https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life");
                ui.label("Background on the history, rules, and notable structures in Conway's Game of Life.");
            });
}

#[cfg(test)]
mod control_tests {
    use super::*;

    #[test]
    fn speed_wheel_adds_exactly_one() {
        assert_eq!(wheel_speed(1.3,true),2.3);
        assert_eq!(wheel_speed(5.0,false),4.0);
        assert_eq!(wheel_speed(0.5,false),0.25);
    }

    #[test]
    fn shift_speed_snaps_to_adjacent_integer() {
        assert_eq!(next_unit_speed(1.3, true), 2.0);
        assert_eq!(next_unit_speed(1.3, false), 1.0);
        assert_eq!(next_unit_speed(4.5, true), 5.0);
        assert_eq!(next_unit_speed(4.5, false), 4.0);
        assert_eq!(next_unit_speed(2.0, true), 3.0);
        assert_eq!(next_unit_speed(2.0, false), 1.0);
    }

    #[test]
    fn normal_speed_keeps_presets() {
        assert_eq!(next_speed(4.5, true), 8.0);
        assert_eq!(next_speed(1.3, true), 2.0);
        assert_eq!(next_speed(8.0, false), 4.0);
    }
}

#[cfg(test)]
mod zoom_anchor_tests {
    use super::*;

    #[test]
    fn zoom_keeps_pointer_world_position_fixed() {
        let center = egui::vec2(12.5, -8.0);
        let screen_center = egui::pos2(500.0, 300.0);
        let pointer = egui::pos2(650.0, 180.0);
        let before = center + (pointer - screen_center) / 10.0;
        let updated = zoom_center_at_pointer(center, 10.0, 25.0, pointer, screen_center);
        let after = updated + (pointer - screen_center) / 25.0;
        assert!((before - after).length() < 0.0001);
    }

    #[test]
    fn zoom_at_viewport_center_does_not_pan() {
        let center = egui::vec2(200.0, -90.0);
        let pointer = egui::pos2(300.0, 200.0);
        assert_eq!(zoom_center_at_pointer(center, 5.0, 15.0, pointer, pointer), center);
    }
}

#[cfg(test)]
mod square_frame_regression_tests {
    use super::square_frame_thickness;

    #[test]
    fn tiny_cells_never_have_an_invalid_border_range() {
        for side in [0.5_f32, 0.51, 0.75, 1.0, 2.0, 10.0, 70.0] {
            let thickness = square_frame_thickness(side);
            assert!(thickness.is_finite());
            assert!(thickness >= 0.0);
            assert!(thickness <= side / 2.0);
        }
    }
}

#[cfg(test)]
mod formatting_tests {
    use super::*;
    #[test]
    fn groups_large_counts() {
        assert_eq!(group_int(1_234_567_u64), "1,234,567");
        assert_eq!(signed_group_int(12345), "+12,345");
        assert_eq!(signed_group_int(-12345), "-12,345");
    }
}

#[cfg(test)]
mod rule_switch_reset_tests {
    use super::*;

    #[test]
    fn switching_rules_clears_all_population_and_snapshot_state() {
        let mut app = LifeApp::default();
        app.universe = Universe::with_rule([(0, 0), (1, 0), (2, 0)].into(), LifeRule::Conway);
        app.universe.advance();
        app.snapshot = Some(app.universe.clone());
        app.initial.insert((100, 100));
        app.running = true;
        app.accumulator = 1.0;
        app.stamp = Some(vec![(0, 0)]);
        app.selected = 0;
        app.switch_rule(LifeRule::BriansBrain);

        assert_eq!(app.universe.rule, LifeRule::BriansBrain);
        assert_eq!(app.universe.generation, 0);
        assert!(app.universe.current.is_empty());
        assert!(app.universe.previous.is_empty());
        assert!(app.universe.older.is_empty());
        assert!(app.universe.forecast.is_empty());
        assert!(app.universe.refractory.is_empty());
        assert!(app.universe.next_refractory.is_empty());
        assert!(app.snapshot.is_none());
        assert!(app.initial.is_empty());
        assert!(!app.running);
        assert_eq!(app.accumulator, 0.0);
        assert!(app.stamp.is_none());
        assert_eq!(app.selected, usize::MAX);
    }

    #[test]
    fn reselecting_same_rule_preserves_simulation() {
        let mut app = LifeApp::default();
        app.universe = Universe::with_rule([(0, 0)].into(), LifeRule::HighLife);
        app.snapshot = Some(app.universe.clone());
        app.switch_rule(LifeRule::HighLife);
        assert_eq!(app.universe.current.len(), 1);
        assert!(app.snapshot.is_some());
    }
}

#[cfg(test)]
mod empty_startup_tests {
    use super::*;

    #[test]
    fn new_application_starts_with_an_empty_paused_board() {
        let app = LifeApp::default();
        assert!(app.universe.current.is_empty());
        assert!(app.universe.previous.is_empty());
        assert!(app.universe.older.is_empty());
        assert!(app.universe.forecast.is_empty());
        assert!(app.universe.refractory.is_empty());
        assert!(app.initial.is_empty());
        assert!(app.snapshot.is_none());
        assert_eq!(app.universe.generation, 0);
        assert!(!app.running);
    }
}
