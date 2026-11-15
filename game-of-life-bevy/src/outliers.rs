//! Density-based outlier detection for sparse Life populations.
use crate::simulation::{Cell, Cells};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlierOptions {
    pub radius: i64,
    pub min_neighbors: usize,
    pub min_mass_percent: f32,
    pub padding: i64,
    pub multiple_masses: bool,
    pub include_births: bool,
}
impl Default for OutlierOptions {
    fn default() -> Self { Self { radius: 15, min_neighbors: 5, min_mass_percent: 5.0, padding: 8, multiple_masses: true, include_births: true } }
}

#[derive(Clone, Debug)]
pub struct Region { pub bounds: (Cell, Cell), pub population: usize }
#[derive(Clone, Debug)]
pub struct OutlierAnalysis { pub remove: Cells, pub regions: Vec<Region>, pub population: usize }

fn bucket(p: Cell, side: i64) -> Cell { (p.0.div_euclid(side), p.1.div_euclid(side)) }
fn neighbors(p: Cell, points: &[Cell], bins: &HashMap<Cell, Vec<usize>>, radius: i64) -> Vec<usize> {
    let r2 = (radius as i128) * (radius as i128);
    let (bx, by) = bucket(p, radius);
    let mut found = Vec::new();
    for dy in -1..=1_i64 { for dx in -1..=1_i64 {
        let Some(x) = bx.checked_add(dx) else { continue };
        let Some(y) = by.checked_add(dy) else { continue };
        if let Some(entries) = bins.get(&(x,y)) { for &idx in entries {
            let q = points[idx];
            let ax = p.0 as i128 - q.0 as i128;
            let ay = p.1 as i128 - q.1 as i128;
            if ax*ax + ay*ay <= r2 { found.push(idx); }
        }}
    }}
    found
}

/// Identify low-density cells outside substantial density-connected regions.
/// Detection may include predicted births; removals always affect only current live cells.
pub fn analyze(current: &Cells, forecast: &Cells, options: OutlierOptions) -> OutlierAnalysis {
    let mut result = OutlierAnalysis { remove: Cells::new(), regions: Vec::new(), population: current.len() };
    if current.is_empty() { return result; }
    let radius = options.radius.clamp(1, 256);
    let padding = options.padding.clamp(0, 1024);
    let points: Vec<Cell> = if options.include_births {
        current.union(forecast).copied().collect()
    } else { current.iter().copied().collect() };
    let mut bins: HashMap<Cell, Vec<usize>> = HashMap::new();
    for (idx, &point) in points.iter().enumerate() { bins.entry(bucket(point, radius)).or_default().push(idx); }
    let mut seen = vec![false; points.len()];
    let mut core = vec![false; points.len()];
    let mut adjacency = Vec::with_capacity(points.len());
    for (idx, &point) in points.iter().enumerate() {
        let near = neighbors(point, &points, &bins, radius);
        core[idx] = near.len() >= options.min_neighbors.max(1);
        adjacency.push(near);
    }
    let mut masses: Vec<(Region, Cells)> = Vec::new();
    for start in 0..points.len() {
        if seen[start] || !core[start] { continue; }
        let mut queue = VecDeque::from([start]);
        seen[start] = true;
        let mut members = HashSet::new();
        while let Some(idx) = queue.pop_front() {
            members.insert(points[idx]);
            if core[idx] {
                for &j in &adjacency[idx] {
                    if !seen[j] { seen[j] = true; if core[j] { queue.push_back(j); } }
                    members.insert(points[j]);
                }
            }
        }
        let live: Cells = members.intersection(current).copied().collect();
        if live.is_empty() { continue; }
        let min_x = members.iter().map(|p| p.0).min().unwrap();
        let min_y = members.iter().map(|p| p.1).min().unwrap();
        let max_x = members.iter().map(|p| p.0).max().unwrap();
        let max_y = members.iter().map(|p| p.1).max().unwrap();
        masses.push((Region { bounds: ((min_x.saturating_sub(padding), min_y.saturating_sub(padding)),(max_x.saturating_add(padding),max_y.saturating_add(padding))), population: live.len() }, live));
    }
    masses.sort_by(|a,b| b.0.population.cmp(&a.0.population));
    let minimum = ((current.len() as f64 * f64::from(options.min_mass_percent.clamp(0.,100.)) / 100.).ceil() as usize).max(1);
    // No credible mass: fail safe rather than erase the entire universe.
    if !masses.iter().any(|m|m.0.population >= minimum) { return result; }
    for (region, _) in masses {
        if region.population < minimum { continue; }
        result.regions.push(region);
        if !options.multiple_masses { break; }
    }
    result.remove = current.iter().filter(|&&(x,y)| !result.regions.iter().any(|r| {
        let (lo,hi)=r.bounds;
        x>=lo.0 && x<=hi.0 && y>=lo.1 && y<=hi.1
    })).copied().collect();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn square(x:i64,y:i64,n:i64)->Cells { (0..n).flat_map(|dy| (0..n).map(move |dx| (x+dx,y+dy))).collect() }
    #[test] fn defaults() { let o=OutlierOptions::default(); assert_eq!(o.radius,15); assert_eq!(o.min_neighbors,5); assert_eq!(o.min_mass_percent,5.); assert_eq!(o.padding,8); assert!(o.multiple_masses && o.include_births); }
    #[test] fn removes_distant_outliers() { let mut c=square(0,0,6); c.insert((200,200));let r=analyze(&c,&c,OutlierOptions::default());assert_eq!(r.regions.len(),1);assert_eq!(r.remove,Cells::from([(200,200)])); }
    #[test] fn retains_multiple_masses() {let mut c=square(0,0,6); c.extend(square(200,0,6));c.insert((500,500));let r=analyze(&c,&c,OutlierOptions::default());assert_eq!(r.regions.len(),2);assert_eq!(r.remove.len(),1);}
    #[test] fn single_mass_mode() {let mut c=square(0,0,7);c.extend(square(200,0,6));let mut o=OutlierOptions::default();o.multiple_masses=false;let r=analyze(&c,&c,o);assert_eq!(r.regions.len(),1);assert_eq!(r.remove.len(),36);}
    #[test] fn no_major_mass_fails_safe() {let c=Cells::from([(0,0),(1000,1000)]);let r=analyze(&c,&c,OutlierOptions::default());assert!(r.remove.is_empty());}
    #[test] fn births_only_inform_detection() {let c=square(0,0,6);let mut f=c.clone();f.insert((300,300));let r=analyze(&c,&f,OutlierOptions::default());assert!(r.remove.is_empty());}
    #[test] fn padding_retains_nearby_island() {let mut c=square(0,0,6);c.insert((13,3));let r=analyze(&c,&c,OutlierOptions::default());assert!(!r.remove.contains(&(13,3)));}
    #[test] fn empty_safe() {let r=analyze(&Cells::new(),&Cells::new(),OutlierOptions::default()); assert!(r.remove.is_empty());}
}

/// Bounding rectangles around separately isolated removal groups. Visualization only.
pub fn removal_group_bounds(remove: &Cells, radius: i64) -> Vec<(Cell, Cell)> {
    use std::collections::VecDeque;
    if remove.is_empty() {return Vec::new();}
    let radius=radius.max(1);
    let mut buckets: std::collections::HashMap<(i64,i64),Vec<Cell>>=std::collections::HashMap::new();
    for &p in remove {buckets.entry((p.0.div_euclid(radius),p.1.div_euclid(radius))).or_default().push(p);}
    let mut visited=Cells::new();
    let mut result=Vec::new();
    for &seed in remove {
        if !visited.insert(seed){continue;}
        let mut queue=VecDeque::from([seed]);
        let (mut min_x,mut min_y,mut max_x,mut max_y)=(seed.0,seed.1,seed.0,seed.1);
        while let Some(p)=queue.pop_front(){
            min_x=min_x.min(p.0);min_y=min_y.min(p.1);max_x=max_x.max(p.0);max_y=max_y.max(p.1);
            let bx=p.0.div_euclid(radius);let by=p.1.div_euclid(radius);
            for dx in -1..=1 {for dy in -1..=1 {
                if let Some(v)=buckets.get(&(bx+dx,by+dy)) {
                    for &q in v {if (p.0-q.0).abs()<=radius && (p.1-q.1).abs()<=radius && visited.insert(q) {queue.push_back(q);}}
                }
            }}
        }
        result.push(((min_x,min_y),(max_x,max_y)));
    }
    result
}

#[cfg(test)]
mod removal_bounds_tests {
    use super::*;
    #[test]
    fn separate_removal_masses_produce_distinct_rectangles() {
        let cells: Cells = [(0,0),(1,1),(100,100),(101,100)].into_iter().collect();
        let mut bounds = removal_group_bounds(&cells, 3);
        bounds.sort();
        assert_eq!(bounds, vec![((0,0),(1,1)),((100,100),(101,100))]);
    }
    #[test]
    fn empty_removal_has_no_regions() {
        assert!(removal_group_bounds(&Cells::new(), 15).is_empty());
    }
}
