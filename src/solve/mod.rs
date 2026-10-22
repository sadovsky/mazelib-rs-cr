//! Maze solvers.

use crate::{Cell, Grid, Path, Result, Rng};

/// Finds routes from `start` to `end`. Each returned path lists the cells
/// strictly between `start` and `end`.
pub trait Solver {
    /// Solve `grid`.
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, rng: &mut Rng) -> Result<Vec<Path>>;
}
