//! An independent clean-room implementation of the mazelib API in Rust.
//!
//! Generators build [`Grid`]s, solvers find paths through them, transmuters
//! rewrite them, and [`Maze`] ties the pieces together. All randomness flows
//! through an explicit [`Rng`], which is bit-compatible with CPython's `random`
//! and NumPy's legacy `RandomState` for the same seed.
#![forbid(unsafe_code)]

pub mod error;
pub mod generate;
pub mod grid;
pub mod maze;
pub mod rng;
pub mod solve;
pub mod transmute;

pub use error::{MazeError, Result};
pub use generate::Generator;
pub use grid::{Cell, Grid, Path};
pub use maze::Maze;
pub use rng::Rng;
pub use solve::Solver;
pub use transmute::Transmuter;
