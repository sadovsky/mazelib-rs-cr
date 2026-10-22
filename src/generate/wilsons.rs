//! Wilson's algorithm: loop-erased random walks into a growing tree, giving
//! a uniform spanning tree.

use super::cellgrid::{blank, carve, cell_neighbors, open_cell};
use super::{check_size, Generator, HuntOrder};
use crate::{Grid, Result, Rng};

/// Wilson's algorithm with a configurable walk-start order.
#[derive(Debug, Clone)]
pub struct Wilsons {
    h: usize,
    w: usize,
    hunt_order: HuntOrder,
}

impl Wilsons {
    /// Validate the size (at least 3x3 cells).
    pub fn new(h: i64, w: i64, hunt_order: HuntOrder) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(Wilsons { h, w, hunt_order })
    }
}

impl Generator for Wilsons {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let n = h * w;
        let mut grid = blank(h, w);
        let mut in_tree = vec![false; n];
        let first = rng.py.below(n);
        in_tree[first] = true;
        open_cell(&mut grid, (first / w, first % w));
        let mut remaining = n - 1;
        // Serpentine cursor: cells before it are all in the tree.
        let mut cursor = 0usize;
        // Last exit direction for each cell on the current walk.
        let mut exit: Vec<usize> = vec![usize::MAX; n];

        while remaining > 0 {
            let start = match self.hunt_order {
                HuntOrder::Serpentine => {
                    while in_tree[cursor] {
                        cursor += 1;
                    }
                    cursor
                }
                HuntOrder::Random => {
                    // Random picks with a bounded budget, then a scan so the
                    // generator always completes.
                    let mut pick = None;
                    for _ in 0..n.saturating_mul(4) {
                        let k = rng.py.below(n);
                        if !in_tree[k] {
                            pick = Some(k);
                            break;
                        }
                    }
                    match pick {
                        Some(k) => k,
                        None => {
                            let open: Vec<usize> = (0..n).filter(|&k| !in_tree[k]).collect();
                            open[rng.py.below(open.len())]
                        }
                    }
                }
            };

            // Random walk until the tree is hit, recording exits.
            let mut cur = start;
            while !in_tree[cur] {
                let nbrs = cell_neighbors(cur / w, cur % w, h, w);
                let (ni, nj) = nbrs[rng.py.below(nbrs.len())];
                let next = ni * w + nj;
                exit[cur] = next;
                cur = next;
            }

            // Retrace the loop-erased path into the tree.
            let mut cur = start;
            while !in_tree[cur] {
                let next = exit[cur];
                carve(&mut grid, (cur / w, cur % w), (next / w, next % w));
                in_tree[cur] = true;
                remaining -= 1;
                cur = next;
            }
        }
        Ok(grid)
    }

    fn size(&self) -> (usize, usize) {
        (self.h, self.w)
    }
}

#[cfg(test)]
mod tests {
    use super::super::cellgrid::check::{assert_perfect, INVALID, SIZES};
    use super::*;

    #[test]
    fn perfect_both_orders() {
        for order in [HuntOrder::Random, HuntOrder::Serpentine] {
            for (h, w) in SIZES {
                for seed in 0..8 {
                    let mut g = Wilsons::new(h, w, order).unwrap();
                    let grid = g.generate(&mut Rng::seeded(seed).unwrap()).unwrap();
                    assert_perfect(&grid, h as usize, w as usize);
                }
            }
        }
    }

    #[test]
    fn repeat_generate() {
        let mut g = Wilsons::new(6, 3, HuntOrder::Random).unwrap();
        let mut rng = Rng::seeded(5).unwrap();
        for _ in 0..4 {
            assert_perfect(&g.generate(&mut rng).unwrap(), 6, 3);
        }
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                Wilsons::new(h, w, HuntOrder::Random),
                Err(crate::MazeError::InvalidInput(_))
            ));
        }
    }
}
