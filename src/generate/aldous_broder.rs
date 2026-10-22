//! Aldous-Broder random-walk generator.

use super::cellgrid::{blank, carve, cell_neighbors, open_cell};
use super::{check_size, Generator};
use crate::{Grid, Result, Rng};

/// A random walk that opens a passage whenever it enters an unvisited cell.
///
/// Like the reference library, the walk prefers unvisited neighbours when
/// any exist, which keeps the run time roughly linear in the cell count.
#[derive(Debug, Clone)]
pub struct AldousBroder {
    h: usize,
    w: usize,
}

impl AldousBroder {
    /// Validate the size (both dimensions at least 3).
    pub fn new(h: i64, w: i64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(AldousBroder { h, w })
    }
}

impl Generator for AldousBroder {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut grid = blank(h, w);
        let mut visited = vec![false; h * w];
        let mut cur = (rng.py.below(h), rng.py.below(w));
        visited[cur.0 * w + cur.1] = true;
        open_cell(&mut grid, cur);
        let mut remaining = h * w - 1;
        while remaining > 0 {
            let nbrs = cell_neighbors(cur.0, cur.1, h, w);
            let fresh: Vec<_> = nbrs
                .iter()
                .copied()
                .filter(|&(i, j)| !visited[i * w + j])
                .collect();
            if fresh.is_empty() {
                cur = nbrs[rng.py.below(nbrs.len())];
            } else {
                let next = fresh[rng.py.below(fresh.len())];
                carve(&mut grid, cur, next);
                visited[next.0 * w + next.1] = true;
                remaining -= 1;
                cur = next;
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
                let mut g = AldousBroder::new(h, w).unwrap();
                let grid = g.generate(&mut rng).unwrap();
                assert_perfect(&grid, h as usize, w as usize);
                // a second call on the same object is also valid
                assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
            }
        }
    }

    #[test]
    fn deterministic_per_seed() {
        let a = AldousBroder::new(8, 8)
            .unwrap()
            .generate(&mut Rng::seeded(3).unwrap())
            .unwrap();
        let b = AldousBroder::new(8, 8)
            .unwrap()
            .generate(&mut Rng::seeded(3).unwrap())
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                AldousBroder::new(h, w),
                Err(MazeError::InvalidInput(_))
            ));
        }
    }
}
