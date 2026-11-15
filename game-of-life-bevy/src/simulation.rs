use std::collections::{HashMap, HashSet};

pub type Cell = (i64, i64);
pub type Cells = HashSet<Cell>;

/// Deterministic B3/S23; only existing live cells and their neighbors are examined.
pub fn evolve(current: &Cells) -> Cells {
    let mut counts: HashMap<Cell, u8> = HashMap::with_capacity(current.len().saturating_mul(4));
    for &(x, y) in current {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                if let (Some(nx), Some(ny)) = (x.checked_add(dx), y.checked_add(dy)) {
                    let count = counts.entry((nx, ny)).or_default();
                    *count = count.saturating_add(1);
                }
            }
        }
    }
    counts.into_iter().filter_map(|(pos, count)| {
        ((count == 3) || (count == 2 && current.contains(&pos))).then_some(pos)
    }).collect()
}


/// Supported cellular automata. Brian's Brain uses an additional refractory state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LifeRule { #[default] Conway, HighLife, Seeds, DayNight, BriansBrain }
impl LifeRule {
    pub const ALL: [Self; 5] = [Self::Conway, Self::HighLife, Self::Seeds, Self::DayNight, Self::BriansBrain];
    pub fn name(self) -> &'static str { match self { Self::Conway => "Conway's Life", Self::HighLife => "HighLife", Self::Seeds => "Seeds", Self::DayNight => "Day & Night", Self::BriansBrain => "Brian's Brain" } }
    pub fn notation(self) -> &'static str { match self { Self::Conway => "B3/S23", Self::HighLife => "B36/S23", Self::Seeds => "B2/S", Self::DayNight => "B3678/S34678", Self::BriansBrain => "Three-state" } }
    pub fn description(self) -> &'static str { match self { Self::Conway => "Balanced emergence, oscillation, movement", Self::HighLife => "Supports replicator patterns", Self::Seeds => "Every living cell dies each generation; new cells arise from pairs of neighbors", Self::DayNight => "Symmetric under exchanging living and dead states", Self::BriansBrain => "Traveling patterns and persistent activity" } }
}

/// Compute the next ON and refractory populations. A refractory cell is not ON.
pub fn evolve_rule(current: &Cells, refractory: &Cells, rule: LifeRule) -> (Cells, Cells) {
    let mut counts: HashMap<Cell, u8> = HashMap::with_capacity(current.len().saturating_mul(6));
    for &(x,y) in current {
        // Include living cells, including those without live neighbors.
        counts.entry((x,y)).or_default();
        for dy in -1_i64..=1 { for dx in -1_i64..=1 {
            if dx == 0 && dy == 0 {continue;}
            if let (Some(nx),Some(ny)) = (x.checked_add(dx),y.checked_add(dy)) {
                *counts.entry((nx,ny)).or_default() += 1;
            }
        }}
    }
    let mut next=Cells::new();
    for (pos,n) in counts {
        if rule == LifeRule::BriansBrain {
            if !current.contains(&pos) && !refractory.contains(&pos) && n==2 {next.insert(pos);}
        } else {
            let alive=current.contains(&pos);
            let birth=match rule { LifeRule::Conway=>n==3, LifeRule::HighLife=>n==3||n==6, LifeRule::Seeds=>n==2, LifeRule::DayNight=>matches!(n,3|6|7|8), LifeRule::BriansBrain=>false };
            let survival=match rule { LifeRule::Conway|LifeRule::HighLife=>matches!(n,2|3), LifeRule::Seeds=>false, LifeRule::DayNight=>matches!(n,3|4|6|7|8),LifeRule::BriansBrain=>false };
            if (alive && survival)||(!alive && birth) {next.insert(pos);}
        }
    }
    let next_refractory=if rule==LifeRule::BriansBrain {current.clone()} else {Cells::new()};
    (next,next_refractory)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellKind { Birth, Living, Dying }

#[derive(Clone, Debug)]
pub struct Universe {
    pub rule: LifeRule,
    pub refractory: Cells,
    pub next_refractory: Cells,
    pub current: Cells,
    pub previous: Cells,
    pub older: Cells,
    pub forecast: Cells,
    pub generation: u64,
    pub edited_history: bool,
}
impl Default for Universe {
    fn default() -> Self { Self::new(Cells::new()) }
}
impl Universe {
    pub fn new(current: Cells) -> Self { Self::with_rule(current, LifeRule::Conway) }
    pub fn with_rule(current: Cells, rule: LifeRule) -> Self {
        let (forecast,next_refractory) = evolve_rule(&current,&Cells::new(),rule);
        Self { rule, refractory:Cells::new(), next_refractory, previous: current.clone(), older: current.clone(), current, forecast, generation: 0, edited_history: false }
    }
    pub fn advance(&mut self) {
        self.older = std::mem::take(&mut self.previous);
        self.previous = std::mem::replace(&mut self.current, std::mem::take(&mut self.forecast));
        self.refractory = std::mem::take(&mut self.next_refractory);
        self.recompute_forecast();
        self.generation = self.generation.saturating_add(1);
        self.edited_history = false;
    }
    pub fn set_rule(&mut self, rule: LifeRule) {
        self.rule=rule;
        self.refractory.clear();
        self.reset_history();
        self.recompute_forecast();
    }
    pub fn recompute_forecast(&mut self) {
        (self.forecast,self.next_refractory)=evolve_rule(&self.current,&self.refractory,self.rule);
    }
    pub fn edit(&mut self, pos: Cell, alive: bool) {
        if alive { self.current.insert(pos); self.refractory.remove(&pos); } else { self.current.remove(&pos); self.refractory.remove(&pos); }
        self.recompute_forecast();
        self.edited_history = true;
    }
    /// Apply a brush stroke and recompute the forecast once per stroke.
    pub fn paint_cells(&mut self, updates: impl IntoIterator<Item = (Cell, bool)>) {
        let mut changed = false;
        for (pos, alive) in updates {
            changed |= if alive { self.current.insert(pos) } else { self.current.remove(&pos) };
            changed |= self.refractory.remove(&pos);
        }
        if changed {
            self.recompute_forecast();
            self.edited_history = true;
        }
    }
    /// Remove live cells inside an inclusive rectangle, without iterating empty coordinates.
    pub fn erase_rectangle(&mut self, a: Cell, b: Cell) -> usize {
        let (min_x, max_x) = (a.0.min(b.0), a.0.max(b.0));
        let (min_y, max_y) = (a.1.min(b.1), a.1.max(b.1));
        let before = self.current.len();
        self.current.retain(|&(x, y)| x < min_x || x > max_x || y < min_y || y > max_y);
        self.refractory.retain(|&(x, y)| x < min_x || x > max_x || y < min_y || y > max_y);
        let removed = before - self.current.len();
        if removed > 0 {
            self.recompute_forecast();
            self.edited_history = true;
        }
        removed
    }
    pub fn reset_history(&mut self) {
        self.previous = self.current.clone();
        self.older = self.current.clone();
        self.edited_history = false;
    }
    pub fn forecast_kind(&self, pos: &Cell) -> Option<CellKind> {
        match (self.current.contains(pos), self.forecast.contains(pos)) {
            (false, true) => Some(CellKind::Birth),
            (true, true) => Some(CellKind::Living),
            (true, false) => Some(CellKind::Dying),
            (false, false) => None,
        }
    }
    pub fn history_kind(&self, pos: &Cell) -> Option<CellKind> {
        match (self.previous.contains(pos), self.current.contains(pos)) {
            (false, true) => Some(CellKind::Birth),
            (true, true) => Some(CellKind::Living),
            (true, false) => Some(CellKind::Dying),
            (false, false) => None,
        }
    }
    pub fn transition_counts(&self) -> (usize, usize) {
        (self.forecast.difference(&self.current).count(), self.current.difference(&self.forecast).count())
    }
}

/// Predicted change from the current population to the next generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NetChange {
    pub delta: i128,
    pub rate_percent: Option<f64>,
}

pub fn net_change(births: usize, deaths: usize, current_population: usize) -> NetChange {
    let delta = births as i128 - deaths as i128;
    let rate_percent = (current_population > 0)
        .then(|| delta as f64 / current_population as f64 * 100.0);
    NetChange { delta, rate_percent }
}

#[cfg(test)]
mod net_change_tests {
    use super::*;

    #[test]
    fn positive_and_negative_changes() {
        assert_eq!(net_change(25, 5, 100).delta, 20);
        assert_eq!(net_change(25, 5, 100).rate_percent, Some(20.0));
        assert_eq!(net_change(5, 25, 100).rate_percent, Some(-20.0));
        assert_eq!(net_change(10, 10, 100).rate_percent, Some(0.0));
    }

    #[test]
    fn zero_population_has_no_defined_percentage() {
        assert_eq!(net_change(0, 0, 0).rate_percent, None);
    }
}

/// Inclusive population bounds, or None for an empty universe.
pub fn bounding_box(cells: &Cells) -> Option<(Cell, Cell)> {
    let mut iter = cells.iter().copied();
    let (mut min_x, mut min_y) = iter.next()?;
    let (mut max_x, mut max_y) = (min_x, min_y);
    for (x, y) in iter {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    Some(((min_x, min_y), (max_x, max_y)))
}

#[cfg(test)]
mod rectangle_tests {
    use super::*;
    #[test]
    fn erase_rectangle_is_inclusive_and_direction_independent() {
        let mut universe = Universe::new(Cells::from_iter([(0,0),(1,1),(2,2),(3,3),(-1,-1)]));
        assert_eq!(universe.erase_rectangle((2,2),(0,0)),3);
        assert_eq!(universe.current, Cells::from_iter([(3,3),(-1,-1)]));
        assert!(universe.edited_history);
    }
    #[test]
    fn empty_rectangle_has_no_effect() {
        let mut universe = Universe::new(Cells::from_iter([(0,0)]));
        assert_eq!(universe.erase_rectangle((10,10),(20,20)),0);
        assert!(!universe.edited_history);
    }
}

#[cfg(test)]
mod alternative_rules_tests {
    use super::*;
    #[test] fn conway_blinker() {
        let a=Cells::from_iter([(0,0),(1,0),(2,0)]);
        let (b,_)=evolve_rule(&a,&Cells::new(),LifeRule::Conway);
        assert_eq!(b,Cells::from_iter([(1,-1),(1,0),(1,1)]));
    }
    #[test] fn highlife_birth_at_six() {
        let a=Cells::from_iter([(-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1)]);
        assert!(evolve_rule(&a,&Cells::new(),LifeRule::HighLife).0.contains(&(0,0)));
        assert!(!evolve_rule(&a,&Cells::new(),LifeRule::Conway).0.contains(&(0,0)));
    }
    #[test] fn seeds_has_no_survivors() {
        let a=Cells::from_iter([(0,0),(1,0)]);
        let (next,_)=evolve_rule(&a,&Cells::new(),LifeRule::Seeds);
        assert!(!next.contains(&(0,0)) && !next.contains(&(1,0)));
        assert!(next.contains(&(0,1)));
    }
    #[test] fn day_night_survival_at_four() {
        let a=Cells::from_iter([(0,0),(-1,0),(1,0),(0,-1),(0,1)]);
        assert!(evolve_rule(&a,&Cells::new(),LifeRule::DayNight).0.contains(&(0,0)));
    }
    #[test] fn brians_brain_three_states() {
        let mut u=Universe::with_rule(Cells::from_iter([(0,0),(1,0)]),LifeRule::BriansBrain);
        u.advance();
        assert!(u.refractory.contains(&(0,0)));
        assert!(u.current.contains(&(0,1)));
        u.advance();
        assert!(!u.refractory.contains(&(0,0)));
        assert!(!u.current.contains(&(0,0)));
    }
    #[test] fn switching_rule_clears_refractory() {
        let mut u=Universe::with_rule(Cells::from_iter([(0,0)]),LifeRule::BriansBrain);
        u.advance(); u.set_rule(LifeRule::Conway);
        assert!(u.refractory.is_empty()); assert_eq!(u.rule,LifeRule::Conway);
    }
}
