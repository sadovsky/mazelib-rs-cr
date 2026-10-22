//! BacktrackingSolver: a non-reversing random walk.

use super::common::{non_reversing, solve_walker};
use crate::{Cell, Grid, Path, Result, Rng, Solver};

/// Walks from the start's entry square by cell moves, picking uniformly among
/// the moves that don't go straight back, and reversing only at dead ends.
/// The raw output is the whole walk; pruning turns it into a simple route.
#[derive(Debug, Clone, Default)]
pub struct BacktrackingSolver;

impl BacktrackingSolver {
    /// A new solver (no options).
    pub fn new() -> Self {
        BacktrackingSolver
    }
}

impl Solver for BacktrackingSolver {
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, rng: &mut Rng) -> Result<Vec<Path>> {
        solve_walker(grid, start, end, rng, &mut non_reversing)
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::testutil::*;
    use super::*;
    use crate::solve::prune_solutions;

    #[test]
    fn golden_g1_cases() {
        for seed in 0..4 {
            let mut rng = Rng::seeded(seed).unwrap();
            let g = g1();
            let raw = BacktrackingSolver::new()
                .solve(&g, (3, 0), (1, 6), &mut rng)
                .unwrap();
            assert_eq!(raw.len(), 1);
            assert_eq!(raw[0][0], (3, 1));
            assert_eq!(*raw[0].last().unwrap(), (1, 5));
            check_walker(&g, (3, 0), (1, 6), &raw);
            let r1 = vec![(3, 1), (2, 1), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5)];
            assert_eq!(prune_solutions(raw, (3, 0), (1, 6)), vec![r1]);

            let raw = BacktrackingSolver::new()
                .solve(&g, (5, 3), (3, 1), &mut rng)
                .unwrap();
            assert_eq!(raw[0][0], (5, 3));
            assert_eq!(*raw[0].last().unwrap(), (3, 1));
            assert_eq!(
                prune_solutions(raw, (5, 3), (3, 1)),
                vec![vec![(5, 2), (5, 1), (4, 1)]]
            );
        }
    }

    #[test]
    fn loopy_and_unreachable() {
        let mut rng = Rng::seeded(1).unwrap();
        for (g, s, e) in [
            (c1(), (12, 9), (0, 9)),
            (c1(), (3, 9), (9, 7)),
            (room(), (1, 0), (7, 8)),
        ] {
            let raw = BacktrackingSolver::new().solve(&g, s, e, &mut rng).unwrap();
            check_walker(&g, s, e, &raw);
        }
        let raw = BacktrackingSolver::new()
            .solve(&split(), (1, 1), (1, 5), &mut rng)
            .unwrap();
        assert!(raw.is_empty());
        assert!(BacktrackingSolver::new()
            .solve(&g1(), (7, 1), (1, 6), &mut rng)
            .is_err());
    }
}
