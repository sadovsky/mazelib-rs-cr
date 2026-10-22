//! Randomized Kruskal: shuffle the interior wall slots, open each one that
//! joins two different trees.

use super::cellgrid::{blank, carve, open_cell};
use super::{check_size, Generator};
use crate::{Grid, Result, Rng};

/// Kruskal's algorithm over a union-find of cells.
#[derive(Debug, Clone)]
pub struct Kruskal {
    h: usize,
    w: usize,
}

impl Kruskal {
    /// Validate the size (at least 3x3 cells).
    pub fn new(h: i64, w: i64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(Kruskal { h, w })
    }
}

/// Union-find with path halving and union by size.
pub(crate) struct Dsu {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl Dsu {
    pub(crate) fn new(n: usize) -> Self {
        Dsu {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    pub(crate) fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    /// Merge the sets of `a` and `b`; `false` when already merged.
    pub(crate) fn union(&mut self, a: usize, b: usize) -> bool {
        let (mut ra, mut rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        if self.size[ra] < self.size[rb] {
            std::mem::swap(&mut ra, &mut rb);
        }
        self.parent[rb] = ra;
        self.size[ra] += self.size[rb];
        true
    }
}

impl Generator for Kruskal {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut grid = blank(h, w);
        for i in 0..h {
            for j in 0..w {
                open_cell(&mut grid, (i, j));
            }
        }
        // Each edge joins cell (i, j) to its south or east neighbour.
        let mut edges: Vec<((usize, usize), (usize, usize))> =
            Vec::with_capacity((h - 1) * w + h * (w - 1));
        for i in 0..h {
            for j in 0..w {
                if i + 1 < h {
                    edges.push(((i, j), (i + 1, j)));
                }
                if j + 1 < w {
                    edges.push(((i, j), (i, j + 1)));
                }
            }
        }
        rng.np.shuffle(&mut edges);
        let mut dsu = Dsu::new(h * w);
        let mut trees = h * w;
        for (a, b) in edges {
            if trees == 1 {
                break;
            }
            if dsu.union(a.0 * w + a.1, b.0 * w + b.1) {
                carve(&mut grid, a, b);
                trees -= 1;
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
    fn perfect_on_golden_sizes() {
        for (h, w) in SIZES {
            for seed in 0..8 {
                let mut g = Kruskal::new(h, w).unwrap();
                let mut rng = Rng::seeded(seed).unwrap();
                let grid = g.generate(&mut rng).unwrap();
                assert_perfect(&grid, h as usize, w as usize);
            }
        }
    }

    #[test]
    fn deterministic_and_repeatable() {
        let mut g = Kruskal::new(4, 6).unwrap();
        let a = g.generate(&mut Rng::seeded(3).unwrap()).unwrap();
        let b = g.generate(&mut Rng::seeded(3).unwrap()).unwrap();
        assert_eq!(a, b);
        assert_perfect(&g.generate(&mut Rng::seeded(4).unwrap()).unwrap(), 4, 6);
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                Kruskal::new(h, w),
                Err(crate::MazeError::InvalidInput(_))
            ));
        }
    }
}
