use crate::simulation::{Cell, Cells};

#[derive(Clone, Debug)]
pub struct Pattern { pub name: String, pub category: String, pub description: String, pub cells: Vec<Cell> }

pub fn parse_rle(source: &str) -> Result<Vec<Cell>, String> {
    let mut body = String::new();
    let mut has_header = false;
    let mut width = None;
    let mut height = None;
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if !has_header {
            if !line.starts_with('x') { return Err("missing RLE header".into()); }
            for part in line.split(',') {
                let Some((key, value)) = part.split_once('=') else { continue; };
                let parsed = value.trim().split_whitespace().next().unwrap_or("");
                match key.trim() {
                    "x" => width = parsed.parse::<i64>().ok(),
                    "y" => height = parsed.parse::<i64>().ok(),
                    _ => {}
                }
            }
            has_header = true;
            continue;
        }
        body.push_str(line);
    }
    let (w,h) = (width.ok_or("invalid x")?, height.ok_or("invalid y")?);
    if !(1..=100_000).contains(&w) || !(1..=100_000).contains(&h) { return Err("invalid dimensions".into()); }
    let (mut x,mut y,mut run) = (0i64,0i64,0i64);
    let mut cells = Vec::new();
    let mut ended = false;
    for ch in body.chars() {
        if ch.is_ascii_digit() {
            run = run.checked_mul(10).and_then(|n| n.checked_add(ch.to_digit(10)? as i64)).ok_or("run length overflow")?;
            continue;
        }
        let n = if run == 0 { 1 } else { run };
        run = 0;
        match ch {
            'b' | 'o' => {
                if y >= h || x.checked_add(n).is_none_or(|v| v > w) { return Err("RLE exceeds declared dimensions".into()); }
                if ch == 'o' {
                    if cells.len().saturating_add(n as usize) > 2_000_000 { return Err("pattern exceeds 2 million cells".into()); }
                    for i in 0..n { cells.push((x+i,y)); }
                }
                x += n;
            }
            '$' => { y = y.checked_add(n).ok_or("row overflow")?; x = 0; if y >= h { return Err("RLE exceeds declared height".into()); } }
            '!' => { ended = true; break; }
            c if c.is_whitespace() => (),
            _ => return Err(format!("invalid RLE symbol: {ch}")),
        }
    }
    if !ended { return Err("missing ! terminator".into()); }
    Ok(cells)
}

pub fn encode_rle(cells: &Cells) -> String {
    if cells.is_empty() { return "x = 1, y = 1, rule = B3/S23\nb!\n".into(); }
    let min_x = cells.iter().map(|p| p.0).min().unwrap();
    let max_x = cells.iter().map(|p| p.0).max().unwrap();
    let min_y = cells.iter().map(|p| p.1).min().unwrap();
    let max_y = cells.iter().map(|p| p.1).max().unwrap();
    let mut body = String::new();
    for y in min_y..=max_y {
        if y != min_y { body.push('$'); }
        let mut last = min_x;
        for x in min_x..=max_x {
            if cells.contains(&(x,y)) {
                while last < x { body.push('b'); last += 1; }
                body.push('o'); last += 1;
            }
        }
    }
    body.push('!');
    format!("x = {}, y = {}, rule = B3/S23\n{}\n", max_x-min_x+1, max_y-min_y+1, body)
}

pub fn builtin() -> Vec<Pattern> {
    const ITEMS: &[(&str, &str, &str)] = &[
        ("Block", "Still lifes", "x = 2, y = 2\n2o$2o!"),
        ("Beehive", "Still lifes", "x = 4, y = 3\nb2o$o2bo$b2o!"),
        ("Loaf", "Still lifes", "x = 4, y = 4\nb2o$o2bo$bobo$2bo!"),
        ("Boat", "Still lifes", "x = 3, y = 3\n2o$obo$bo!"),
        ("Tub", "Still lifes", "x = 3, y = 3\nbo$obo$bo!"),
        ("Blinker", "Oscillators", "x = 3, y = 1\n3o!"),
        ("Toad", "Oscillators", "x = 4, y = 2\nb3o$3o!"),
        ("Beacon", "Oscillators", "x = 4, y = 4\n2o2b$2o2b$2b2o$2b2o!"),
        ("Pulsar", "Oscillators", "x = 13, y = 13\n2b3o3b3o$13b$o4bobo4bo$o4bobo4bo$o4bobo4bo$2b3o3b3o$13b$2b3o3b3o$o4bobo4bo$o4bobo4bo$o4bobo4bo$13b$2b3o3b3o!"),
        ("Glider", "Spaceships", "x = 3, y = 3\nbo$2bo$3o!"),
        ("LWSS", "Spaceships", "x = 5, y = 4\nbo2bo$4o$o3bo$4o!"),
        ("R-pentomino", "Methuselahs", "x = 3, y = 3\nb2o$2o$bo!"),
        ("Diehard", "Methuselahs", "x = 8, y = 3\n6bo$2o$bo3b3o!"),
        ("Acorn", "Methuselahs", "x = 7, y = 3\nbo$3bo$2o2b3o!"),
        ("Gosper glider gun", "Guns", "x = 36, y = 9\n24bo11b$22bobo11b$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o14b$2o8bo3bob2o4bobo11b$10bo5bo7bo11b$11bo3bo20b$12b2o!"),
    ];
    let mut result: Vec<Pattern> = ITEMS.iter().filter_map(|(name, cat, text)| parse_rle(text).ok().map(|cells| Pattern {
        name: (*name).into(),
        category: (*cat).into(),
        description: builtin_description(name).into(),
        cells,
    })).collect();
    result.extend(load_catalog_patterns());
    const ORDER: &[&str] = &["Still lifes","Oscillators","Spaceships","Guns","Eaters","Puffers","Rakes","Breeders","Methuselahs","Universal Constructors","Imported"];
    result.sort_by_key(|p| ORDER.iter().position(|cat| *cat == p.category.as_str()).unwrap_or(ORDER.len()));
    result
}

/// Installed bundles resolve patterns relative to the executable, independent of the working directory.
pub fn pattern_directory() -> std::path::PathBuf {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let Ok(exe) = std::env::current_exe() {
        #[cfg(target_os = "macos")]
        let bundled = exe.parent().and_then(|p| p.parent()).map(|p| p.join("Resources/patterns"));
        #[cfg(target_os = "windows")]
        let bundled = exe.parent().map(|p| p.join("assets/patterns"));
        if let Some(path) = bundled.filter(|p| p.is_dir()) {
            return path;
        }
    }
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/patterns")
}

/// Missing or unsupported optional pattern files are skipped.
fn load_catalog_patterns() -> Vec<Pattern> {
    let mut loaded = Vec::new();
    let base = pattern_directory();
    for line in include_str!("../assets/patterns/catalog.tsv").lines() {
        let mut fields = line.splitn(4, '\t');
        let (Some(name),Some(category),Some(file),Some(description)) = (fields.next(),fields.next(),fields.next(),fields.next()) else { continue; };
        if !(file.ends_with(".rle") || file.ends_with(".mc")) || file.contains('/') || file.contains('\\') {continue;}
        if let Ok(cells)=parse_pattern_file(&base.join(file)) {
            loaded.push(Pattern{name:name.into(),category:category.into(),description:description.into(),cells});
        }
    }
    loaded
}

fn builtin_description(name: &str) -> &'static str {
    match name {
        "Block" => "A stable 2×2 arrangement. Every cell survives unchanged from one generation to the next.",
        "Beehive" => "A six-cell still life shaped like a hexagonal hive. It remains unchanged indefinitely.",
        "Loaf" => "A seven-cell asymmetric still life; the pattern remains stable through every generation.",
        "Boat" => "A compact five-cell still life. Its cells remain alive without creating new cells or dying.",
        "Tub" => "A symmetric four-cell diamond-shaped still life that remains unchanged.",
        "Blinker" => "A three-cell oscillator that alternates between horizontal and vertical lines every generation (period 2).",
        "Toad" => "A six-cell period-2 oscillator that alternates between two arrangements.",
        "Beacon" => "A period-2 oscillator made from two interacting blocks whose adjoining corners blink.",
        "Pulsar" => "A large period-3 oscillator with a distinctive fourfold-symmetric arrangement.",
        "Glider" => "A five-cell spaceship that moves diagonally by one cell every four generations.",
        "LWSS" => "Lightweight spaceship: a compact moving pattern that travels horizontally across the field.",
        "R-pentomino" => "A small five-cell methuselah that evolves for 1,103 generations before stabilizing.",
        "Diehard" => "A seven-cell methuselah whose population eventually disappears after 130 generations.",
        "Acorn" => "A seven-cell methuselah that evolves for 5,206 generations before stabilizing.",
        "Gosper glider gun" => "A repeating mechanism that produces a new glider every 30 generations.",
        _ => "A Conway's Game of Life pattern.",
    }
}

pub fn rotate(cells: &[(i64,i64)]) -> Vec<Cell> { normalize(cells.iter().map(|&(x,y)| (-y,x)).collect()) }
pub fn mirror(cells: &[(i64,i64)]) -> Vec<Cell> { normalize(cells.iter().map(|&(x,y)| (-x,y)).collect()) }
fn normalize(mut cells: Vec<Cell>) -> Vec<Cell> {
    if let (Some(mx),Some(my)) = (cells.iter().map(|p|p.0).min(),cells.iter().map(|p|p.1).min()) {
        for (x,y) in &mut cells { *x -= mx; *y -= my; }
    }
    cells
}

/// Decode a two-state Golly [M2] Macrocell file into sparse live cells.
/// A bounded expansion deliberately rejects huge Hashlife universes: reading
/// a compressed quadtree is not the same as running a compressed simulator.
pub fn parse_macrocell(source: &str) -> Result<Vec<Cell>, String> {
    const MAX_CELLS: usize = 2_000_000;
    const MAX_NODES: usize = 3_000_000;
    #[derive(Clone)]
    enum Node { Leaf(Vec<(i64,i64)>), Branch { level: u32, children: [usize;4] } }
    let mut lines = source.lines().map(str::trim).filter(|s| !s.is_empty());
    let header = lines.next().ok_or("empty Macrocell file")?;
    if !header.starts_with("[M2]") { return Err("expected [M2] Macrocell header".into()); }
    let mut nodes: Vec<Option<Node>> = vec![None];
    for line in lines {
        if line.starts_with('#') {
            if let Some(rule) = line.strip_prefix("#R") {
                let rule=rule.trim().to_ascii_uppercase();
                if !matches!(rule.as_str(),"B3/S23"|"LIFE"|"23/3"|"S23/B3") {
                    return Err(format!("unsupported Macrocell rule {rule}; expected B3/S23"));
                }
            }
            continue;
        }
        if nodes.len() > MAX_NODES { return Err("Macrocell node limit exceeded".into()); }
        let node = if line.starts_with('.') || line.starts_with('*') || line.starts_with('$') {
            let (mut x,mut y) = (0i64,0i64);
            let mut pixels=Vec::new();
            for ch in line.chars() {
                match ch {
                    '*' => { if x>=8 || y>=8 {return Err("Macrocell leaf exceeds 8x8".into());} pixels.push((x,y)); x+=1; }
                    '.' => { x+=1; if x>8 {return Err("Macrocell leaf exceeds 8x8".into());} }
                    '$' => { y+=1; x=0; if y>8 {return Err("Macrocell leaf exceeds 8x8".into());} }
                    _ => return Err("invalid Macrocell leaf character".into()),
                }
            }
            Node::Leaf(pixels)
        } else {
            let parts=line.split_whitespace().collect::<Vec<_>>();
            if parts.len()!=5 {return Err("expected five numbers in Macrocell branch".into());}
            let level=parts[0].parse::<u32>().map_err(|_|"invalid Macrocell level")?;
            if !(4..=61).contains(&level) {return Err("unsupported Macrocell level".into());}
            let mut children=[0usize;4];
            for (i,child) in children.iter_mut().enumerate() {
                *child=parts[i+1].parse::<usize>().map_err(|_|"invalid child index")?;
                if *child >= nodes.len() {return Err("Macrocell node refers to an undefined child".into());}
                if let Some(Node::Branch{level: child_level,..})=&nodes[*child] {
                    if *child_level != level-1 {return Err("Macrocell child level mismatch".into());}
                } else if nodes[*child].is_some() && level!=4 {
                    return Err("Macrocell leaf only valid under level-4 branch".into());
                }
            }
            Node::Branch{level,children}
        };
        nodes.push(Some(node));
    }
    if nodes.len()<2 {return Err("Macrocell has no root node".into());}
    let root=nodes.len()-1;
    let mut stack=vec![(root,0i64,0i64)];
    let mut result=Vec::new();
    while let Some((index,x,y))=stack.pop() {
        if index==0 {continue;}
        match nodes[index].as_ref().ok_or("missing Macrocell node")? {
            Node::Leaf(pixels) => {
                if result.len().saturating_add(pixels.len())>MAX_CELLS {return Err("Macrocell expands beyond 2 million live cells; needs a Hashlife backend".into());}
                for &(dx,dy) in pixels { result.push((x.checked_add(dx).ok_or("x overflow")?,y.checked_add(dy).ok_or("y overflow")?)); }
            }
            Node::Branch{level,children} => {
                let half=1i64.checked_shl(level-1).ok_or("Macrocell dimensions overflow")?;
                for (i,&child) in children.iter().enumerate() {
                    if child==0 {continue;}
                    stack.push((child,x.checked_add(if i%2==1 {half}else{0}).ok_or("x overflow")?,y.checked_add(if i>=2 {half}else{0}).ok_or("y overflow")?));
                }
            }
        }
    }
    // A Macrocell root has a virtual origin; normalize like imported RLE patterns.
    Ok(normalize(result))
}

pub fn parse_pattern_file(path: &std::path::Path) -> Result<Vec<Cell>, String> {
    let contents=std::fs::read_to_string(path).map_err(|e|format!("{}: {e}",path.display()))?;
    match path.extension().and_then(|s|s.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("rle") => parse_rle(&contents),
        Some("mc") => parse_macrocell(&contents),
        _ => Err("supported pattern extensions: .rle and .mc".into()),
    }
}

#[cfg(test)]
mod macrocell_tests {
    use super::*;

    #[test]
    fn parses_single_leaf_macrocell() {
        let cells=parse_macrocell("[M2] (test)\n#R B3/S23\n.*$..*$***$$$$$\n")
            .expect("valid [M2] leaf");
        let points: std::collections::HashSet<_>=cells.into_iter().collect();
        assert_eq!(points.len(), 5);
        assert!(points.contains(&(1,0)));
        assert!(points.contains(&(0,2)));
    }

    #[test]
    fn expands_quadtrees_and_reused_nodes() {
        let input="[M2] (test)\n*$$$$$$$$\n4 1 0 0 1\n";
        let cells=parse_macrocell(input).unwrap();
        let points: std::collections::HashSet<_>=cells.into_iter().collect();
        assert_eq!(points.len(),2);
        assert!(points.contains(&(0,0)));
        assert!(points.contains(&(8,8)));
    }

    #[test]
    fn rejects_undefined_reference() {
        assert!(parse_macrocell("[M2]\n4 2 0 0 0\n").is_err());
    }

    #[test]
    fn rejects_non_life_rule() {
        assert!(parse_macrocell("[M2]\n#R B36/S23\n*$\n").is_err());
    }

    #[test]
    fn rejects_oversized_expansion() {
        // Shared nodes can expand exponentially; the cap must be enforced.
        let mut input=String::from("[M2]\n********$********$********$********$********$********$********$********\n");
        for level in 4..=12 { input.push_str(&format!("{level} {} {} {} {}\n",level-3,level-3,level-3,level-3)); }
        assert!(parse_macrocell(&input).is_err());
    }
}

#[cfg(test)]
mod gun_catalog_tests {
    use super::*;

    #[test]
    fn gosper_glider_gun_is_bundled_and_selectable() {
        let gun = builtin().into_iter().find(|p| p.name == "Gosper glider gun")
            .expect("Gosper glider gun must be bundled");
        assert_eq!(gun.category, "Guns");
        assert_eq!(gun.cells.len(), 36);
        assert!(!gun.description.is_empty());
        // The catalog uses filter_map, so invalid RLE would silently remove the gun.
        // Assert that its declared 36x9 bounds and population agree with the source.
        assert_eq!(gun.cells.iter().map(|(x, _)| x).max(), Some(&35));
        assert_eq!(gun.cells.iter().map(|(_, y)| y).max(), Some(&8));
    }
}
