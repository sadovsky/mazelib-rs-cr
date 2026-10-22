//! Maze transmuters: in-place rewrites of an existing grid.

use crate::{Cell, Grid, Result, Rng};

mod common;
mod cul_de_sac;
mod dead_end;
mod perturbation;

pub use cul_de_sac::CuldeSacFiller;
pub use dead_end::DeadEndFiller;
pub use perturbation::Perturbation;

/// Rewrites `grid` in place, respecting `start`/`end` when set.
pub trait Transmuter {
    /// Transmute `grid`.
    fn transmute(
        &mut self,
        grid: &mut Grid,
        start: Option<Cell>,
        end: Option<Cell>,
        rng: &mut Rng,
    ) -> Result<()>;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::common::{adjacent, distance, on_border};
    use super::Transmuter;
    use crate::grid::OPEN;
    use crate::{Cell, Grid, Rng};

    use std::collections::HashMap;

    /// Every invariant a transmuter golden case is checked against.
    /// (Space-separated, in checker order.)
    const ALL_PROPS: &str = "shape border_unchanged only_adds_walls keeps_route keeps_shortest_length perfect_if_input_perfect";

    /// The Prims 3x3 golden grid.
    pub(crate) const G1: [&str; 7] = [
        "#######", "#.....#", "#.#####", "#...#.#", "#.###.#", "#.....#", "#######",
    ];

    const CASES: &str = include_str!("testdata/transmute_cases.tsv");

    /// Whether the open squares form one connected, acyclic set.
    pub(crate) fn perfect(g: &Grid) -> bool {
        let (h, w) = g.shape();
        let mut seen = vec![false; h * w];
        let (mut n_open, mut comps, mut edges) = (0usize, 0usize, 0usize);
        for r in 0..h {
            for c in 0..w {
                if !g.is_open(r, c) {
                    continue;
                }
                n_open += 1;
                edges += [(r + 1, c), (r, c + 1)]
                    .iter()
                    .filter(|&&(a, b)| g.is_open(a, b))
                    .count();
                if seen[r * w + c] {
                    continue;
                }
                comps += 1;
                let mut stack = vec![(r, c)];
                seen[r * w + c] = true;
                while let Some(p) = stack.pop() {
                    for n in adjacent(g, p) {
                        if g.is_open(n.0, n.1) && !seen[n.0 * w + n.1] {
                            seen[n.0 * w + n.1] = true;
                            stack.push(n);
                        }
                    }
                }
            }
        }
        comps == 1 && edges == n_open - comps
    }

    /// Assert the `cr` transmute invariants named in `props`.
    pub(crate) fn assert_invariants(
        id: &str,
        before: &Grid,
        after: &Grid,
        start: Cell,
        end: Cell,
        props: &[&str],
    ) {
        assert_eq!(before.shape(), after.shape(), "{id}: shape");
        let (h, w) = before.shape();
        let squares = (0..h).flat_map(|r| (0..w).map(move |c| (r, c)));
        for (r, c) in squares {
            if props.contains(&"border_unchanged") && on_border(before, (r, c)) {
                assert_eq!(before.get(r, c), after.get(r, c), "{id}: border ({r},{c})");
            }
            if props.contains(&"only_adds_walls") {
                assert!(
                    !(before.get(r, c) != OPEN && after.get(r, c) == OPEN),
                    "{id}: opened ({r},{c})"
                );
            }
        }
        let (db, da) = (distance(before, start, end), distance(after, start, end));
        if props.contains(&"keeps_route") && db.is_some() {
            assert!(da.is_some(), "{id}: route lost");
        }
        if props.contains(&"keeps_shortest_length") && db.is_some() {
            assert_eq!(db, da, "{id}: shortest length");
        }
        if props.contains(&"perfect_if_input_perfect") && perfect(before) {
            assert!(perfect(after), "{id}: no longer perfect");
        }
    }

    fn cell(s: &str) -> Cell {
        let (r, c) = s.split_once(',').unwrap();
        (r.parse().unwrap(), c.parse().unwrap())
    }

    /// Run every golden case for `algo`, checking the invariants and, for the
    /// deterministic transmuters, the exact expected grid.
    pub(crate) fn check_case(
        algo: &str,
        make: impl Fn(&HashMap<String, i64>) -> Box<dyn Transmuter>,
    ) {
        let mut ran = 0;
        for line in CASES.lines().filter(|l| !l.is_empty()) {
            let f: Vec<&str> = line.split('\t').collect();
            let [id, name, params, start, end, seed, grid, expected] = f[..] else {
                panic!("bad fixture line");
            };
            if name != algo {
                continue;
            }
            let params: HashMap<String, i64> = params
                .split(';')
                .filter_map(|kv| kv.split_once('='))
                .map(|(k, v)| (k.to_string(), v.parse().unwrap()))
                .collect();
            let (start, end) = (cell(start), cell(end));
            let before = Grid::from_rows(&grid.split('/').collect::<Vec<_>>());
            let want = Grid::from_rows(&expected.split('/').collect::<Vec<_>>());
            let mut after = before.clone();
            let mut rng = Rng::seeded(seed.parse().unwrap()).unwrap();
            make(&params)
                .transmute(&mut after, Some(start), Some(end), &mut rng)
                .unwrap();
            // The port keeps perfection for every parameter set, so Perturbation
            // checks everything except the two wall-only properties.
            let perturbation_skips = ["only_adds_walls", "keeps_shortest_length"];
            let props: Vec<&str> = ALL_PROPS
                .split(' ')
                .filter(|p| algo != "Perturbation" || !perturbation_skips.contains(p))
                .collect();
            assert_invariants(id, &before, &after, start, end, &props);
            if algo != "Perturbation" {
                assert_eq!(after.to_rows(), want.to_rows(), "{id}: exact grid");
            }
            ran += 1;
        }
        assert!(ran > 0, "no golden cases for {algo}");
    }
}
