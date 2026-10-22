//! Maze generators.

use crate::{Grid, Result, Rng};

/// Builds a maze grid. Constructors validate the size and options; `generate`
/// draws from `rng` and returns a `(2h+1) x (2w+1)` grid.
pub trait Generator {
    /// Produce a new grid.
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid>;
    /// Size in cells, `(h, w)`.
    fn size(&self) -> (usize, usize);
}
