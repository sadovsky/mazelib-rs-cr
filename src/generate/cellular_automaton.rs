//! Complexity/density wall growing: wall seeds on posts, each extended a few
//! steps into still-open neighbouring posts. Imperfect but always connected.

use super::{check_size, Generator};
use crate::grid::{OPEN, WALL};
use crate::{Grid, Result, Rng};

/// The cellular-automaton style wall grower.
#[derive(Debug, Clone)]
pub struct CellularAutomaton {
    h: usize,
    w: usize,
    complexity: f64,
    density: f64,
}

impl CellularAutomaton {
    /// Validate the size. The floats are not validated: values at most 1 are
    /// fractions, larger values are absolute counts (1.0 is the usual default).
    pub fn new(h: i64, w: i64, complexity: f64, density: f64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(CellularAutomaton {
            h,
            w,
            complexity,
            density,
        })
    }

    /// Effective `(seed basis D, seed count, step budget)` for this size.
    fn effective(&self) -> (f64, usize, usize) {
        let (h, w) = (self.h as f64, self.w as f64);
        let d = if self.density <= 1.0 {
            self.density * h * w
        } else {
            self.density
        };
        let c = if self.complexity <= 1.0 {
            self.complexity * (h + w)
        } else {
            self.complexity
        };
        let posts = (self.h + 1) * (self.w + 1);
        // `as usize` saturates: negatives and NaN become 0.
        // SLOPFORK-Q: huge float options are capped so generation always ends
        // quickly. Once every post is wall more seeds change nothing, and a
        // single growth can never take more steps than there are posts.
        let seeds = ((2.0 * d).floor() as usize).min(posts.saturating_mul(16));
        let steps = (c.floor() as usize).min(posts);
        (d, seeds, steps)
    }
}

impl Generator for CellularAutomaton {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let (rows, cols) = (2 * h + 1, 2 * w + 1);
        let mut grid = Grid::new(rows, cols, OPEN);
        for r in 0..rows {
            grid.set(r, 0, WALL);
            grid.set(r, cols - 1, WALL);
        }
        for c in 0..cols {
            grid.set(0, c, WALL);
            grid.set(rows - 1, c, WALL);
        }

        let (d, seeds, steps) = self.effective();
        for s in 0..seeds {
            // Posts are (2a, 2b) with a in 0..=h, b in 0..=w.
            let (mut r, mut c) = if (s as f64) < d {
                if rng.py.chance(0.5) {
                    let row = if rng.py.chance(0.5) { 0 } else { 2 * h };
                    (row, 2 * rng.py.below(w + 1))
                } else {
                    let col = if rng.py.chance(0.5) { 0 } else { 2 * w };
                    (2 * rng.py.below(h + 1), col)
                }
            } else {
                (2 * rng.py.below(h + 1), 2 * rng.py.below(w + 1))
            };
            grid.set(r, c, WALL);
            for _ in 0..steps {
                let nbrs = grid.neighbors((r, c), 2);
                let walls = nbrs.iter().filter(|&&(a, b)| grid.is_wall(a, b)).count();
                let open: Vec<(usize, usize)> = nbrs
                    .into_iter()
                    .filter(|&(a, b)| grid.is_open(a, b))
                    .collect();
                // Grow only while attached to other walls and not boxed in.
                if walls == 0 || open.is_empty() {
                    break;
                }
                let (nr, nc) = open[rng.py.below(open.len())];
                grid.set(nr, nc, WALL);
                grid.set((r + nr) / 2, (c + nc) / 2, WALL);
                r = nr;
                c = nc;
            }
        }
        Ok(grid)
    }

    fn size(&self) -> (usize, usize) {
        (self.h, self.w)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::cellgrid::check::{INVALID, SIZES};
    use super::*;

    /// Shape, alphabet, border walls, open cells and one open component;
    /// posts too when `posts` is set.
    pub(crate) fn assert_loose(g: &Grid, h: usize, w: usize, posts: bool) {
        let (rows, cols) = (2 * h + 1, 2 * w + 1);
        assert_eq!(g.shape(), (rows, cols), "shape");
        let mut open = 0;
        for r in 0..rows {
            for c in 0..cols {
                let v = g.get(r, c);
                assert!(v <= 1, "alphabet");
                if r == 0 || c == 0 || r == rows - 1 || c == cols - 1 {
                    assert_eq!(v, WALL, "border at {r},{c}");
                }
                if r % 2 == 1 && c % 2 == 1 {
                    assert_eq!(v, OPEN, "cell at {r},{c}");
                }
                if posts && r % 2 == 0 && c % 2 == 0 {
                    assert_eq!(v, WALL, "post at {r},{c}");
                }
                if v == OPEN {
                    open += 1;
                }
            }
        }
        let mut seen = vec![false; rows * cols];
        let mut stack = vec![(1usize, 1usize)];
        seen[cols + 1] = true;
        let mut reached = 0;
        while let Some(p) = stack.pop() {
            reached += 1;
            for q in g.neighbors(p, 1) {
                if g.is_open(q.0, q.1) && !seen[q.0 * cols + q.1] {
                    seen[q.0 * cols + q.1] = true;
                    stack.push(q);
                }
            }
        }
        assert_eq!(reached, open, "connected");
    }

    #[test]
    fn golden_variants_valid() {
        for (cx, dn) in [(1.0, 1.0), (0.5, 0.5), (2.0, 0.2)] {
            for (h, w) in SIZES {
                for seed in 0..8 {
                    let mut g = CellularAutomaton::new(h, w, cx, dn).unwrap();
                    let grid = g.generate(&mut Rng::seeded(seed).unwrap()).unwrap();
                    assert_loose(&grid, h as usize, w as usize, false);
                }
            }
        }
    }

    #[test]
    fn extreme_floats_never_fail() {
        for (cx, dn) in [
            (0.0, 0.0),
            (-1.0, -3.0),
            (1e300, 1e300),
            (f64::NAN, f64::NAN),
            (f64::INFINITY, 5.0),
            (50.0, 50.0),
        ] {
            let mut g = CellularAutomaton::new(5, 4, cx, dn).unwrap();
            let grid = g.generate(&mut Rng::seeded(2).unwrap()).unwrap();
            assert_loose(&grid, 5, 4, false);
        }
    }

    #[test]
    fn effective_counts() {
        let g = CellularAutomaton::new(4, 7, 2.0, 0.2).unwrap();
        let (_, seeds, steps) = g.effective();
        assert_eq!((seeds, steps), ((0.4f64 * 28.0).floor() as usize, 2));
        let g = CellularAutomaton::new(3, 5, 1.0, 1.0).unwrap();
        assert_eq!(g.effective().1, 30);
        assert_eq!(g.effective().2, 8);
    }

    #[test]
    fn deterministic_and_rejects_small() {
        let mut g = CellularAutomaton::new(8, 8, 1.0, 1.0).unwrap();
        let a = g.generate(&mut Rng::seeded(1).unwrap()).unwrap();
        let b = g.generate(&mut Rng::seeded(1).unwrap()).unwrap();
        assert_eq!(a, b);
        for (h, w) in INVALID {
            assert!(matches!(
                CellularAutomaton::new(h, w, 1.0, 1.0),
                Err(crate::MazeError::InvalidInput(_))
            ));
        }
    }
}
