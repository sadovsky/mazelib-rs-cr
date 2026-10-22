//! Eller's row-by-row generator, with a union-find over cell indices so set
//! labels can never overflow.

use super::cellgrid::{blank, carve, open_cell};
use super::{check_size, Generator};
use crate::{Grid, Result, Rng};

/// Eller's algorithm with tunable horizontal and vertical join probabilities.
#[derive(Debug, Clone)]
pub struct Ellers {
    h: usize,
    w: usize,
    xskew: f64,
    yskew: f64,
}

impl Ellers {
    /// Validate the size. `xskew` (horizontal join chance) and `yskew` (extra
    /// downward join chance) are clamped to `[0, 1]`; Python defaults are 0.5.
    pub fn new(h: i64, w: i64, xskew: f64, yskew: f64) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(Ellers {
            h,
            w,
            xskew: clamp01(xskew),
            yskew: clamp01(yskew),
        })
    }

    /// The clamped `(xskew, yskew)`.
    pub fn skews(&self) -> (f64, f64) {
        (self.xskew, self.yskew)
    }
}

fn clamp01(v: f64) -> f64 {
    // NaN stays NaN, and `random() < NaN` is always false.
    v.clamp(0.0, 1.0)
}

struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Dsu {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    /// Merge; returns false when already joined.
    fn union(&mut self, a: usize, b: usize) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        self.parent[rb] = ra;
        true
    }
}

impl Generator for Ellers {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        let (h, w) = (self.h, self.w);
        let mut grid = blank(h, w);
        let mut dsu = Dsu::new(h * w);
        let idx = |i: usize, j: usize| i * w + j;
        for i in 0..h {
            for j in 0..w {
                open_cell(&mut grid, (i, j));
            }
            let last = i + 1 == h;
            // horizontal joins
            for j in 0..w - 1 {
                let join = last || rng.py.chance(self.xskew);
                if join && dsu.union(idx(i, j), idx(i, j + 1)) {
                    carve(&mut grid, (i, j), (i, j + 1));
                }
            }
            if last {
                break;
            }
            // group the row's cells by set, in order of first appearance
            let mut groups: Vec<(usize, Vec<usize>)> = Vec::new();
            for j in 0..w {
                let root = dsu.find(idx(i, j));
                match groups.iter_mut().find(|(r, _)| *r == root) {
                    Some((_, members)) => members.push(j),
                    None => groups.push((root, vec![j])),
                }
            }
            let mut down = vec![false; w];
            for (_, members) in &groups {
                let pick = members[rng.py.below(members.len())];
                down[pick] = true;
            }
            for d in down.iter_mut() {
                if !*d && rng.py.chance(self.yskew) {
                    *d = true;
                }
            }
            for (j, &d) in down.iter().enumerate() {
                if d {
                    dsu.union(idx(i, j), idx(i + 1, j));
                    carve(&mut grid, (i, j), (i + 1, j));
                }
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
    use crate::MazeError;

    #[test]
    fn perfect_on_golden_variants() {
        for (x, y) in [(0.5, 0.5), (0.1, 0.9), (0.9, 0.1)] {
            for (h, w) in SIZES {
                for seed in 0..8 {
                    let mut rng = Rng::seeded(seed).unwrap();
                    let mut g = Ellers::new(h, w, x, y).unwrap();
                    assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                    assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                }
            }
        }
    }

    #[test]
    fn known_bug_case_is_fine() {
        for seed in 0..8 {
            let g = Ellers::new(12, 17, 0.9, 0.1)
                .unwrap()
                .generate(&mut Rng::seeded(seed).unwrap())
                .unwrap();
            assert_eq!(g.shape(), (25, 35));
            assert_perfect(&g, 12, 17);
        }
    }

    #[test]
    fn extremes_and_clamping() {
        for (x, y) in [
            (0.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
            (1.0, 0.0),
            (-3.0, 7.0),
            (f64::NAN, f64::NAN),
        ] {
            let g = Ellers::new(6, 9, x, y)
                .unwrap()
                .generate(&mut Rng::seeded(1).unwrap())
                .unwrap();
            assert_perfect(&g, 6, 9);
        }
        assert_eq!(Ellers::new(3, 3, -1.0, 2.0).unwrap().skews(), (0.0, 1.0));
        // xskew = 0: no horizontal openings above the last row
        let g = Ellers::new(5, 6, 0.0, 0.5)
            .unwrap()
            .generate(&mut Rng::seeded(2).unwrap())
            .unwrap();
        for i in 0..4 {
            for j in 0..5 {
                assert!(g.is_wall(2 * i + 1, 2 * j + 2));
            }
        }
        // xskew = 1: the first row is a full corridor, and with yskew = 0
        // every row is (only one cell drops down, the rest are fresh sets).
        // SLOPFORK-Q: the spec's "every row is one corridor" for xskew = 1
        // conflicts with "never join cells already in the same set" when
        // several cells drop down from one set; we follow the no-loop rule.
        let g = Ellers::new(5, 6, 1.0, 0.5)
            .unwrap()
            .generate(&mut Rng::seeded(2).unwrap())
            .unwrap();
        assert!((1..12).all(|c| g.is_open(1, c)));
        let g = Ellers::new(5, 6, 1.0, 0.0)
            .unwrap()
            .generate(&mut Rng::seeded(2).unwrap())
            .unwrap();
        for i in 0..5 {
            assert!((1..12).all(|c| g.is_open(2 * i + 1, c)));
        }
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                Ellers::new(h, w, 0.5, 0.5),
                Err(MazeError::InvalidInput(_))
            ));
        }
    }
}
