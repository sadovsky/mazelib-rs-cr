//! ShortestPaths: all distinct shortest routes, shortest first.

use super::common::{all_shortest, manhattan, validate};
use super::shortest_path::check_entry_moves;
use crate::{Cell, Grid, MazeError, Path, Result, Rng, Solver};

/// Most solutions ShortestPaths returns; open rooms have combinatorially many.
pub const SHORTEST_PATHS_CAP: usize = 1000;

/// Breadth-first search over open squares, returning every distinct shortest
/// route (up to [`SHORTEST_PATHS_CAP`]), each strictly between `start` and
/// `end`. All have the same length, so the list is trivially shortest first.
/// The end being unreachable is an error (AssertionError), as in the original.
#[derive(Debug, Clone, Default)]
pub struct ShortestPaths;

impl ShortestPaths {
    /// A new solver (no options).
    pub fn new() -> Self {
        ShortestPaths
    }
}

impl Solver for ShortestPaths {
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, _rng: &mut Rng) -> Result<Vec<Path>> {
        validate(grid, start, end)?;
        // SLOPFORK-Q: the entry-move check runs before the d <= 1 shortcut (questions.md Q-solve-1).
        check_entry_moves(grid, start)?;
        if manhattan(start, end) <= 1 {
            return Ok(vec![Vec::new()]);
        }
        // Spec ShortestPaths item 6, "minimal" level: only shortest routes.
        let sols = all_shortest(grid, start, end, SHORTEST_PATHS_CAP);
        if sols.is_empty() {
            return Err(MazeError::input("no valid solutions"));
        }
        Ok(sols)
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::testutil::*;
    use super::*;

    #[test]
    fn golden_cases() {
        let mut rng = Rng::seeded(0).unwrap();
        let g = g1();
        let r1 = vec![(3, 1), (2, 1), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5)];
        assert_eq!(
            ShortestPaths::new()
                .solve(&g, (3, 0), (1, 6), &mut rng)
                .unwrap(),
            vec![r1]
        );
        assert_eq!(
            ShortestPaths::new()
                .solve(&g, (5, 3), (3, 1), &mut rng)
                .unwrap(),
            vec![vec![(5, 2), (5, 1), (4, 1)]]
        );
        let c = c1();
        let s = ShortestPaths::new()
            .solve(&c, (12, 9), (0, 9), &mut rng)
            .unwrap();
        assert_eq!(s.len(), 2);
        assert!(s.iter().all(|p| p.len() == 23));
        check_shortest(&c, (12, 9), (0, 9), &s, false);
    }

    #[test]
    fn rooms_and_errors() {
        let mut rng = Rng::seeded(0).unwrap();
        let r = room();
        let s = ShortestPaths::new()
            .solve(&r, (1, 0), (7, 8), &mut rng)
            .unwrap();
        check_shortest(&r, (1, 0), (7, 8), &s, false);
        assert!(matches!(
            ShortestPaths::new().solve(&split(), (1, 1), (1, 5), &mut rng),
            Err(MazeError::InvalidInput(_))
        ));
    }
}
