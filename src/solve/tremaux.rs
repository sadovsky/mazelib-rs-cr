//! Tremaux: a marking walker that prefers unexplored passages.

use super::common::{arrived, entry_square, fallback, manhattan, validate, walk};
use crate::{Cell, Grid, Path, Result, Rng, Solver};

/// Trémaux's algorithm. Every cell the walker stands on gets a visit mark.
/// At a junction it prefers never-visited cells, then once-visited ones, then
/// the rest, and avoids going straight back when it has a choice. The raw
/// output is the whole walk starting at `start` itself; pruning turns it into
/// a simple route.
#[derive(Debug, Clone, Default)]
pub struct Tremaux;

impl Tremaux {
    /// A new solver (no options).
    pub fn new() -> Self {
        Tremaux
    }
}

impl Solver for Tremaux {
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, rng: &mut Rng) -> Result<Vec<Path>> {
        validate(grid, start, end)?;
        if manhattan(start, end) <= 1 {
            return Ok(vec![Vec::new()]);
        }
        let Some(entry) = entry_square(grid, start).filter(|e| grid.is_open(e.0, e.1)) else {
            return Ok(fallback(grid, start, end));
        };
        let w = grid.cols();
        let mut visits = vec![0u32; grid.rows() * w];
        let mut path = vec![start];
        visits[start.0 * w + start.1] += 1;
        if entry != start {
            path.push(entry);
            visits[entry.0 * w + entry.1] += 1;
        }
        let mut policy = |moves: &[(Cell, Cell)], prev: Option<Cell>, rng: &mut Rng| -> usize {
            let pick = if moves.len() == 1 {
                0
            } else {
                let count = |i: usize| visits[moves[i].1 .0 * w + moves[i].1 .1].min(2);
                let group = |k: u32| -> Vec<usize> {
                    (0..moves.len()).filter(|&i| count(i) == k).collect()
                };
                let fresh = group(0);
                if !fresh.is_empty() {
                    fresh[rng.py.below(fresh.len())]
                } else {
                    let once = group(1);
                    let mut g = if once.is_empty() { group(2) } else { once };
                    if g.len() > 1 {
                        if let Some(p) = prev {
                            g.retain(|&i| moves[i].1 != p);
                        }
                    }
                    if g.is_empty() {
                        // Only reachable when every remaining cell is `prev`.
                        rng.py.below(moves.len())
                    } else {
                        g[rng.py.below(g.len())]
                    }
                }
            };
            let t = moves[pick].1;
            visits[t.0 * w + t.1] += 1;
            pick
        };
        let goal = |p: Cell| arrived(p, end);
        match walk(grid, entry, None, &goal, rng, &mut policy) {
            Some(rest) => {
                path.extend(rest);
                Ok(vec![path])
            }
            None => Ok(fallback(grid, start, end)),
        }
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
            let raw = Tremaux::new().solve(&g, (3, 0), (1, 6), &mut rng).unwrap();
            assert_eq!(&raw[0][..2], &[(3, 0), (3, 1)]);
            assert_eq!(*raw[0].last().unwrap(), (1, 5));
            check_walker(&g, (3, 0), (1, 6), &raw);
            let r1 = g1_route();
            assert_eq!(prune_solutions(raw, (3, 0), (1, 6)), vec![r1]);

            let raw = Tremaux::new().solve(&g, (5, 3), (3, 1), &mut rng).unwrap();
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
        for seed in 0..3 {
            let mut rng = Rng::seeded(seed).unwrap();
            for (g, s, e) in [
                (c1(), (12, 9), (0, 9)),
                (c1(), (3, 9), (9, 7)),
                (room(), (1, 0), (7, 8)),
            ] {
                let raw = Tremaux::new().solve(&g, s, e, &mut rng).unwrap();
                check_walker(&g, s, e, &raw);
            }
            assert!(Tremaux::new()
                .solve(&split(), (1, 1), (1, 5), &mut rng)
                .unwrap()
                .is_empty());
        }
    }
}
