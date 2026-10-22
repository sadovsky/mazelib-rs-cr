//! Chain: follow a guide line from start to end, sending robots around obstacles.

use super::common::{
    arrived, cell_moves, cell_route, entry_square, fallback, manhattan, non_reversing, validate,
    walk,
};
use crate::{Cell, Grid, Path, Result, Rng, Solver};

/// Which hand Chain's robots favour. It sets the order in which robots are
/// sent out (clockwise for right, counter-clockwise for left), which decides
/// ties between equally long robot walks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Turn {
    /// Robots are sent out N, W, S, E.
    Left,
    /// Robots are sent out N, E, S, W (the default).
    #[default]
    Right,
}

impl Turn {
    /// `"left"` gives [`Turn::Left`]; anything else gives [`Turn::Right`].
    pub fn parse(s: &str) -> Turn {
        if s == "left" {
            Turn::Left
        } else {
            Turn::Right
        }
    }

    /// Sort key for the move from `from` to `to` (robot launch order).
    fn rank(self, from: Cell, to: Cell) -> u8 {
        let north = to.0 < from.0;
        let south = to.0 > from.0;
        let west = to.1 < from.1;
        match (self, north, south, west) {
            (_, true, _, _) => 0,
            (_, _, true, _) => 2,
            (Turn::Right, _, _, false) | (Turn::Left, _, _, true) => 1,
            _ => 3,
        }
    }
}

/// The chain algorithm. A guide line of cells runs from the start's entry
/// square toward the end, diagonally while both coordinates are far off and
/// straight afterwards. The walker follows it; where a wall blocks the next
/// guide cell, robots (non-reversing random walkers, one per available move)
/// race to it and the shortest robot walk is kept. The raw output is the
/// concatenated walk; pruning turns it into a simple route.
#[derive(Debug, Clone, Default)]
pub struct Chain {
    turn: Turn,
}

impl Chain {
    /// A new solver with the given robot hand.
    pub fn new(turn: Turn) -> Self {
        Chain { turn }
    }

    /// The robot hand.
    pub fn turn(&self) -> Turn {
        self.turn
    }

    /// Send one robot per available move from `from` and return the shortest
    /// walk (squares after `from`) that reaches `goal`, with its final square.
    fn robots(
        &self,
        grid: &Grid,
        from: Cell,
        goal: &dyn Fn(Cell) -> bool,
        rng: &mut Rng,
    ) -> Option<(Path, Cell)> {
        let mut moves = cell_moves(grid, from);
        moves.sort_by_key(|&(_, t)| self.turn.rank(from, t));
        let mut best: Option<Path> = None;
        for (slot, tgt) in moves {
            let Some(rest) = walk(grid, tgt, Some(from), goal, rng, &mut non_reversing) else {
                continue;
            };
            let mut trip = vec![slot, tgt];
            trip.extend(rest);
            if best.as_ref().is_none_or(|b| trip.len() < b.len()) {
                best = Some(trip);
            }
        }
        best.map(|b| {
            let last = *b.last().unwrap_or(&from);
            (b, last)
        })
    }
}

/// The guide line: cells from `entry` stepping two rows and/or two columns
/// toward `end`, ending at the first one that has arrived (or can't move).
fn guide_line(entry: Cell, end: Cell) -> Vec<Cell> {
    let mut line = vec![entry];
    let mut g = entry;
    while !arrived(g, end) {
        let step = |a: usize, b: usize| {
            if a >= b + 2 {
                a - 2
            } else if b >= a + 2 {
                a + 2
            } else {
                a
            }
        };
        let n = (step(g.0, end.0), step(g.1, end.1));
        if n == g {
            break;
        }
        line.push(n);
        g = n;
    }
    line
}

/// The squares for a direct step from guide cell `a` to `b` (straight or an
/// L round a corner) if the way is open.
fn direct_step(grid: &Grid, a: Cell, b: Cell) -> Option<Path> {
    let mid = |p: Cell, q: Cell| ((p.0 + q.0) / 2, (p.1 + q.1) / 2);
    let open = |sq: &[Cell]| sq.iter().all(|p| grid.is_open(p.0, p.1));
    if a.0 == b.0 || a.1 == b.1 {
        let s = vec![mid(a, b), b];
        return open(&s).then_some(s);
    }
    for corner in [(b.0, a.1), (a.0, b.1)] {
        let s = vec![mid(a, corner), corner, mid(corner, b), b];
        if open(&s) {
            return Some(s);
        }
    }
    None
}

impl Solver for Chain {
    fn solve(&mut self, grid: &Grid, start: Cell, end: Cell, rng: &mut Rng) -> Result<Vec<Path>> {
        validate(grid, start, end)?;
        if manhattan(start, end) <= 1 {
            return Ok(vec![Vec::new()]);
        }
        let Some(entry) = entry_square(grid, start).filter(|e| grid.is_open(e.0, e.1)) else {
            return Ok(fallback(grid, start, end));
        };
        let at_end = |p: Cell| arrived(p, end);
        if cell_route(grid, entry, &at_end).is_none() {
            return Ok(fallback(grid, start, end));
        }
        let guide = guide_line(entry, end);
        let mut path = vec![entry];
        let mut cur = entry;
        let mut k = 0;
        while k + 1 < guide.len() {
            let next = guide[k + 1];
            if let Some(step) = direct_step(grid, cur, next) {
                path.extend(step);
                cur = next;
                k += 1;
                continue;
            }
            // Blocked: aim for the nearest later guide cell that can be reached.
            // Robustness beyond the original, which can hang here (Chain.md item 6).
            let target = (k + 1..guide.len()).find(|&j| {
                let g = guide[j];
                cell_route(grid, cur, &|p| p == g).is_some()
            });
            let Some(j) = target else {
                break;
            };
            let g = guide[j];
            let Some((trip, last)) = self.robots(grid, cur, &|p| p == g, rng) else {
                break;
            };
            path.extend(trip);
            cur = last;
            k = j;
        }
        if !arrived(cur, end) {
            // The guide line didn't get there (cut off, or it can't arrive):
            // a last robot leg to the end, which is reachable from here.
            match self.robots(grid, cur, &at_end, rng) {
                Some((trip, _)) => path.extend(trip),
                None => return Ok(fallback(grid, start, end)),
            }
        }
        Ok(vec![path])
    }
}

#[cfg(test)]
mod tests {
    use super::super::common::testutil::*;
    use super::*;
    use crate::solve::prune_solutions;

    #[test]
    fn turn_parse() {
        assert_eq!(Turn::parse("left"), Turn::Left);
        assert_eq!(Turn::parse("right"), Turn::Right);
        assert_eq!(Turn::parse("bogus"), Turn::Right);
        assert_eq!(Chain::default().turn(), Turn::Right);
    }

    #[test]
    fn guide_line_shape() {
        assert_eq!(guide_line((3, 1), (1, 6)), vec![(3, 1), (1, 3), (1, 5)]);
        assert_eq!(guide_line((5, 3), (3, 1)), vec![(5, 3), (3, 1)]);
    }

    #[test]
    fn golden_g1_open_guide_line() {
        let g = g1();
        let r1 = g1_route();
        for turn in [Turn::Left, Turn::Right] {
            let mut rng = Rng::seeded(0).unwrap();
            let raw = Chain::new(turn)
                .solve(&g, (3, 0), (1, 6), &mut rng)
                .unwrap();
            assert_eq!(raw, vec![r1.clone()]);
            let raw = Chain::new(turn)
                .solve(&g, (5, 3), (3, 1), &mut rng)
                .unwrap();
            check_walker(&g, (5, 3), (3, 1), &raw);
            assert_eq!(
                prune_solutions(raw, (5, 3), (3, 1)),
                vec![vec![(5, 2), (5, 1), (4, 1)]]
            );
        }
    }

    #[test]
    fn loopy_and_unreachable() {
        for seed in 0..3 {
            for turn in [Turn::Left, Turn::Right] {
                let mut rng = Rng::seeded(seed).unwrap();
                for (g, s, e) in [
                    (c1(), (12, 9), (0, 9)),
                    (c1(), (3, 9), (9, 7)),
                    (c1(), (9, 0), (3, 12)),
                    (room(), (1, 0), (7, 8)),
                    (g1(), (0, 5), (6, 1)),
                ] {
                    let raw = Chain::new(turn).solve(&g, s, e, &mut rng).unwrap();
                    assert_eq!(raw.len(), 1);
                    check_walker(&g, s, e, &raw);
                }
                assert!(Chain::new(turn)
                    .solve(&split(), (1, 1), (1, 5), &mut rng)
                    .unwrap()
                    .is_empty());
            }
        }
    }
}
