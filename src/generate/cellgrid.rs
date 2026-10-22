//! Shared helpers for cell-based generators: cell/grid coordinate mapping,
//! carving, and (in tests) a perfect-maze checker.

use crate::Grid;

/// Grid position of cell `(i, j)`.
pub(crate) fn pos(i: usize, j: usize) -> (usize, usize) {
    (2 * i + 1, 2 * j + 1)
}

/// In-bounds orthogonal neighbour cells of `(i, j)` in an `h x w` cell maze
/// (N, S, W, E order).
pub(crate) fn cell_neighbors(i: usize, j: usize, h: usize, w: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);
    if i > 0 {
        out.push((i - 1, j));
    }
    if i + 1 < h {
        out.push((i + 1, j));
    }
    if j > 0 {
        out.push((i, j - 1));
    }
    if j + 1 < w {
        out.push((i, j + 1));
    }
    out
}

/// Open cell `a`, cell `b`, and the wall slot between them (they must be adjacent).
pub(crate) fn carve(grid: &mut Grid, a: (usize, usize), b: (usize, usize)) {
    let (ar, ac) = pos(a.0, a.1);
    let (br, bc) = pos(b.0, b.1);
    grid.set(ar, ac, crate::grid::OPEN);
    grid.set(br, bc, crate::grid::OPEN);
    grid.set((ar + br) / 2, (ac + bc) / 2, crate::grid::OPEN);
}

/// Open a single cell.
pub(crate) fn open_cell(grid: &mut Grid, c: (usize, usize)) {
    let (r, k) = pos(c.0, c.1);
    grid.set(r, k, crate::grid::OPEN);
}

/// A fresh all-wall grid for an `h x w` cell maze.
pub(crate) fn blank(h: usize, w: usize) -> Grid {
    Grid::walls(2 * h + 1, 2 * w + 1)
}

#[cfg(test)]
pub(crate) mod check {
    use crate::Grid;

    /// Panic unless `g` is a perfect `h x w` maze: correct shape, frame and
    /// posts walls, cells open, connected, and exactly `h*w - 1` open slots.
    pub(crate) fn assert_perfect(g: &Grid, h: usize, w: usize) {
        assert_eq!(g.shape(), (2 * h + 1, 2 * w + 1), "shape");
        let (rows, cols) = g.shape();
        let mut open = 0usize;
        let mut adj = 0usize;
        for r in 0..rows {
            for c in 0..cols {
                let v = g.get(r, c);
                assert!(v <= 1, "alphabet");
                if r == 0 || c == 0 || r == rows - 1 || c == cols - 1 {
                    assert_eq!(v, 1, "border at {r},{c}");
                }
                if r % 2 == 0 && c % 2 == 0 {
                    assert_eq!(v, 1, "post at {r},{c}");
                }
                if r % 2 == 1 && c % 2 == 1 {
                    assert_eq!(v, 0, "cell at {r},{c}");
                }
                if v == 0 {
                    open += 1;
                    if g.is_open(r + 1, c) {
                        adj += 1;
                    }
                    if g.is_open(r, c + 1) {
                        adj += 1;
                    }
                }
            }
        }
        // connectivity via flood fill
        let mut seen = vec![false; rows * cols];
        let mut stack = vec![(1usize, 1usize)];
        seen[cols + 1] = true;
        let mut reached = 0usize;
        while let Some(p) = stack.pop() {
            reached += 1;
            for q in g.neighbors(p, 1) {
                if g.is_open(q.0, q.1) && !seen[q.0 * cols + q.1] {
                    seen[q.0 * cols + q.1] = true;
                    stack.push(q);
                }
            }
        }
        assert_eq!(reached, open, "connected");
        assert_eq!(adj, open - 1, "acyclic");
        assert_eq!(open - h * w, h * w - 1, "open slots");
    }

    /// Goldens sizes (h, w).
    pub(crate) const SIZES: [(i64, i64); 6] = [(3, 3), (3, 5), (5, 3), (4, 7), (8, 8), (12, 17)];

    /// Goldens invalid sizes.
    pub(crate) const INVALID: [(i64, i64); 6] = [(1, 1), (2, 2), (2, 5), (5, 2), (0, 3), (-1, 4)];
}
