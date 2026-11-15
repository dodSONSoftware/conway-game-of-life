use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CellShape {
    #[default]
    Filled,
    Circle,
    OutlinedSquare,
    Diamond,
    Hexagon,
    Star,
    Cross,
}
impl CellShape {
    pub const ALL: [Self; 7] = [Self::Filled, Self::Circle, Self::OutlinedSquare,
        Self::Diamond, Self::Hexagon, Self::Star, Self::Cross];
    pub fn label(self) -> &'static str {
        match self {
            Self::Filled => "Filled", Self::Circle => "Circle",
            Self::OutlinedSquare => "Square frame", Self::Diamond => "Diamond",
            Self::Hexagon => "Hexagon", Self::Star => "Star", Self::Cross => "Cross",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Swatch { pub rgb: [u8;3], pub alpha: f32, pub visible: bool, pub shape: CellShape }
impl Default for Swatch { fn default() -> Self { Self{rgb:[255,255,255],alpha:1.,visible:true,shape:CellShape::Filled} } }
impl Swatch { pub fn new(rgb:[u8;3],alpha:f32,visible:bool)->Self{Self{rgb,alpha,visible,shape:CellShape::Filled}} }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Shortcuts { pub toggle_run: String, pub faster: String, pub slower: String, pub fit_all: String, pub reset: String, pub patterns: String, pub settings: String }
impl Default for Shortcuts {
    fn default() -> Self { Self { toggle_run: "Space".into(), faster: "ArrowUp".into(), slower: "ArrowDown".into(), fit_all: "Enter".into(), reset: "".into(), patterns: "F2".into(), settings: "F3".into() } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub birth: Swatch, pub living: Swatch, pub dying: Swatch,
    pub trail_1: Swatch, pub trail_2: Swatch, pub grid: Swatch,
    pub background: [u8;3], pub history_mode: bool, pub live_only: bool,
    pub speed: f32, pub show_library: bool,
    pub ui_scale: f32, pub random_width: u32, pub random_height: u32,
    pub random_density: f32, pub shift_brush_size: u32, pub stamp_spacing_extra: u32, pub keys: Shortcuts,
}
impl Default for Settings {
    fn default() -> Self { Self {
        birth: Swatch { shape: CellShape::Circle, ..Swatch::new([0,214,255],0.75,true) }, living: Swatch::new([54,229,122],1.,true),
        dying: Swatch { shape: CellShape::OutlinedSquare, ..Swatch::new([255,100,72],0.55,true) },trail_1:Swatch::new([100,141,255],0.18,false),
        trail_2:Swatch::new([155,128,232],0.08,false),grid:Swatch::new([53,64,80],0.25,true),
        background:[16,20,29],history_mode:false,live_only:false,speed:5.,show_library:true,
        ui_scale:1.35,random_width:120,random_height:90,random_density:0.30,shift_brush_size:5,stamp_spacing_extra:15,keys:Shortcuts::default(),
    } }
}
pub fn config_dir() -> PathBuf {
    if let Some(root)=std::env::var_os("XDG_CONFIG_HOME") { return PathBuf::from(root).join("game-of-life-bevy"); }
    if cfg!(target_os="macos") { if let Some(home)=std::env::var_os("HOME") { return PathBuf::from(home).join("Library/Application Support/game-of-life-bevy"); } }
    if cfg!(target_os="windows") { if let Some(root)=std::env::var_os("APPDATA") { return PathBuf::from(root).join("game-of-life-bevy"); } }
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config/game-of-life-bevy")
}
impl Settings {
    pub fn load() -> Self { fs::read_to_string(config_dir().join("settings.toml")).ok().and_then(|s|toml::from_str(&s).ok()).unwrap_or_default() }
    pub fn save(&self)->Result<(),String>{let root=config_dir();fs::create_dir_all(&root).map_err(|e|e.to_string())?;let data=toml::to_string_pretty(self).map_err(|e|e.to_string())?;fs::write(root.join("settings.toml"),data).map_err(|e|e.to_string())}
}
