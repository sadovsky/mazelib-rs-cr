//! Recursive-backtracker (randomized depth-first search) generator.

use super::cellgrid::{blank, carve, cell_neighbors, open_cell};
use super::{check_size, Generator};
use crate::{Grid, Result, Rng};

/// Randomized depth-first search with an explicit stack.
#[derive(Debug, Clone)]
pub struct BacktrackingGenerator {
    h: usize,
    w: usize,
}

impl BacktrackingGenerator {
    /// Validate the size (both dimensions at least 3).
    pub fn new(h: i64, w: i64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(BacktrackingGenerator { h, w })
    }
}

impl Generator for BacktrackingGenerator {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut grid = blank(h, w);
        let mut visited = vec![false; h * w];
        let start = (rng.py.below(h), rng.py.below(w));
        visited[start.0 * w + start.1] = true;
        open_cell(&mut grid, start);
        let mut stack = vec![start];
        while let Some(&top) = stack.last() {
            let fresh: Vec<_> = cell_neighbors(top.0, top.1, h, w)
                .into_iter()
                .filter(|&(i, j)| !visited[i * w + j])
                .collect();
            if fresh.is_empty() {
                stack.pop();
                continue;
            }
            let next = fresh[rng.py.below(fresh.len())];
            carve(&mut grid, top, next);
            visited[next.0 * w + next.1] = true;
            stack.push(next);
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
                let mut g = BacktrackingGenerator::new(h, w).unwrap();
                assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
            }
        }
    }

    #[test]
    fn large_does_not_overflow_stack() {
        let mut g = BacktrackingGenerator::new(200, 300).unwrap();
        let grid = g.generate(&mut Rng::seeded(1).unwrap()).unwrap();
        assert_perfect(&grid, 200, 300);
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                BacktrackingGenerator::new(h, w),
                Err(MazeError::InvalidInput(_))
            ));
        }
    }
}
