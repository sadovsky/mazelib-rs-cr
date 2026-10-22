//! Binary-tree generator.

use super::cellgrid::{blank, carve, open_cell};
use super::{check_size, Generator};
use crate::{Grid, Result, Rng};

/// Which two directions every cell may carve toward.
///
/// Following the reference library's convention, "N" means increasing row
/// index (toward the bottom of the printed grid) and "S" decreasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skew {
    /// Row + 1 or column - 1.
    NW,
    /// Row + 1 or column + 1.
    NE,
    /// Row - 1 or column - 1.
    SW,
    /// Row - 1 or column + 1.
    SE,
}

impl Skew {
    const ALL: [Skew; 4] = [Skew::NW, Skew::NE, Skew::SW, Skew::SE];

    /// Parse the exact (case-sensitive, untrimmed) option string.
    pub fn parse(s: &str) -> Option<Skew> {
        match s {
            "NW" => Some(Skew::NW),
            "NE" => Some(Skew::NE),
            "SW" => Some(Skew::SW),
            "SE" => Some(Skew::SE),
            _ => None,
        }
    }

    /// `(row delta, column delta)` of the vertical and horizontal directions.
    fn deltas(self) -> (isize, isize) {
        match self {
            Skew::NW => (1, -1),
            Skew::NE => (1, 1),
            Skew::SW => (-1, -1),
            Skew::SE => (-1, 1),
        }
    }
}

/// Every cell but one opens toward one of the skew's two directions.
#[derive(Debug, Clone)]
pub struct BinaryTree {
    h: usize,
    w: usize,
    skew: Option<Skew>,
}

impl BinaryTree {
    /// Validate the size. `skew: None` means a skew is picked at random on the
    /// first `generate` call and then reused for this object.
    pub fn new(h: i64, w: i64, skew: Option<Skew>) -> Result<Self> {
        let (h, w) = check_size(h, w)?;
        Ok(BinaryTree { h, w, skew })
    }

    /// The skew in use, if already fixed.
    pub fn skew(&self) -> Option<Skew> {
        self.skew
    }
}

fn step(v: usize, d: isize, limit: usize) -> Option<usize> {
    let n = v.checked_add_signed(d)?;
    (n < limit).then_some(n)
}

impl Generator for BinaryTree {
    fn generate(&mut self, rng: &mut Rng) -> Result<Grid> {
        // SLOPFORK-Q: the original draws the random skew at construction; our
        // constructor has no RNG, so it is drawn on first generate instead.
        let skew = match self.skew {
            Some(s) => s,
            None => {
                let s = Skew::ALL[rng.py.below(4)];
                self.skew = Some(s);
                s
            }
        };
        let (h, w) = (self.h, self.w);
        let (dr, dc) = skew.deltas();
        let mut grid = blank(h, w);
        for i in 0..h {
            for j in 0..w {
                open_cell(&mut grid, (i, j));
                let vert = step(i, dr, h).map(|r| (r, j));
                let horiz = step(j, dc, w).map(|c| (i, c));
                let target = match (vert, horiz) {
                    (Some(a), Some(b)) => Some(if rng.py.below(2) == 0 { a } else { b }),
                    (a, b) => a.or(b),
                };
                if let Some(t) = target {
                    carve(&mut grid, (i, j), t);
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
    fn perfect_on_golden_sizes() {
        let skews = [
            None,
            Some(Skew::NW),
            Some(Skew::NE),
            Some(Skew::SW),
            Some(Skew::SE),
        ];
        for skew in skews {
            for (h, w) in SIZES {
                for seed in 0..8 {
                    let mut rng = Rng::seeded(seed).unwrap();
                    let mut g = BinaryTree::new(h, w, skew).unwrap();
                    assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                    let fixed = g.skew();
                    assert!(fixed.is_some());
                    assert_perfect(&g.generate(&mut rng).unwrap(), h as usize, w as usize);
                    assert_eq!(g.skew(), fixed);
                }
            }
        }
    }

    #[test]
    fn corridors_follow_skew() {
        let (h, w) = (4usize, 7usize);
        let mut rng = Rng::seeded(0).unwrap();
        for (skew, row, col) in [
            (Skew::NW, 2 * h - 1, 1),
            (Skew::NE, 2 * h - 1, 2 * w - 1),
            (Skew::SW, 1, 1),
            (Skew::SE, 1, 2 * w - 1),
        ] {
            let g = BinaryTree::new(h as i64, w as i64, Some(skew))
                .unwrap()
                .generate(&mut rng)
                .unwrap();
            assert!((1..2 * w).all(|c| g.is_open(row, c)), "{skew:?} row");
            assert!((1..2 * h).all(|r| g.is_open(r, col)), "{skew:?} col");
        }
    }

    #[test]
    fn parse_is_exact() {
        assert_eq!(Skew::parse("NE"), Some(Skew::NE));
        assert_eq!(Skew::parse("nw"), None);
        assert_eq!(Skew::parse(" NW"), None);
    }

    #[test]
    fn rejects_small() {
        for (h, w) in INVALID {
            assert!(matches!(
                BinaryTree::new(h, w, None),
                Err(MazeError::InvalidInput(_))
            ));
        }
    }
}
