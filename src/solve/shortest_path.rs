//! ShortestPath: breadth-first search stopping at the first route found.

use super::common::{cell_moves, entry_square, manhattan, one_shortest, validate};
use crate::{Cell, Grid, MazeError, Path, Result, Rng, Solver};

/// Fail (AssertionError) when the start's entry square has no cell move,
/// as the original does for ShortestPath and ShortestPaths.
pub(crate) fn check_entry_moves(grid: &Grid, start: Cell) -> Result<()> {
    match entry_square(grid, start) {
        Some(e) if !cell_moves(grid, e).is_empty() => Ok(()),
        _ => Err(MazeError::input("input maze is invalid")),
    }
}

/// Breadth-first search over open squares from `start`; returns one shortest
/// route (strictly between `start` and `end`), or `[]` when the end can't be
/// reached. Unlike the original, inner entrances get their route too.
#[derive(Debug, Clone, Default)]
pub struct ShortestPath;

impl ShortestPath {
    /// A new solver (no options).
    pub fn new() -> Self {
        ShortestPath
    }
}

impl Solver for ShortestPath {
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, _rng: &mut Rng) -> Result<Vec<Path>> {
        validate(grid, start, end)?;
        // SLOPFORK-Q: the entry-move check runs before the d <= 1 shortcut (questions.md Q-solve-1).
        check_entry_moves(grid, start)?;
        if manhattan(start, end) <= 1 {
            return Ok(vec![Vec::new()]);
        }
        Ok(one_shortest(grid, start, end).into_iter().collect())
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
        let r1 = g1_route();
        assert_eq!(
            ShortestPath::new()
                .solve(&g, (3, 0), (1, 6), &mut rng)
                .unwrap(),
            vec![r1]
        );
        // The original returned [] here; the true shortest is returned instead.
        assert_eq!(
            ShortestPath::new()
                .solve(&g, (5, 3), (3, 1), &mut rng)
                .unwrap(),
            vec![vec![(5, 2), (5, 1), (4, 1)]]
        );
        let c = c1();
        for (s, e) in [((12, 9), (0, 9)), ((3, 9), (9, 7))] {
            let sols = ShortestPath::new().solve(&c, s, e, &mut rng).unwrap();
            assert_eq!(sols.len(), 1);
            check_shortest(&c, s, e, &sols, true);
        }
        let sols = ShortestPath::new()
            .solve(&c, (12, 9), (0, 9), &mut rng)
            .unwrap();
        assert_eq!(sols[0].len(), 23);
    }

    #[test]
    fn errors_and_unreachable() {
        let mut rng = Rng::seeded(0).unwrap();
        assert!(ShortestPath::new()
            .solve(&split(), (1, 1), (1, 5), &mut rng)
            .unwrap()
            .is_empty());
        let closed = Grid::from_rows(&["#####", "#.#.#", "#####"]);
        assert!(matches!(
            ShortestPath::new().solve(&closed, (1, 0), (1, 4), &mut rng),
            Err(MazeError::InvalidInput(_))
        ));
        assert!(matches!(
            ShortestPath::new().solve(&g1(), (1, 1), (1, 9), &mut rng),
            Err(MazeError::InvalidInput(_))
        ));
    }
}
