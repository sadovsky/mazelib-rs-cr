//! Hunt-and-kill generator.

use super::cellgrid::{blank, carve, cell_neighbors, open_cell};
use super::{check_size, Generator, HuntOrder};
use crate::{Grid, Result, Rng};

/// Random carving walks; when a walk is stuck, hunt for a visited cell with
/// an unvisited neighbour and walk again from there.
#[derive(Debug, Clone)]
pub struct HuntAndKill {
    h: usize,
    w: usize,
    hunt_order: HuntOrder,
}

impl HuntAndKill {
    /// Validate the size.
    pub fn new(h: i64, w: i64, hunt_order: HuntOrder) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(HuntAndKill { h, w, hunt_order })
    }
}

struct State {
    h: usize,
    w: usize,
    visited: Vec<bool>,
    remaining: usize,
    grid: Grid,
}

impl State {
    fn fresh_neighbors(&self, c: (usize, usize)) -> Vec<(usize, usize)> {
        cell_neighbors(c.0, c.1, self.h, self.w)
            .into_iter()
            .filter(|&(i, j)| !self.visited[i * self.w + j])
            .collect()
    }

    /// The "kill" phase: carve into unvisited cells until stuck.
    fn walk(&mut self, mut cur: (usize, usize), rng: &mut Rng) {
        loop {
            let fresh = self.fresh_neighbors(cur);
            if fresh.is_empty() {
                return;
            }
            let next = fresh[rng.py.below(fresh.len())];
            carve(&mut self.grid, cur, next);
            self.visited[next.0 * self.w + next.1] = true;
            self.remaining -= 1;
            cur = next;
        }
    }
}

impl Generator for HuntAndKill {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut st = State {
            h,
            w,
            visited: vec![false; h * w],
            remaining: h * w - 1,
            grid: blank(h, w),
        };
        let start = (rng.py.below(h), rng.py.below(w));
        st.visited[start.0 * w + start.1] = true;
        open_cell(&mut st.grid, start);
        st.walk(start, rng);

        if self.hunt_order == HuntOrder::Random {
            let trials = (2 * h + 1) * (2 * w + 1);
            for _ in 0..trials {
                if st.remaining == 0 {
                    break;
                }
                let c = (rng.py.below(h), rng.py.below(w));
                if st.visited[c.0 * w + c.1] {
                    st.walk(c, rng);
                }
            }
        }

        // Serpentine hunt (the whole algorithm for `Serpentine`, and the
        // completion guarantee for `Random`): each hunt walks from the first
        // visited cell, in row-major order, that has an unvisited neighbour.
        // Every cell before `done` is visited with no unvisited neighbour,
        // which is permanent, so the scan can always start there.
        let mut done = 0usize;
        while st.remaining > 0 {
            while st.visited[done] && st.fresh_neighbors((done / w, done % w)).is_empty() {
                done += 1;
            }
            let start = (done..h * w)
                .find(|&k| st.visited[k] && !st.fresh_neighbors((k / w, k % w)).is_empty());
            match start {
                Some(k) => st.walk((k / w, k % w), rng),
                // unreachable: the visited region is connected and the grid is
                // connected, so some visited cell borders an unvisited one
                None => break,
            }
        }
        Ok(st.grid)
    }

    fn size(&self) -> (usize, usize) {
        (self.h, self.w)
    }
}

#[cfg(test)]
mod tests {
    use super::super::cellgrid::check::{assert_perfect, INVALID, SIZES};
    use super::*;
    use crate::MazeError;

    #[test]
    fn perfect_on_golden_variants() {
        for order in [HuntOrder::Random, HuntOrder::Serpentine] {
            for (h, w) in SIZES {
                for seed in 0..8 {
                    let mut rng = Rng::seeded(seed).unwrap();
                    let mut g = HuntAndKill::new(h, w, order).unwrap();
                    assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                    assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                }
            }
        }
    }

    #[test]
    fn larger_sizes() {
        for order in [HuntOrder::Random, HuntOrder::Serpentine] {
            let g = HuntAndKill::new(60, 90, order)
                .unwrap()
                .generate(&mut Rng::seeded(5).unwrap())
                .unwrap();
            assert_perfect(&g, 60, 90);
        }
    }

    #[test]
    fn many_seeds_small_random() {
        for seed in 0..200 {
            let g = HuntAndKill::new(3, 3, HuntOrder::Random)
                .unwrap()
                .generate(&mut Rng::seeded(seed).unwrap())
                .unwrap();
            assert_perfect(&g, 3, 3);
        }
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                HuntAndKill::new(h, w, HuntOrder::Random),
                Err(MazeError::InvalidInput(_))
            ));
        }
    }
}
