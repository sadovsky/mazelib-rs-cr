//! Recursive-division generator (with an explicit work stack).

use super::{check_size, Generator};
use crate::grid::{OPEN, WALL};
use crate::{Grid, Result, Rng};

/// Splits chambers with single-door walls, cutting across the longer side.
#[derive(Debug, Clone)]
pub struct Division {
    h: usize,
    w: usize,
}

impl Division {
    /// Validate the size (both dimensions at least 3).
    pub fn new(h: i64, w: i64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(Division { h, w })
    }
}

/// A chamber in cell coordinates: top-left cell and size in cells.
#[derive(Debug, Clone, Copy)]
struct Chamber {
    r0: usize,
    c0: usize,
    hh: usize,
    ww: usize,
}

impl Generator for Division {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let (rows, cols) = (2 * h + 1, 2 * w + 1);
        let mut grid = Grid::walls(rows, cols);
        for r in 1..rows - 1 {
            for c in 1..cols - 1 {
                grid.set(r, c, OPEN);
            }
        }
        let mut work = vec![Chamber {
            r0: 0,
            c0: 0,
            hh: h,
            ww: w,
        }];
        while let Some(ch) = work.pop() {
            // A chamber one cell thick is already a simple corridor.
            if ch.hh < 2 || ch.ww < 2 {
                continue;
            }
            let horizontal = match ch.hh.cmp(&ch.ww) {
                std::cmp::Ordering::Greater => true,
                std::cmp::Ordering::Less => false,
                std::cmp::Ordering::Equal => rng.py.below(2) == 0,
            };
            if horizontal {
                // wall below cell row r0 + k, k in [0, hh - 1)
                let k = rng.py.below(ch.hh - 1);
                let gr = 2 * (ch.r0 + k) + 2;
                for gc in 2 * ch.c0 + 1..=2 * (ch.c0 + ch.ww) - 1 {
                    grid.set(gr, gc, WALL);
                }
                let door = ch.c0 + rng.py.below(ch.ww);
                grid.set(gr, 2 * door + 1, OPEN);
                work.push(Chamber { hh: k + 1, ..ch });
                work.push(Chamber {
                    r0: ch.r0 + k + 1,
                    hh: ch.hh - k - 1,
                    ..ch
                });
            } else {
                let k = rng.py.below(ch.ww - 1);
                let gc = 2 * (ch.c0 + k) + 2;
                for gr in 2 * ch.r0 + 1..=2 * (ch.r0 + ch.hh) - 1 {
                    grid.set(gr, gc, WALL);
                }
                let door = ch.r0 + rng.py.below(ch.hh);
                grid.set(2 * door + 1, gc, OPEN);
                work.push(Chamber { ww: k + 1, ..ch });
                work.push(Chamber {
                    c0: ch.c0 + k + 1,
                    ww: ch.ww - k - 1,
                    ..ch
                });
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
    use crate::MazeError;

    #[test]
    fn perfect_on_golden_sizes() {
        for (h, w) in SIZES {
            for seed in 0..8 {
                let mut rng = Rng::seeded(seed).unwrap();
                let mut g = Division::new(h, w).unwrap();
                assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
            }
        }
    }

    #[test]
    fn first_cut_across_longer_side() {
        // 3x5 is wider than tall: the first cut is a vertical wall, so some
        // interior even column is wall in every odd row but one.
        let g = Division::new(3, 5)
            .unwrap()
            .generate(&mut Rng::seeded(0).unwrap())
            .unwrap();
        let found = (1..5).any(|k| {
            let c = 2 * k;
            (1..6).filter(|&r| g.is_wall(r, c)).count() == 4
        });
        assert!(found);
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                Division::new(h, w),
                Err(MazeError::InvalidInput(_))
            ));
        }
    }
}
