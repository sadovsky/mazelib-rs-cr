//! Sidewinder: the top row is one corridor; every later row is split into
//! runs, and each run opens upward through one of its cells.

use super::cellgrid::{blank, carve, open_cell};
use super::{check_size, Generator};
use crate::{Grid, Result, Rng};

/// Sidewinder with a run-closing bias `skew` (probability of carving east is `1 - skew`).
#[derive(Debug, Clone)]
pub struct Sidewinder {
    h: usize,
    w: usize,
    skew: f64,
}

impl Sidewinder {
    /// Validate the size. `skew` is not validated (0.5 is the usual default).
    pub fn new(h: i64, w: i64, skew: f64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(Sidewinder { h, w, skew })
    }
}

impl Generator for Sidewinder {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut grid = blank(h, w);
        open_cell(&mut grid, (0, 0));
        for j in 1..w {
            carve(&mut grid, (0, j - 1), (0, j));
        }
        for i in 1..h {
            let mut run_start = 0usize;
            for j in 0..w {
                open_cell(&mut grid, (i, j));
                // Carve east with probability 1 - skew, i.e. unless random() < skew.
                let east = j + 1 < w && !rng.py.chance(self.skew);
                if east {
                    carve(&mut grid, (i, j), (i, j + 1));
                } else {
                    let k = run_start + rng.py.below(j - run_start + 1);
                    carve(&mut grid, (i, k), (i - 1, k));
                    run_start = j + 1;
                }
            }
        }
        Ok(grid)
    }

    fn size(&self) -> (usize, usize) {
        (self.h, self.w)
    }
}

#[cfg(test)]
mod tests {
    use super::super::cellgrid::check::{assert_perfect, INVALID, SIZES};
    use super::*;

    #[test]
    fn perfect_for_all_skews() {
        for skew in [0.5, 0.1, 0.9, 0.0, 1.0, -2.0, 7.0, f64::NAN] {
            for (h, w) in SIZES {
                for seed in 0..8 {
                    let mut g = Sidewinder::new(h, w, skew).unwrap();
                    let grid = g.generate(&mut Rng::seeded(seed).unwrap()).unwrap();
                    assert_perfect(&grid, h as usize, w as usize);
                    let w2 = 2 * w as usize;
                    assert!((1..w2).all(|c| grid.is_open(1, c)), "top corridor");
                }
            }
        }
    }

    #[test]
    fn skew_one_hangs_columns() {
        let mut g = Sidewinder::new(4, 5, 1.0).unwrap();
        let grid = g.generate(&mut Rng::seeded(0).unwrap()).unwrap();
        for r in 2..8 {
            for c in (1..10).step_by(2) {
                assert!(grid.is_open(r, c));
            }
        }
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                Sidewinder::new(h, w, 0.5),
                Err(crate::MazeError::InvalidInput(_))
            ));
        }
    }
}
