//! Randomized Prim's (frontier-cell form): grow one tree from a random cell
//! by absorbing a uniformly random frontier cell each step.

use super::cellgrid::{blank, carve, cell_neighbors, open_cell};
use super::{check_size, Generator};
use crate::{Grid, Result, Rng};

/// Prim's algorithm.
#[derive(Debug, Clone)]
pub struct Prims {
    h: usize,
    w: usize,
}

impl Prims {
    /// Validate the size (at least 3x3 cells).
    pub fn new(h: i64, w: i64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(Prims { h, w })
    }
}

impl Generator for Prims {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut grid = blank(h, w);
        let mut visited = vec![false; h * w];
        let mut in_frontier = vec![false; h * w];
        let mut frontier: Vec<(usize, usize)> = Vec::new();

        let start = (rng.py.below(h), rng.py.below(w));
        open_cell(&mut grid, start);
        visited[start.0 * w + start.1] = true;
        for n in cell_neighbors(start.0, start.1, h, w) {
            in_frontier[n.0 * w + n.1] = true;
            frontier.push(n);
        }

        while !frontier.is_empty() {
            let idx = rng.py.below(frontier.len());
            let cell = frontier.swap_remove(idx);
            let neighbors = cell_neighbors(cell.0, cell.1, h, w);
            let inside: Vec<(usize, usize)> = neighbors
                .iter()
                .copied()
                .filter(|n| visited[n.0 * w + n.1])
                .collect();
            // A frontier cell always has at least one visited neighbour.
            let &link = rng.py.choose(&inside).unwrap_or(&cell);
            carve(&mut grid, cell, link);
            visited[cell.0 * w + cell.1] = true;
            for n in neighbors {
                let k = n.0 * w + n.1;
                if !visited[k] && !in_frontier[k] {
                    in_frontier[k] = true;
                    frontier.push(n);
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
    fn perfect_on_golden_sizes() {
        for (h, w) in SIZES {
            for seed in 0..8 {
                let mut g = Prims::new(h, w).unwrap();
                let grid = g.generate(&mut Rng::seeded(seed).unwrap()).unwrap();
                assert_perfect(&grid, h as usize, w as usize);
            }
        }
    }

    #[test]
    fn non_square_and_repeat() {
        let mut g = Prims::new(3, 9).unwrap();
        let mut rng = Rng::seeded(11).unwrap();
        for _ in 0..3 {
            assert_perfect(&g.generate(&mut rng).unwrap(), 3, 9);
        }
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                Prims::new(h, w),
                Err(crate::MazeError::InvalidInput(_))
            ));
        }
    }
}
