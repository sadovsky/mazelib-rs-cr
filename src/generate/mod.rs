//! Maze generators.
//!
//! Every generator is constructed from its size in cells (`h` rows, `w`
//! columns, signed so invalid input can be reported) plus typed options, and
//! implements [`Generator`].

use crate::{Grid, Result, Rng};

/// Builds a maze grid. Constructors validate the size and options; `generate`
/// draws from `rng` and returns a `(2h+1) x (2w+1)` grid.
pub trait Generator {
    /// Produce a new grid.
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid>;
    /// Size in cells, `(h, w)`.
    fn size(&self) -> (usize, usize);
}

/// How hunting generators pick the next starting cell (`hunt_order`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HuntOrder {
    /// Pick at random (the default).
    #[default]
    Random,
    /// Scan rows top to bottom, left to right.
    Serpentine,
}

impl HuntOrder {
    /// Parse the Python option string: trimmed, case-insensitive; anything
    /// other than `serpentine` means [`HuntOrder::Random`].
    pub fn parse(s: &str) -> Self {
        if s.trim().eq_ignore_ascii_case("serpentine") {
            HuntOrder::Serpentine
        } else {
            HuntOrder::Random
        }
    }
}

/// Validate a generator size: both dimensions must be at least 3 cells.
pub(crate) fn check_size(h: i64, w: i64) -> Result<(usize, usize)> {
    if h < 3 || w < 3 {
        return Err(crate::MazeError::input("Mazes cannot be smaller than 3x3."));
    }
    Ok((h as usize, w as usize))
}
