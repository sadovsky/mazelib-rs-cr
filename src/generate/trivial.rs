//! Trivial unicursal mazes: a serpentine (boustrophedon) or an inward spiral.

use super::cellgrid::{blank, carve, open_cell};
use super::{check_size, Generator};
use crate::grid::OPEN;
use crate::{Grid, Result, Rng};

/// The shape of a [`TrivialMaze`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MazeType {
    /// An inward rectangular spiral starting at the top-left cell (the default).
    #[default]
    Spiral,
    /// A boustrophedon covering every row or every column.
    Serpentine,
}

impl MazeType {
    /// Parse the Python option string: trimmed, case-insensitive; anything
    /// other than `serpentine` means [`MazeType::Spiral`].
    pub fn parse(s: &str) -> Self {
        if s.trim().eq_ignore_ascii_case("serpentine") {
            MazeType::Serpentine
        } else {
            MazeType::Spiral
        }
    }
}

/// A single-path maze. One fair coin per `generate` picks the orientation.
#[derive(Debug, Clone)]
pub struct TrivialMaze {
    h: usize,
    w: usize,
    maze_type: MazeType,
}

impl TrivialMaze {
    /// Validate the size (at least 3x3 cells).
    pub fn new(h: i64, w: i64, maze_type: MazeType) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(TrivialMaze { h, w, maze_type })
    }

    /// Serpentine: vertical corridors joined alternately top/bottom, or the
    /// transposed horizontal layout.
    fn serpentine(&self, vertical: bool) -> Grid {
        let (h, w) = (self.h, self.w);
        let mut grid = blank(h, w);
        if vertical {
            for c in (1..2 * w).step_by(2) {
                for r in 1..2 * h {
                    grid.set(r, c, OPEN);
                }
            }
            for c in (2..2 * w).step_by(4) {
                grid.set(1, c, OPEN);
            }
            for c in (4..2 * w).step_by(4) {
                grid.set(2 * h - 1, c, OPEN);
            }
        } else {
            for r in (1..2 * h).step_by(2) {
                for c in 1..2 * w {
                    grid.set(r, c, OPEN);
                }
            }
            for r in (2..2 * h).step_by(4) {
                grid.set(r, 1, OPEN);
            }
            for r in (4..2 * h).step_by(4) {
                grid.set(r, 2 * w - 1, OPEN);
            }
        }
        grid
    }

    /// Spiral from the top-left cell, turning through `dirs` when blocked.
    fn spiral(&self, down_first: bool) -> Grid {
        let (h, w) = (self.h, self.w);
        // (drow, dcol) in cell units.
        let dirs: [(isize, isize); 4] = if down_first {
            [(1, 0), (0, 1), (-1, 0), (0, -1)]
        } else {
            [(0, 1), (1, 0), (0, -1), (-1, 0)]
        };
        let mut grid = blank(h, w);
        let mut visited = vec![false; h * w];
        let mut cur = (0usize, 0usize);
        visited[0] = true;
        open_cell(&mut grid, cur);
        let step = |c: (usize, usize), d: (isize, isize)| -> Option<(usize, usize)> {
            let r = c.0.checked_add_signed(d.0)?;
            let k = c.1.checked_add_signed(d.1)?;
            (r < h && k < w).then_some((r, k))
        };
        let mut d = 0usize;
        loop {
            // Try the current direction, then turn through the cycle.
            let mut moved = false;
            for t in 0..4 {
                let dir = dirs[(d + t) % 4];
                if let Some(n) = step(cur, dir) {
                    if !visited[n.0 * w + n.1] {
                        d = (d + t) % 4;
                        visited[n.0 * w + n.1] = true;
                        carve(&mut grid, cur, n);
                        cur = n;
                        moved = true;
                        break;
                    }
                }
            }
            if !moved {
                break;
            }
        }
        grid
    }
}

impl Generator for TrivialMaze {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let coin = rng.py.chance(0.5);
        Ok(match self.maze_type {
            MazeType::Serpentine => self.serpentine(coin),
            MazeType::Spiral => self.spiral(coin),
        })
    }

    fn size(&self) -> (usize, usize) {
        (self.h, self.w)
    }
}

#[cfg(test)]
mod tests {
    use super::super::cellgrid::check::{assert_perfect, INVALID, SIZES};
    use super::*;

    fn degree(g: &Grid, r: usize, c: usize) -> usize {
        g.neighbors((r, c), 1)
            .into_iter()
            .filter(|&(a, b)| g.is_open(a, b))
            .count()
    }

    fn assert_unicursal(g: &Grid, h: usize, w: usize) {
        let mut ones = 0;
        for i in 0..h {
            for j in 0..w {
                match degree(g, 2 * i + 1, 2 * j + 1) {
                    1 => ones += 1,
                    2 => {}
                    d => panic!("cell degree {d}"),
                }
            }
        }
        assert_eq!(ones, 2);
    }

    #[test]
    fn all_shapes_perfect_and_unicursal() {
        for (h, w) in SIZES {
            let (hu, wu) = (h as usize, w as usize);
            let t = TrivialMaze::new(h, w, MazeType::Spiral).unwrap();
            for coin in [true, false] {
                let g = t.serpentine(coin);
                assert_perfect(&g, hu, wu);
                assert_unicursal(&g, hu, wu);
                assert_eq!(degree(&g, 1, 1), 2);
                if coin {
                    assert_eq!(degree(&g, 2 * hu - 1, 1), 1);
                } else {
                    assert_eq!(degree(&g, 1, 2 * wu - 1), 1);
                }
                let s = t.spiral(coin);
                assert_perfect(&s, hu, wu);
                assert_unicursal(&s, hu, wu);
                assert_eq!(degree(&s, 1, 1), 1);
            }
        }
    }

    #[test]
    fn matches_documented_examples() {
        let t = TrivialMaze::new(3, 5, MazeType::Spiral).unwrap();
        let rows = |v: &[&str]| v.iter().map(|s| s.replace('X', "#")).collect::<Vec<_>>();
        assert_eq!(
            t.spiral(true).to_rows(),
            rows(&[
                "XXXXXXXXXXX",
                "X.X.......X",
                "X.X.XXXXX.X",
                "X.X.....X.X",
                "X.XXXXXXX.X",
                "X.........X",
                "XXXXXXXXXXX"
            ])
        );
        assert_eq!(
            t.serpentine(true).to_rows(),
            rows(&[
                "XXXXXXXXXXX",
                "X...X...X.X",
                "X.X.X.X.X.X",
                "X.X.X.X.X.X",
                "X.X.X.X.X.X",
                "X.X...X...X",
                "XXXXXXXXXXX"
            ])
        );
        assert_eq!(
            t.serpentine(false).to_rows(),
            rows(&[
                "XXXXXXXXXXX",
                "X.........X",
                "X.XXXXXXXXX",
                "X.........X",
                "XXXXXXXXX.X",
                "X.........X",
                "XXXXXXXXXXX"
            ])
        );
    }

    #[test]
    fn generate_and_parse() {
        assert_eq!(MazeType::parse(" Serpentine "), MazeType::Serpentine);
        assert_eq!(MazeType::parse("bogus"), MazeType::Spiral);
        for mt in [MazeType::Spiral, MazeType::Serpentine] {
            let mut t = TrivialMaze::new(4, 7, mt).unwrap();
            for seed in 0..8 {
                assert_perfect(&t.generate(&mut Rng::seeded(seed).unwrap()).unwrap(), 4, 7);
            }
        }
        for (h, w) in INVALID {
            assert!(matches!(
                TrivialMaze::new(h, w, MazeType::Spiral),
                Err(crate::MazeError::InvalidInput(_))
            ));
        }
    }
}
