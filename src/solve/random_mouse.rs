//! RandomMouse: a simple random walk.

use super::common::solve_walker;
use crate::{Cell, Grid, Path, Result, Rng, Solver};

/// Walks from the start's entry square by cell moves, picking uniformly among
/// all available moves (including straight back) until it arrives. The raw
/// output is the whole walk; pruning turns it into a simple route.
#[derive(Debug, Clone, Default)]
pub struct RandomMouse;

impl RandomMouse {
    /// A new solver (no options).
    pub fn new() -> Self {
        RandomMouse
    }
}

impl Solver for RandomMouse {
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, rng: &mut Rng) -> Result<Vec<Path>> {
        let mut policy =
            |moves: &[(Cell, Cell)], _prev: Option<Cell>, rng: &mut Rng| rng.py.below(moves.len());
        solve_walker(grid, start, end, rng, &mut policy)
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::testutil::*;
    use super::*;
    use crate::solve::prune_solutions;

    #[test]
    fn golden_g1_cases() {
        let g = g1();
        for seed in 0..4 {
            let mut rng = Rng::seeded(seed).unwrap();
            let raw = RandomMouse::new()
                .solve(&g, (3, 0), (1, 6), &mut rng)
                .unwrap();
            check_walker(&g, (3, 0), (1, 6), &raw);
            let r1 = vec![(3, 1), (2, 1), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5)];
            assert_eq!(prune_solutions(raw, (3, 0), (1, 6)), vec![r1]);
            let raw = RandomMouse::new()
                .solve(&g, (5, 3), (3, 1), &mut rng)
                .unwrap();
            assert_eq!(
                prune_solutions(raw, (5, 3), (3, 1)),
                vec![vec![(5, 2), (5, 1), (4, 1)]]
            );
        }
    }

    #[test]
    fn loopy_and_unreachable() {
        let mut rng = Rng::seeded(0).unwrap();
        for (g, s, e) in [
            (c1(), (12, 9), (0, 9)),
            (c1(), (3, 9), (9, 7)),
            (room(), (1, 0), (7, 8)),
        ] {
            let raw = RandomMouse::new().solve(&g, s, e, &mut rng).unwrap();
            check_walker(&g, s, e, &raw);
        }
        assert!(RandomMouse::new()
            .solve(&split(), (1, 1), (1, 5), &mut rng)
            .unwrap()
            .is_empty());
    }
}
