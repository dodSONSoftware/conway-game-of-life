use game_of_life_bevy::{simulation::{Cells,Universe,CellKind,evolve},patterns::{parse_rle,encode_rle,rotate,mirror},settings::Settings};
fn cells(pts:&[(i64,i64)])->Cells{pts.iter().copied().collect()}
#[test]fn block_is_stable(){let c=cells(&[(0,0),(0,1),(1,0),(1,1)]);assert_eq!(evolve(&c),c);}
#[test]fn blinker_period_two(){let c=cells(&[(0,0),(1,0),(2,0)]);assert_eq!(evolve(&evolve(&c)),c);}
#[test]fn glider_moves_after_four(){let mut c=cells(&[(1,0),(2,1),(0,2),(1,2),(2,2)]);let first=c.clone();for _ in 0..4{c=evolve(&c);}assert_eq!(c,first.iter().map(|(x,y)|(x+1,y+1)).collect());}
#[test]fn snapshots_rotate(){let c=cells(&[(0,0),(1,0),(2,0)]);let mut u=Universe::new(c.clone());let next=evolve(&c);u.advance();assert_eq!(u.current,next);assert_eq!(u.previous,c);assert_eq!(u.older,c);assert_eq!(u.generation,1);u.advance();assert_eq!(u.older,c);assert_eq!(u.previous,next);}
#[test]fn classifications(){let u=Universe::new(cells(&[(0,0),(1,0),(2,0)]));assert_eq!(u.forecast_kind(&(1,-1)),Some(CellKind::Birth));assert_eq!(u.forecast_kind(&(1,0)),Some(CellKind::Living));assert_eq!(u.forecast_kind(&(0,0)),Some(CellKind::Dying));}
#[test]fn rle_roundtrip(){let p=cells(&[(0,0),(1,0),(1,1),(2,2)]);let txt=encode_rle(&p);assert_eq!(cells(&parse_rle(&txt).unwrap()),p);}
#[test]fn transform_preserves_population(){let p=vec![(0,0),(1,0),(1,1)];assert_eq!(rotate(&p).len(),3);assert_eq!(mirror(&p).len(),3);}
#[test]fn palette_serialization(){let s=Settings::default();let encoded=toml::to_string(&s).unwrap();let decoded:Settings=toml::from_str(&encoded).unwrap();assert_eq!(decoded.birth.rgb,s.birth.rgb);}

#[test]
fn new_settings_defaults() {
    let settings = Settings::default();
    assert_eq!(settings.keys.toggle_run, "Space");
    assert_eq!(settings.keys.faster, "ArrowUp");
    assert_eq!(settings.keys.slower, "ArrowDown");
    assert_eq!((settings.random_width, settings.random_height), (120, 90));
    assert_eq!(settings.ui_scale, 1.35);
    assert_eq!(settings.shift_brush_size, 5);
}

#[test]
fn legacy_settings_have_backward_compatible_defaults() {
    let settings: Settings = toml::from_str("speed = 30.0\nshow_library = true\n").unwrap();
    assert_eq!(settings.speed, 30.0);
    assert_eq!(settings.random_density, 0.30);
    assert_eq!(settings.keys.toggle_run, "Space");
}

#[test]
fn fit_bounds_include_all_live_cells() {
    use game_of_life_bevy::simulation::bounding_box;
    assert_eq!(bounding_box(&Cells::new()), None);
    assert_eq!(bounding_box(&cells(&[(5, -10), (-20, 18), (2, 3)])), Some(((-20, -10), (5, 18))));
}

#[test]
fn fit_key_has_backward_compatible_default() {
    let settings: Settings = toml::from_str("speed = 20.0\n[keys]\ntoggle_run = 'Space'\n").unwrap();
    assert_eq!(settings.keys.fit_all, "Enter");
}

#[test]
fn cell_shapes_serde_backward_compatibility() {
    use game_of_life_bevy::settings::{CellShape, Swatch};
    let swatch: Swatch = toml::from_str("rgb = [1, 2, 3]\nalpha = 0.5\nvisible = true").unwrap();
    assert_eq!(swatch.shape, CellShape::Filled);
    for shape in CellShape::ALL {
        let mut swatch = swatch.clone();
        swatch.shape = shape;
        let encoded = toml::to_string(&swatch).unwrap();
        let decoded: Swatch = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.shape, shape);
    }
}

#[test]
fn display_mode_defaults_to_three_layers_and_random_density_is_thirty_percent() {
    let settings = game_of_life_bevy::settings::Settings::default();
    assert!(!settings.live_only);
    assert!((settings.random_density - 0.30).abs() < f32::EPSILON);
    let text = toml::to_string(&settings).expect("serialize settings");
    let recovered: game_of_life_bevy::settings::Settings = toml::from_str(&text).expect("deserialize settings");
    assert!(!recovered.live_only);
}

#[test]
fn universe_is_not_bounded_by_the_visible_window() {
    use game_of_life_bevy::simulation::{Cells, Universe};
    let cells: Cells = [(1_000_000_000, 1_000_000_000), (1_000_000_001, 1_000_000_000), (1_000_000_002, 1_000_000_000)].into_iter().collect();
    let mut universe = Universe::new(cells);
    universe.advance();
    assert!(universe.current.contains(&(1_000_000_001, 999_999_999)));
}

#[test]
fn single_slot_snapshot_restores_full_generation_history() {
    let start = cells(&[(0,0),(1,0),(2,0)]);
    let mut world = Universe::new(start);
    world.advance();
    let snapshot = world.clone();
    world.advance();
    world.current.clear();
    world = snapshot.clone();
    assert_eq!(world.generation, snapshot.generation);
    assert_eq!(world.current, snapshot.current);
    assert_eq!(world.previous, snapshot.previous);
    assert_eq!(world.older, snapshot.older);
    assert_eq!(world.forecast, snapshot.forecast);
}
