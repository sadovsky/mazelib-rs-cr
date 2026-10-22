//! Maze transmuters: in-place rewrites of an existing grid.

use crate::{Cell, Grid, Result, Rng};

/// Rewrites `grid` in place, respecting `start`/`end` when set.
pub trait Transmuter {
    /// Transmute `grid`.
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        end: Option<Cell>,
        rng: &mut Rng,
    ) -> Result<()>;
}
