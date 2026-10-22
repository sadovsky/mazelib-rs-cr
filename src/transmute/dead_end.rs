//! Dead-end filling: seal one dead end per pass.

use super::common::{adjacent, cells, check_entrance, on_border};
use super::Transmuter;
use crate::grid::{OPEN, WALL};
use crate::{Cell, Grid, Result, Rng};

/// Pass count used when `iterations <= 0`.
const DEFAULT_PASSES: usize = 100;

/// Seals dead ends one per pass, scanning cells in row-major order.
///
/// Cells at or orthogonally next to `start`/`end` are never sealed, and
/// border squares are never changed. Deterministic: the RNG is unused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadEndFiller {
    passes: usize,
}

impl Default for DeadEndFiller {
    fn default() -> Self {
        DeadEndFiller::new(1)
    }
}

impl DeadEndFiller {
    /// `iterations > 0` passes, or 100 passes when `iterations <= 0`.
    pub fn new(iterations: i64) -> Self {
        let passes = if iterations > 0 {
            usize::try_from(iterations).unwrap_or(usize::MAX)
        } else {
            DEFAULT_PASSES
        };
        DeadEndFiller { passes }
    }

    /// A fractional pass count: positive values are truncated toward zero
    /// (so `0.5` means no passes); zero, negative or NaN mean 100 passes.
    pub fn from_f64(iterations: f64) -> Self {
        let passes = if iterations > 0.0 {
            // `as` saturates for huge values; passes stop early anyway once
            // no dead end is left.
            iterations.trunc() as usize
        } else {
            DEFAULT_PASSES
        };
        DeadEndFiller { passes }
    }

    /// The normalized number of passes.
    pub fn passes(&self) -> usize {
        self.passes
    }
}

/// Number of open orthogonally adjacent squares of `p`.
fn open_around(g: &Grid, p: Cell) -> usize {
    adjacent(g, p)
        .into_iter()
        .filter(|&(r, c)| g.is_open(r, c))
        .count()
}

impl Transmuter for DeadEndFiller {
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        end: Option<Cell>,
        _rng: &mut Rng,
    ) -> Result<()> {
        check_entrance(grid, start, "start")?;
        check_entrance(grid, end, "end")?;
        // SLOPFORK-Q: the spec recommends a TypeError when start or end is
        // absent, but the library has no such error kind; we take the other
        // allowed option and proceed, protecting only the entrances given.
        let entrances: Vec<Cell> = start.into_iter().chain(end).collect();

        // Work on a copy where the entrance squares count as open.
        let mut work = grid.clone();
        for &(r, c) in &entrances {
            work.set(r, c, OPEN);
        }
        let protected: Vec<Cell> = entrances
            .iter()
            .flat_map(|&e| std::iter::once(e).chain(adjacent(grid, e)))
            .collect();

        for _ in 0..self.passes {
            // "Degree <= 1" tested as "at most one open adjacent square", which
            // equals the degree test on well-formed grids and is a leaf removal
            // on any grid.
            let found = cells(&work).find(|&p| {
                work.is_open(p.0, p.1) && !protected.contains(&p) && open_around(&work, p) <= 1
            });
            let Some(x) = found else { break };
            work.set(x.0, x.1, WALL);
            grid.set(x.0, x.1, WALL);
            for s in adjacent(&work, x) {
                // SLOPFORK-Q: only close a neighbouring square once it has become a dead
                // end itself, so the removal never cuts through a route.
                if on_border(&work, s)
                    || !work.is_open(s.0, s.1)
                    || entrances.contains(&s)
                    || open_around(&work, s) > 1
                {
                    continue;
                }
                work.set(s.0, s.1, WALL);
                grid.set(s.0, s.1, WALL);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transmute::tests::{check_case, G1};

    fn run(iterations: i64, start: Cell, end: Cell) -> Grid {
        let mut g = Grid::from_rows(&G1);
        let mut rng = Rng::new();
        DeadEndFiller::new(iterations)
            .transmute(&mut g, Some(start), Some(end), &mut rng)
            .unwrap();
        g
    }

    #[test]
    fn normalizes_iterations() {
        assert_eq!(DeadEndFiller::new(3).passes(), 3);
        assert_eq!(DeadEndFiller::new(0).passes(), 100);
        assert_eq!(DeadEndFiller::new(-4).passes(), 100);
        assert_eq!(DeadEndFiller::from_f64(2.7).passes(), 2);
        assert_eq!(DeadEndFiller::from_f64(0.5).passes(), 0);
        assert_eq!(DeadEndFiller::from_f64(-1.0).passes(), 100);
        assert_eq!(DeadEndFiller::default().passes(), 1);
    }

    #[test]
    fn default_outer() {
        let g = run(1, (3, 0), (1, 6));
        let mut want: Vec<String> = G1.iter().map(|s| s.to_string()).collect();
        want[3] = "#.###.#".into();
        want[4] = "#.###.#".into();
        assert_eq!(g.to_rows(), want);
    }

    #[test]
    fn three_inner_seals_top() {
        let g = run(3, (5, 3), (3, 1));
        let rows = g.to_rows();
        assert_eq!(rows[1], "#######");
        assert_eq!(rows[2], "#######");
        assert_eq!(
            &rows[3..],
            &G1[3..].iter().map(|s| s.to_string()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn entrances_untouched_and_large_count() {
        let g = run(999, (3, 0), (1, 6));
        assert!(g.is_wall(3, 0) && g.is_wall(1, 6));
        // Only the route from the start to the end survives.
        let open: usize = g.as_slice().iter().filter(|&&v| v == OPEN).count();
        assert_eq!(open, 7);
    }

    #[test]
    fn missing_entrances_and_out_of_bounds() {
        let mut g = Grid::from_rows(&G1);
        let mut rng = Rng::new();
        DeadEndFiller::new(0)
            .transmute(&mut g, None, None, &mut rng)
            .unwrap();
        let mut g = Grid::from_rows(&G1);
        assert!(DeadEndFiller::new(1)
            .transmute(&mut g, Some((9, 9)), Some((1, 6)), &mut rng)
            .is_err());
    }

    #[test]
    fn golden_cases() {
        check_case("DeadEndFiller", |p| {
            let it = p.get("iterations").copied().unwrap_or(1);
            Box::new(DeadEndFiller::new(it))
        });
    }
}
