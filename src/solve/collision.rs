//! Collision: every distinct shortest route (flood-fill semantics).

use super::common::{all_shortest, manhattan, validate};
use crate::{Cell, Grid, Path, Result, Rng, Solver};

/// Most solutions Collision returns; open rooms have combinatorially many.
pub const COLLISION_CAP: usize = 64;

/// The collision solver. The documented mechanism floods from the start and
/// walls off the points where fronts collide; the surviving fronts are the
/// shortest routes. This port computes the same result directly: a
/// breadth-first search over open squares, then an enumeration of the
/// distinct shortest routes (up to [`COLLISION_CAP`]) in a fixed order.
/// That also works on mazes with loops and open rooms. Paths are already in
/// final form, so pruning leaves them unchanged.
#[derive(Debug, Clone, Default)]
pub struct Collision;

impl Collision {
    /// A new solver (no options).
    pub fn new() -> Self {
        Collision
    }
}

impl Solver for Collision {
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, _rng: &mut Rng) -> Result<Vec<Path>> {
        validate(grid, start, end)?;
        if manhattan(start, end) <= 1 {
            return Ok(vec![Vec::new()]);
        }
        Ok(all_shortest(grid, start, end, COLLISION_CAP))
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
            Collision::new()
                .solve(&g, (3, 0), (1, 6), &mut rng)
                .unwrap(),
            vec![r1]
        );
        assert_eq!(
            Collision::new()
                .solve(&g, (5, 3), (3, 1), &mut rng)
                .unwrap(),
            vec![vec![(5, 2), (5, 1), (4, 1)]]
        );
        let c = c1();
        let s = Collision::new()
            .solve(&c, (3, 9), (9, 7), &mut rng)
            .unwrap();
        assert_eq!(s.len(), 2);
        check_shortest(&c, (3, 9), (9, 7), &s, true);
    }

    #[test]
    fn known_bug_grids() {
        // CellularAutomaton6x6/oo crashed the original; a room has many routes.
        let mut rng = Rng::seeded(1).unwrap();
        let c = c1();
        let s = Collision::new()
            .solve(&c, (12, 9), (0, 9), &mut rng)
            .unwrap();
        check_shortest(&c, (12, 9), (0, 9), &s, true);
        let r = room();
        let s = Collision::new()
            .solve(&r, (1, 0), (7, 8), &mut rng)
            .unwrap();
        assert_eq!(s.len(), COLLISION_CAP);
        check_shortest(&r, (1, 0), (7, 8), &s, true);
        let mut dedup = s.clone();
        dedup.sort();
        dedup.dedup();
        assert_eq!(dedup.len(), s.len());
        assert!(Collision::new()
            .solve(&split(), (1, 1), (1, 5), &mut rng)
            .unwrap()
            .is_empty());
    }
}
