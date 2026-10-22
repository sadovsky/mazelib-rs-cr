//! The `Maze` facade: one grid plus its entrances, algorithms and solutions.
//!
//! [`Maze`] wires the generator, solver and transmuter modules together,
//! places entrances, renders the maze as text and offers a Monte Carlo
//! helper that builds several mazes and picks one by difficulty.

use std::fmt;

use crate::grid::{Cell, Grid, Path};
use crate::{Generator, MazeError, Result, Rng, Solver, Transmuter};

/// Upper bound on full re-placements in [`Maze::generate_entrances`]. With any
/// grid that passes the up-front feasibility checks, a retry is needed with
/// probability well under 1/2, so this cap is never reached in practice.
const MAX_ENTRANCE_TRIES: usize = 10_000;

/// A maze: a grid, optional entrances, the configured algorithms and the
/// latest solutions. All fields are public, mirroring the Python attributes.
pub struct Maze {
    /// The maze grid (`None` until generated or assigned).
    pub grid: Option<Grid>,
    /// The start cell.
    pub start: Option<Cell>,
    /// The end cell.
    pub end: Option<Cell>,
    /// The generator used by [`Maze::generate`].
    pub generator: Option<Box<dyn Generator>>,
    /// The solver used by [`Maze::solve`].
    pub solver: Option<Box<dyn Solver>>,
    /// Transmuters applied in order by [`Maze::transmute`].
    pub transmuters: Vec<Box<dyn Transmuter>>,
    /// The most recent solutions.
    pub solutions: Option<Vec<Path>>,
    /// Whether [`Maze::solve`] cleans the solver's paths (default `true`).
    pub prune: bool,
    /// The random streams every operation draws from.
    pub rng: Rng,
}

impl Default for Maze {
    fn default() -> Self {
        Maze::new()
    }
}

impl fmt::Debug for Maze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Maze")
            .field("grid", &self.grid)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("generator", &self.generator.is_some())
            .field("solver", &self.solver.is_some())
            .field("transmuters", &self.transmuters.len())
            .field("solutions", &self.solutions)
            .field("prune", &self.prune)
            .finish_non_exhaustive()
    }
}

/// Renders like `tostring(true, true)`.
impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.tostring(true, true))
    }
}

/// One Monte Carlo attempt: a maze with one entrance placement and its solutions.
struct Candidate {
    grid: Grid,
    start: Option<Cell>,
    end: Option<Cell>,
    solutions: Vec<Path>,
}

impl Candidate {
    fn has_solution(&self) -> bool {
        self.solutions.first().is_some_and(|p| !p.is_empty())
    }
}

impl Maze {
    /// An empty maze with default attributes and an unseeded (default-state) RNG.
    pub fn new() -> Self {
        Maze {
            grid: None,
            start: None,
            end: None,
            generator: None,
            solver: None,
            transmuters: Vec::new(),
            solutions: None,
            prune: true,
            rng: Rng::new(),
        }
    }

    /// An empty maze whose RNG streams are seeded with `seed`.
    /// Fails (`InvalidValue`) for seeds outside `0..=2^32-1`.
    pub fn with_seed(seed: i128) -> Result<Self> {
        let mut m = Maze::new();
        m.set_seed(seed)?;
        Ok(m)
    }

    /// Reseed both RNG streams (Python stream first, then NumPy stream).
    pub fn set_seed(&mut self, seed: i128) -> Result<()> {
        self.rng.set_seed(seed)
    }

    /// Replace the grid with a freshly generated one and clear entrances and solutions.
    pub fn generate(&mut self) -> Result<()> {
        let gen = self
            .generator
            .as_mut()
            .ok_or_else(|| MazeError::input("No maze-generation algorithm has been set."))?;
        let grid = gen.generate(&mut self.rng)?;
        self.grid = Some(grid);
        self.start = None;
        self.end = None;
        self.solutions = None;
        Ok(())
    }

    /// Place `start` and `end`, each on the border (`*_outer = true`) or on an
    /// interior cell. Only `start`/`end` are written; the grid is untouched.
    pub fn generate_entrances(&mut self, start_outer: bool, end_outer: bool) -> Result<()> {
        let grid = self
            .grid
            .as_ref()
            .ok_or_else(|| MazeError::input("No grid to place entrances on."))?;
        let (h, w) = grid.shape();
        // Number of odd indices in 1..=n-2 along each axis.
        let odd_rows = h.saturating_sub(1) / 2;
        let odd_cols = w.saturating_sub(1) / 2;
        if odd_rows == 0 || odd_cols == 0 {
            return Err(MazeError::value("Grid too small for entrances."));
        }
        let inner_count = odd_rows * odd_cols;
        if !(start_outer && end_outer) && inner_count < 2 {
            return Err(MazeError::value(
                "Grid too small for non-adjacent entrances.",
            ));
        }
        let geo = Geometry {
            h,
            w,
            odd_rows,
            odd_cols,
        };

        for _ in 0..MAX_ENTRANCE_TRIES {
            let (s, e) = match (start_outer, end_outer) {
                (true, true) => {
                    let side = self.rng.py.below(4);
                    let s = geo.outer_on(side, &mut self.rng);
                    let e = geo.outer_on(opposite(side), &mut self.rng);
                    (s, e)
                }
                (false, false) => {
                    let s = geo.inner(&mut self.rng);
                    let mut e = geo.inner(&mut self.rng);
                    while e == s {
                        e = geo.inner(&mut self.rng);
                    }
                    (s, e)
                }
                (true, false) => {
                    let s = geo.outer(&mut self.rng);
                    let e = geo.inner(&mut self.rng);
                    (s, e)
                }
                (false, true) => {
                    let s = geo.inner(&mut self.rng);
                    let e = geo.outer(&mut self.rng);
                    (s, e)
                }
            };
            if manhattan(s, e) >= 2 {
                self.start = Some(s);
                self.end = Some(e);
                return Ok(());
            }
        }
        Err(MazeError::value("Could not place non-adjacent entrances."))
    }

    /// Run the solver from `start` to `end` and store its paths (pruned when `prune`).
    pub fn solve(&mut self) -> Result<()> {
        let solver = self
            .solver
            .as_mut()
            .ok_or_else(|| MazeError::input("No maze-solving algorithm has been set."))?;
        let (start, end) = match (self.start, self.end) {
            (Some(s), Some(e)) => (s, e),
            _ => return Err(MazeError::input("Start and end times must be set first.")),
        };
        let grid = self
            .grid
            .as_ref()
            .ok_or_else(|| MazeError::input("Maze grid is not set."))?;
        if !grid.in_bounds(start.0, start.1) || !grid.in_bounds(end.0, end.1) {
            return Err(MazeError::input("Entrances must lie inside the maze."));
        }
        // The solver gets a shared reference, so the facade's grid is unchanged.
        let paths = solver.solve(grid, start, end, &mut self.rng)?;
        self.solutions = Some(if self.prune {
            crate::solve::prune_solutions(paths, start, end)
        } else {
            paths
        });
        Ok(())
    }

    /// Apply every transmuter in order, in place on the grid.
    pub fn transmute(&mut self) -> Result<()> {
        let grid = self
            .grid
            .as_mut()
            .ok_or_else(|| MazeError::input("No maze to transmute."))?;
        for t in self.transmuters.iter_mut() {
            t.transmute(grid, self.start, self.end, &mut self.rng)?;
        }
        Ok(())
    }

    /// Render the grid: `#` wall, space open, optionally `S`/`E` for the
    /// entrances and `+` for the first solution path. Rows are joined by `\n`
    /// with no trailing newline. Returns `""` when there is no grid. Overlay
    /// cells outside the grid are ignored.
    pub fn tostring(&self, entrances: bool, show_solutions: bool) -> String {
        let Some(grid) = self.grid.as_ref() else {
            return String::new();
        };
        let (h, w) = grid.shape();
        let mut chars: Vec<Vec<char>> = (0..h)
            .map(|r| {
                (0..w)
                    .map(|c| if grid.get(r, c) == 0 { ' ' } else { '#' })
                    .collect()
            })
            .collect();
        let mut put = |cell: Cell, ch: char| {
            if let Some(slot) = chars.get_mut(cell.0).and_then(|row| row.get_mut(cell.1)) {
                *slot = ch;
            }
        };
        if entrances {
            if let (Some(s), Some(e)) = (self.start, self.end) {
                put(s, 'S');
                put(e, 'E');
            }
        }
        if show_solutions {
            if let Some(first) = self.solutions.as_ref().and_then(|s| s.first()) {
                for &cell in first {
                    put(cell, '+');
                }
            }
        }
        let lines: Vec<String> = chars
            .into_iter()
            .map(|row| row.into_iter().collect())
            .collect();
        lines.join("\n")
    }

    /// [`Maze::generate_monte_carlo_with`] using the first path's length as the score.
    pub fn generate_monte_carlo(
        &mut self,
        repeat: usize,
        entrances: usize,
        difficulty: f64,
    ) -> Result<()> {
        self.generate_monte_carlo_with(repeat, entrances, difficulty, |p: &Path| p.len() as f64)
    }

    /// Build `repeat` mazes, try `entrances` outer/outer placements on each and
    /// keep the best one per maze, order the mazes by `reducer(first path)`
    /// and keep the one at `floor((repeat - 1) * difficulty)`.
    ///
    /// Transmuters are not applied. `difficulty` must be in `[0, 1]`.
    pub fn generate_monte_carlo_with<F>(
        &mut self,
        repeat: usize,
        entrances: usize,
        difficulty: f64,
        reducer: F,
    ) -> Result<()>
    where
        F: Fn(&Path) -> f64,
    {
        // `contains` is false for NaN, as required.
        if !(0.0..=1.0).contains(&difficulty) {
            return Err(MazeError::input("Maze difficulty must be between 0 and 1."));
        }
        if repeat < 1 {
            return Err(MazeError::input("Monte Carlo repeat must be at least 1."));
        }
        if entrances < 1 {
            return Err(MazeError::input(
                "Monte Carlo entrances must be at least 1.",
            ));
        }

        let mut kept: Vec<Candidate> = Vec::with_capacity(repeat);
        for _ in 0..repeat {
            self.generate()?;
            let mut best: Option<Candidate> = None;
            for _ in 0..entrances {
                self.generate_entrances(true, true)?;
                self.solve()?;
                let cand = Candidate {
                    grid: self.grid.clone().unwrap_or_else(|| Grid::walls(0, 0)),
                    start: self.start,
                    end: self.end,
                    solutions: self.solutions.clone().unwrap_or_default(),
                };
                // Prefer a candidate with a non-empty first path, then the
                // most solution paths; the earliest attempt wins ties.
                let better = match &best {
                    None => true,
                    Some(b) => {
                        (cand.has_solution(), cand.solutions.len())
                            > (b.has_solution(), b.solutions.len())
                    }
                };
                if better {
                    best = Some(cand);
                }
            }
            if let Some(b) = best {
                kept.push(b);
            }
        }

        let empty: Path = Vec::new();
        let mut scored: Vec<(f64, Candidate)> = kept
            .into_iter()
            .map(|c| (reducer(c.solutions.first().unwrap_or(&empty)), c))
            .collect();
        // Stable sort; NaN scores compare as equal so the order stays defined.
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let n = scored.len();
        let idx = (((n - 1) as f64) * difficulty).floor() as usize;
        let (_, chosen) = scored.swap_remove(idx.min(n - 1));
        self.grid = Some(chosen.grid);
        self.start = chosen.start;
        self.end = chosen.end;
        self.solutions = Some(chosen.solutions);
        Ok(())
    }
}

/// Entrance geometry of an `h` x `w` grid.
struct Geometry {
    h: usize,
    w: usize,
    odd_rows: usize,
    odd_cols: usize,
}

impl Geometry {
    /// A uniform odd index in `1..=n-2` given the count of such indices.
    fn odd(count: usize, rng: &mut Rng) -> usize {
        2 * rng.py.below(count) + 1
    }

    /// A uniform aligned position on `side` (0 north, 1 south, 2 west, 3 east).
    fn outer_on(&self, side: usize, rng: &mut Rng) -> Cell {
        match side {
            0 => (0, Self::odd(self.odd_cols, rng)),
            1 => (self.h - 1, Self::odd(self.odd_cols, rng)),
            2 => (Self::odd(self.odd_rows, rng), 0),
            _ => (Self::odd(self.odd_rows, rng), self.w - 1),
        }
    }

    /// A uniform side, then a uniform aligned position on it.
    fn outer(&self, rng: &mut Rng) -> Cell {
        let side = rng.py.below(4);
        self.outer_on(side, rng)
    }

    /// A uniform interior odd/odd cell.
    fn inner(&self, rng: &mut Rng) -> Cell {
        let r = Self::odd(self.odd_rows, rng);
        let c = Self::odd(self.odd_cols, rng);
        (r, c)
    }
}

fn opposite(side: usize) -> usize {
    match side {
        0 => 1,
        1 => 0,
        2 => 3,
        _ => 2,
    }
}

fn manhattan(a: Cell, b: Cell) -> usize {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}

#[cfg(test)]
#[path = "maze_tests.rs"]
mod tests;
