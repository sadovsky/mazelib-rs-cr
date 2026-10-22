//! Perturbation: add a few random walls, then reconnect the maze.

use super::common::{cells, check_entrance, is_cell, offset};
use super::Transmuter;
use crate::grid::{OPEN, WALL};
use crate::{Cell, Grid, Result, Rng};

/// Runs `repeat` rounds; each closes `new_walls` random open interior slots
/// and then reopens random slots, each joining two different components of
/// open cells, until the open cells are connected again.
///
/// Only interior wall slots change. A perfect input stays perfect for any
/// parameters. Draws from `rng.py`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Perturbation {
    repeat: i64,
    new_walls: i64,
}

impl Default for Perturbation {
    fn default() -> Self {
        Perturbation::new(1, 1)
    }
}

/// An interior wall slot and the two cells it separates.
#[derive(Debug, Clone, Copy)]
struct Slot {
    at: Cell,
    a: Cell,
    b: Cell,
}

/// Every slot between two orthogonally adjacent interior cells, row-major.
fn interior_slots(g: &Grid) -> Vec<Slot> {
    let mut out = Vec::new();
    for a in cells(g) {
        for d in [(1, 0), (0, 1)] {
            if let (Some(at), Some(b)) = (offset(g, a, d, 1), offset(g, a, d, 2)) {
                if is_cell(g, b) {
                    out.push(Slot { at, a, b });
                }
            }
        }
    }
    out.sort_by_key(|s| s.at);
    out
}

/// Union-find over grid squares.
struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Dsu {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    /// Merge; `false` when already joined.
    fn union(&mut self, a: usize, b: usize) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        self.parent[ra] = rb;
        true
    }
}

impl Perturbation {
    /// `repeat` rounds of `new_walls` added walls. `repeat <= 0` leaves the
    /// grid unchanged; `new_walls <= 0` adds no walls but still reconnects.
    pub fn new(repeat: i64, new_walls: i64) -> Self {
        Perturbation { repeat, new_walls }
    }

    /// Number of rounds.
    pub fn repeat(&self) -> i64 {
        self.repeat
    }

    /// Walls added per round.
    pub fn new_walls(&self) -> i64 {
        self.new_walls
    }

    fn add_walls(&self, grid: &mut Grid, slots: &[Slot], protected: &[Cell], rng: &mut Rng) {
        for _ in 0..self.new_walls.max(0) {
            let open: Vec<Cell> = slots
                .iter()
                .map(|s| s.at)
                .filter(|&(r, c)| grid.is_open(r, c) && !protected.contains(&(r, c)))
                .collect();
            let Some(&(r, c)) = rng.py.choose(&open) else {
                break;
            };
            grid.set(r, c, WALL);
        }
    }

    fn reconnect(grid: &mut Grid, slots: &[Slot], protected: &[Cell], rng: &mut Rng) {
        let w = grid.cols();
        let idx = |(r, c): Cell| r * w + c;
        let mut dsu = Dsu::new(grid.rows() * w);
        let open_cells: Vec<Cell> = cells(grid).filter(|&(r, c)| grid.is_open(r, c)).collect();
        let mut comps = open_cells.len();
        // Join cells already connected through open slots.
        let mut closed: Vec<Slot> = Vec::new();
        for s in slots {
            let both_open = grid.is_open(s.a.0, s.a.1) && grid.is_open(s.b.0, s.b.1);
            if !both_open {
                // Wall cells are never join points.
                continue;
            }
            if grid.is_open(s.at.0, s.at.1) {
                if dsu.union(idx(s.a), idx(s.b)) {
                    comps -= 1;
                }
            } else if !protected.contains(&s.at) {
                closed.push(*s);
            }
        }
        // Kruskal-style: random order, open a slot only if it joins two
        // different components.
        rng.py.shuffle(&mut closed);
        for s in closed {
            if comps <= 1 {
                break;
            }
            if dsu.union(idx(s.a), idx(s.b)) {
                grid.set(s.at.0, s.at.1, OPEN);
                comps -= 1;
            }
        }
    }
}

impl Transmuter for Perturbation {
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        end: Option<Cell>,
        rng: &mut Rng,
    ) -> Result<()> {
        check_entrance(grid, start, "start")?;
        check_entrance(grid, end, "end")?;
        // Entrances are border squares or cells, never interior slots; this
        // only guards unusual inputs.
        let protected: Vec<Cell> = start.into_iter().chain(end).collect();
        let slots = interior_slots(grid);
        for _ in 0..self.repeat.max(0) {
            self.add_walls(grid, &slots, &protected, rng);
            Self::reconnect(grid, &slots, &protected, rng);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transmute::tests::{check_case, perfect, G1};

    fn diff(a: &Grid, b: &Grid) -> (usize, usize) {
        let mut closed = 0;
        let mut opened = 0;
        for (x, y) in a.as_slice().iter().zip(b.as_slice()) {
            match (*x, *y) {
                (OPEN, WALL) => closed += 1,
                (WALL, OPEN) => opened += 1,
                _ => {}
            }
        }
        (closed, opened)
    }

    #[test]
    fn defaults_swap_one_slot() {
        for seed in 0..20 {
            let before = Grid::from_rows(&G1);
            let mut g = before.clone();
            let mut rng = Rng::seeded(seed).unwrap();
            Perturbation::default()
                .transmute(&mut g, Some((3, 0)), Some((1, 6)), &mut rng)
                .unwrap();
            assert!(perfect(&g));
            let (c, o) = diff(&before, &g);
            assert_eq!(c, o);
            assert!(c <= 1);
        }
    }

    #[test]
    fn deterministic_per_seed() {
        let run = |seed| {
            let mut g = Grid::from_rows(&G1);
            let mut rng = Rng::seeded(seed).unwrap();
            Perturbation::new(3, 2)
                .transmute(&mut g, None, None, &mut rng)
                .unwrap();
            g
        };
        assert_eq!(run(5), run(5));
    }

    #[test]
    fn repeat_zero_is_noop_and_new_walls_zero_reconnects() {
        let before = Grid::from_rows(&G1);
        let mut g = before.clone();
        let mut rng = Rng::seeded(0).unwrap();
        Perturbation::new(0, 5)
            .transmute(&mut g, None, None, &mut rng)
            .unwrap();
        assert_eq!(g, before);

        // Two disconnected corridors get joined.
        let split = ["#######", "#.....#", "#######", "#.....#", "#######"];
        let mut g = Grid::from_rows(&split);
        Perturbation::new(1, 0)
            .transmute(&mut g, None, None, &mut rng)
            .unwrap();
        assert!(perfect(&g));
    }

    #[test]
    fn wall_cells_terminate_and_stay() {
        // The two halves are separated by a wall cell column; they can't join.
        let g0 = ["#######", "#.###.#", "#######"];
        let mut g = Grid::from_rows(&g0);
        let mut rng = Rng::seeded(0).unwrap();
        Perturbation::new(2, 3)
            .transmute(&mut g, None, None, &mut rng)
            .unwrap();
        assert_eq!(g, Grid::from_rows(&g0));
    }

    #[test]
    fn large_new_walls_stays_connected() {
        let mut g = Grid::from_rows(&G1);
        let mut rng = Rng::seeded(1).unwrap();
        Perturbation::new(2, 1000)
            .transmute(&mut g, None, None, &mut rng)
            .unwrap();
        assert!(perfect(&g));
    }

    #[test]
    fn golden_cases() {
        check_case("Perturbation", |p| {
            Box::new(Perturbation::new(
                p.get("repeat").copied().unwrap_or(1),
                p.get("new_walls").copied().unwrap_or(1),
            ))
        });
    }
}
