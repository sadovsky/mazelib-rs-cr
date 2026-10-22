//! Cul-de-sac (loop) filling: break loops that hang off a single junction.

use super::common::{adjacent, cell_neighbors, cells, check_entrance, distance};
use super::Transmuter;
use crate::grid::{OPEN, WALL};
use crate::{Cell, Grid, Result, Rng};

/// Seals one cell of every corridor loop that leaves a junction and returns
/// to it, turning the loop into two dead-end branches.
///
/// Loops touching `start`/`end` (or the squares next to them) are left
/// alone, and a seal that would change the start-to-end distance is undone.
/// Deterministic: the RNG is unused.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CuldeSacFiller;

impl CuldeSacFiller {
    /// A cul-de-sac filler (no options).
    pub fn new() -> Self {
        CuldeSacFiller
    }
}

/// Where a walk along a corridor stopped.
struct Walk {
    /// The first cell whose degree is not 2, or the last cell before
    /// returning to the origin on a pure ring.
    stop: Cell,
    /// Every cell visited, including `stop`.
    visited: Vec<Cell>,
}

/// Walk from `origin` through `first`, following corridor cells.
fn walk(g: &Grid, origin: Cell, first: Cell, limit: usize) -> Walk {
    let (mut prev, mut cur) = (origin, first);
    let mut visited = vec![first];
    for _ in 0..limit {
        let nb = cell_neighbors(g, cur);
        if nb.len() != 2 {
            break;
        }
        let next = if nb[0] == prev { nb[1] } else { nb[0] };
        if next == origin {
            // Pure ring: stop at the last cell before the origin.
            break;
        }
        prev = cur;
        cur = next;
        visited.push(cur);
    }
    Walk { stop: cur, visited }
}

impl Transmuter for CuldeSacFiller {
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        end: Option<Cell>,
        _rng: &mut Rng,
    ) -> Result<()> {
        check_entrance(grid, start, "start")?;
        check_entrance(grid, end, "end")?;
        let entrances: Vec<Cell> = start.into_iter().chain(end).collect();
        // Protected squares: the entrances and everything next to them (this
        // covers the entry square of an outer entrance).
        let protected: Vec<Cell> = entrances
            .iter()
            .flat_map(|&e| std::iter::once(e).chain(adjacent(grid, e)))
            .collect();
        let route = match (start, end) {
            (Some(s), Some(e)) => Some((s, e)),
            _ => None,
        };
        let limit = grid.rows() * grid.cols();
        let todo: Vec<Cell> = cells(grid).collect();
        for x in todo {
            if entrances.contains(&x) || protected.contains(&x) || grid.get(x.0, x.1) != OPEN {
                continue;
            }
            let nb = cell_neighbors(grid, x);
            if nb.len() != 2 {
                continue;
            }
            let a = walk(grid, x, nb[0], limit);
            let b = walk(grid, x, nb[1], limit);
            if a.stop != b.stop {
                continue;
            }
            if a.visited
                .iter()
                .chain(&b.visited)
                .any(|p| protected.contains(p))
            {
                continue;
            }
            let before = route.map(|(s, e)| distance(grid, s, e));
            grid.set(x.0, x.1, WALL);
            if let Some((s, e)) = route {
                if before != Some(distance(grid, s, e)) {
                    grid.set(x.0, x.1, OPEN);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transmute::tests::{assert_invariants, check_case};

    fn run(rows: &[&str], start: Option<Cell>, end: Option<Cell>) -> Grid {
        let mut g = Grid::from_rows(rows);
        CuldeSacFiller::new()
            .transmute(&mut g, start, end, &mut Rng::new())
            .unwrap();
        g
    }

    /// The docs example: a loop through (1,3), (1,5), (3,5), (3,3) hanging
    /// off the junction (1,3).
    const DOC: [&str; 7] = [
        "#######", "#.....#", "#.#.#.#", "#.#...#", "#.#####", "#.....#", "#######",
    ];

    #[test]
    fn docs_example_seals_loop_cell() {
        let g = run(&DOC, Some((1, 0)), Some((5, 6)));
        assert_eq!(g.to_rows()[1], "#....##");
        let mut want: Vec<String> = DOC.iter().map(|s| s.to_string()).collect();
        want[1] = "#....##".into();
        assert_eq!(g.to_rows(), want);
        assert_invariants(
            "doc",
            &Grid::from_rows(&DOC),
            &g,
            (1, 0),
            (5, 6),
            &[
                "shape",
                "border_unchanged",
                "only_adds_walls",
                "keeps_route",
                "keeps_shortest_length",
            ],
        );
    }

    #[test]
    fn loop_with_entrance_is_kept() {
        let g = run(&DOC, Some((3, 5)), Some((5, 6)));
        assert_eq!(g, Grid::from_rows(&DOC));
    }

    #[test]
    fn isolated_ring_unchanged() {
        let ring = ["#####", "#...#", "#.#.#", "#...#", "#####"];
        assert_eq!(run(&ring, None, None), Grid::from_rows(&ring));
    }

    #[test]
    fn golden_cases() {
        check_case("CuldeSacFiller", |_| Box::new(CuldeSacFiller::new()));
    }
}
