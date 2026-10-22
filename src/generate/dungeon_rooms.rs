//! Dungeon rooms: carve rectangular rooms, fill the rest with Hunt-and-Kill
//! walks, then join any regions left disconnected.

use super::cellgrid::{carve, cell_neighbors, open_cell, pos};
use super::kruskal::Dsu;
use super::{Generator, HuntOrder};
use crate::grid::{OPEN, WALL};
use crate::{Grid, MazeError, Result, Rng};

/// A room: `((top, left), (bottom, right))` in grid coordinates, inclusive.
pub type Room = ((i64, i64), (i64, i64));

/// Hunt-and-Kill with pre-carved rooms.
#[derive(Debug, Clone)]
pub struct DungeonRooms {
    h: usize,
    w: usize,
    rooms: Vec<Room>,
    base: Option<Grid>,
    hunt_order: HuntOrder,
}

impl DungeonRooms {
    /// Validate the size. A supplied `grid` overrides `h`/`w` (its size is
    /// `(rows-1)/2 x (cols-1)/2` cells). Without one, negative dimensions are
    /// an invalid value and sizes below 3x3 an invalid input.
    pub fn new(
        h: i64,
        w: i64,
        rooms: Option<Vec<Room>>,
        grid: Option<Grid>,
        hunt_order: HuntOrder,
    ) -> Result<Self> {
        let (h, w) = match &grid {
            Some(g) => {
                let gh = (g.rows() as i64 - 1).div_euclid(2);
                let gw = (g.cols() as i64 - 1).div_euclid(2);
                if gh < 3 || gw < 3 {
                    return Err(MazeError::input("Mazes cannot be smaller than 3x3."));
                }
                (gh as usize, gw as usize)
            }
            None => {
                if h < 0 || w < 0 {
                    return Err(MazeError::value("negative dimensions are not allowed"));
                }
                super::check_size(h, w)?
            }
        };
        Ok(DungeonRooms {
            h,
            w,
            rooms: rooms.unwrap_or_default(),
            base: grid,
            hunt_order,
        })
    }

    /// Open one room (clipped to the interior) and maybe a door.
    fn carve_room(&self, grid: &mut Grid, room: Room, rng: &mut Rng) {
        let (h, w) = (self.h as i64, self.w as i64);
        let ((r1, c1), (r2, c2)) = room;
        // SLOPFORK-Q: out-of-range rooms are clipped to the interior rather
        // than ignored; the frame is never opened.
        let (r1, c1) = (r1.max(1), c1.max(1));
        let (r2, c2) = (r2.min(2 * h - 1), c2.min(2 * w - 1));
        if r2 < r1 || c2 < c1 {
            return;
        }
        for r in r1..=r2 {
            for c in c1..=c2 {
                grid.set(r as usize, c as usize, OPEN);
            }
        }
        let odd = |v: i64| v % 2 == 1;
        if !(odd(r1) && odd(c1) && odd(r2) && odd(c2)) {
            return;
        }
        let mut doors: Vec<(i64, i64)> = Vec::new();
        if r1 >= 3 {
            doors.extend((c1..=c2).step_by(2).map(|c| (r1 - 1, c)));
        }
        if r2 < 2 * h - 1 {
            doors.extend((c1..=c2).step_by(2).map(|c| (r2 + 1, c)));
        }
        if c1 >= 3 {
            doors.extend((r1..=r2).step_by(2).map(|r| (r, c1 - 1)));
        }
        if c2 < 2 * w - 1 {
            doors.extend((r1..=r2).step_by(2).map(|r| (r, c2 + 1)));
        }
        if let Some(&(r, c)) = rng.py.choose(&doors) {
            grid.set(r as usize, c as usize, OPEN);
        }
    }

    fn is_open_cell(&self, grid: &Grid, (i, j): (usize, usize)) -> bool {
        let (r, c) = pos(i, j);
        grid.is_open(r, c)
    }

    /// Wall neighbour cells of an open cell.
    fn wall_neighbors(&self, grid: &Grid, cell: (usize, usize)) -> Vec<(usize, usize)> {
        cell_neighbors(cell.0, cell.1, self.h, self.w)
            .into_iter()
            .filter(|&n| !self.is_open_cell(grid, n))
            .collect()
    }

    /// The "kill" phase: random walk into wall cells until stuck.
    fn walk(&self, grid: &mut Grid, mut cur: (usize, usize), rng: &mut Rng) {
        loop {
            let nbrs = self.wall_neighbors(grid, cur);
            let Some(&next) = rng.py.choose(&nbrs) else {
                return;
            };
            carve(grid, cur, next);
            cur = next;
        }
    }

    /// Join disconnected cell regions by opening cell-to-cell slots.
    fn join_regions(&self, grid: &mut Grid, rng: &mut Rng) {
        let (h, w) = (self.h, self.w);
        let mut dsu = Dsu::new(h * w);
        let mut candidates = Vec::new();
        for i in 0..h {
            for j in 0..w {
                for (ni, nj) in [(i + 1, j), (i, j + 1)] {
                    if ni >= h || nj >= w {
                        continue;
                    }
                    let (sr, sc) = (i + ni + 1, j + nj + 1);
                    if grid.is_open(sr, sc) {
                        dsu.union(i * w + j, ni * w + nj);
                    } else {
                        candidates.push(((i, j), (ni, nj)));
                    }
                }
            }
        }
        rng.py.shuffle(&mut candidates);
        for (a, b) in candidates {
            if dsu.union(a.0 * w + a.1, b.0 * w + b.1) {
                grid.set(a.0 + b.0 + 1, a.1 + b.1 + 1, OPEN);
            }
        }
    }
}

impl Generator for DungeonRooms {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut grid = match &self.base {
            Some(g) => g.clone(),
            None => Grid::walls(2 * h + 1, 2 * w + 1),
        };
        for room in self.rooms.clone() {
            self.carve_room(&mut grid, room, rng);
        }

        let walls: Vec<(usize, usize)> = (0..h)
            .flat_map(|i| (0..w).map(move |j| (i, j)))
            .filter(|&c| !self.is_open_cell(&grid, c))
            .collect();
        let Some(&start) = rng.py.choose(&walls) else {
            return Err(MazeError::input("no unvisited cell to start from"));
        };
        open_cell(&mut grid, start);
        self.walk(&mut grid, start, rng);

        let mut cursor = 0usize;
        loop {
            let next = match self.hunt_order {
                HuntOrder::Serpentine => {
                    // Cells only lose wall neighbours, so the scan never rewinds.
                    let mut found = None;
                    while cursor < h * w {
                        let c = (cursor / w, cursor % w);
                        if self.is_open_cell(&grid, c) && !self.wall_neighbors(&grid, c).is_empty()
                        {
                            found = Some(c);
                            break;
                        }
                        cursor += 1;
                    }
                    found
                }
                HuntOrder::Random => {
                    // Uniform among open cells that can still grow: the same
                    // distribution as random picks that skip useless cells,
                    // but certain to terminate.
                    let cands: Vec<(usize, usize)> = (0..h)
                        .flat_map(|i| (0..w).map(move |j| (i, j)))
                        .filter(|&c| {
                            self.is_open_cell(&grid, c) && !self.wall_neighbors(&grid, c).is_empty()
                        })
                        .collect();
                    rng.py.choose(&cands).copied()
                }
            };
            match next {
                Some(c) => self.walk(&mut grid, c, rng),
                None => break,
            }
        }

        // Every cell is open now (walks reach any wall cell next to an open
        // one), but guard anyway so cells_open holds unconditionally.
        for i in 0..h {
            for j in 0..w {
                let (r, c) = pos(i, j);
                if grid.get(r, c) == WALL {
                    grid.set(r, c, OPEN);
                }
            }
        }
        self.join_regions(&mut grid, rng);
        Ok(grid)
    }

    fn size(&self) -> (usize, usize) {
        (self.h, self.w)
    }
}

#[cfg(test)]
mod tests {
    use super::super::cellgrid::check::{INVALID, SIZES};
    use super::super::cellular_automaton::tests::assert_loose;
    use super::*;

    fn golden_room(h: i64, w: i64) -> Room {
        match (h, w) {
            (4, 7) => ((3, 5), (5, 7)),
            (8, 8) => ((5, 5), (7, 7)),
            (12, 17) => ((9, 11), (11, 13)),
            _ => ((3, 3), (5, 5)),
        }
    }

    #[test]
    fn no_rooms_both_orders() {
        for order in [HuntOrder::Random, HuntOrder::Serpentine] {
            for (h, w) in SIZES.iter().copied().chain([(5, 5), (6, 6)]) {
                for seed in 0..8 {
                    let mut g = DungeonRooms::new(h, w, None, None, order).unwrap();
                    let grid = g.generate(&mut Rng::seeded(seed).unwrap()).unwrap();
                    assert_loose(&grid, h as usize, w as usize, true);
                }
            }
        }
    }

    #[test]
    fn golden_rooms() {
        let mut cases: Vec<(i64, i64, Room)> = SIZES
            .iter()
            .map(|&(h, w)| (h, w, golden_room(h, w)))
            .collect();
        cases.push((10, 10, ((3, 3), (9, 9))));
        for (h, w, room) in cases {
            for order in [HuntOrder::Random, HuntOrder::Serpentine] {
                for seed in 0..8 {
                    let mut g = DungeonRooms::new(h, w, Some(vec![room]), None, order).unwrap();
                    let grid = g.generate(&mut Rng::seeded(seed).unwrap()).unwrap();
                    assert_loose(&grid, h as usize, w as usize, false);
                    let ((r1, c1), (r2, c2)) = room;
                    for r in r1..=r2 {
                        for c in c1..=c2 {
                            assert!(grid.is_open(r as usize, c as usize));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn odd_rooms_even_rooms_and_clipping() {
        let rooms = vec![
            ((2, 2), (4, 4)),
            ((7, 1), (9, 3)),
            ((-5, 9), (3, 99)),
            ((5, 5), (3, 3)),
        ];
        for seed in 0..8 {
            let mut g =
                DungeonRooms::new(6, 7, Some(rooms.clone()), None, HuntOrder::Random).unwrap();
            let grid = g.generate(&mut Rng::seeded(seed).unwrap()).unwrap();
            assert_loose(&grid, 6, 7, false);
        }
    }

    #[test]
    fn supplied_grid() {
        let mut base = Grid::walls(9, 9);
        for r in 5..=7 {
            for c in 5..=7 {
                base.set(r, c, OPEN);
            }
        }
        let mut g = DungeonRooms::new(0, 0, None, Some(base.clone()), HuntOrder::Random).unwrap();
        assert_eq!(g.size(), (4, 4));
        for seed in 0..4 {
            let grid = g.generate(&mut Rng::seeded(seed).unwrap()).unwrap();
            assert_loose(&grid, 4, 4, false);
            assert!((5..=7).all(|r| (5..=7).all(|c| grid.is_open(r, c))));
        }
        assert!(matches!(
            DungeonRooms::new(9, 9, None, Some(Grid::walls(5, 9)), HuntOrder::Random),
            Err(MazeError::InvalidInput(_))
        ));
    }

    #[test]
    fn errors() {
        for (h, w) in INVALID {
            let e = DungeonRooms::new(h, w, None, None, HuntOrder::Random).unwrap_err();
            if h < 0 || w < 0 {
                assert!(matches!(e, MazeError::InvalidValue(_)));
            } else {
                assert!(matches!(e, MazeError::InvalidInput(_)));
            }
        }
        let mut g =
            DungeonRooms::new(3, 3, Some(vec![((1, 1), (5, 5))]), None, HuntOrder::Random).unwrap();
        assert!(matches!(
            g.generate(&mut Rng::seeded(0).unwrap()),
            Err(MazeError::InvalidInput(_))
        ));
    }
}
