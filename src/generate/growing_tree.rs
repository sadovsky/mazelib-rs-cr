//! Growing-tree generator.

use super::cellgrid::{blank, carve, cell_neighbors, open_cell};
use super::{check_size, Generator};
use crate::{Grid, Result, Rng};

/// Grows a tree from an active list, extending the newest cell with
/// probability `backtrack_chance` and a random active cell otherwise.
#[derive(Debug, Clone)]
pub struct GrowingTree {
    h: usize,
    w: usize,
    backtrack_chance: f64,
}

impl GrowingTree {
    /// Validate the size. `backtrack_chance` is not validated: 1 or more
    /// always extends the newest cell, 0 or less always a random one (Python
    /// default 1.0).
    pub fn new(h: i64, w: i64, backtrack_chance: f64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(GrowingTree {
            h,
            w,
            backtrack_chance,
        })
    }
}

impl Generator for GrowingTree {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut grid = blank(h, w);
        let mut visited = vec![false; h * w];
        let start = (rng.py.below(h), rng.py.below(w));
        visited[start.0 * w + start.1] = true;
        open_cell(&mut grid, start);
        let mut active = vec![start];
        while !active.is_empty() {
            let k = if rng.py.chance(self.backtrack_chance) {
                active.len() - 1
            } else {
                rng.py.below(active.len())
            };
            let cur = active[k];
            let fresh: Vec<_> = cell_neighbors(cur.0, cur.1, h, w)
                .into_iter()
                .filter(|&(i, j)| !visited[i * w + j])
                .collect();
            if fresh.is_empty() {
                active.remove(k);
                continue;
            }
            let next = fresh[rng.py.below(fresh.len())];
            carve(&mut grid, cur, next);
            visited[next.0 * w + next.1] = true;
            active.push(next);
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
    fn perfect_on_golden_variants() {
        for p in [1.0, 0.0, 0.5, -2.0, 3.0, f64::NAN] {
            for (h, w) in SIZES {
                for seed in 0..8 {
                    let mut rng = Rng::seeded(seed).unwrap();
                    let mut g = GrowingTree::new(h, w, p).unwrap();
                    assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                    assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                }
            }
        }
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                GrowingTree::new(h, w, 1.0),
                Err(MazeError::InvalidInput(_))
            ));
        }
    }
}
