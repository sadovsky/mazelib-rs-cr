//! Maze solvers.

mod backtracking;
mod chain;
mod collision;
mod common;
mod prune;
mod random_mouse;
mod shortest_path;
mod shortest_paths;
mod tremaux;

pub use backtracking::BacktrackingSolver;
pub use chain::{Chain, Turn};
pub use collision::{Collision, COLLISION_CAP};
pub use prune::prune_solutions;
pub use random_mouse::RandomMouse;
pub use shortest_path::ShortestPath;
pub use shortest_paths::{ShortestPaths, SHORTEST_PATHS_CAP};
pub use tremaux::Tremaux;

use crate::{Cell, Grid, Path, Result, Rng};

/// Finds routes from `start` to `end`. Each returned path lists the cells
/// strictly between `start` and `end`.
pub trait Solver {
    /// Solve `grid`.
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, rng: &mut Rng) -> Result<Vec<Path>>;
}
