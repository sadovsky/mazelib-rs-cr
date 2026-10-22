//! Helpers shared by the transmuters: cell geometry, degrees, distances.

use std::collections::VecDeque;

use crate::error::MazeError;
use crate::{Cell, Grid, Result};

/// The four orthogonal unit offsets (N, S, W, E).
pub(crate) const DIRS: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// `(r, c)` shifted by `k` times `(dr, dc)`, or `None` when it leaves the grid.
pub(crate) fn offset(g: &Grid, (r, c): Cell, (dr, dc): (isize, isize), k: isize) -> Option<Cell> {
    let nr = r as isize + dr * k;
    let nc = c as isize + dc * k;
    if nr < 0 || nc < 0 {
        return None;
    }
    let (nr, nc) = (nr as usize, nc as usize);
    g.in_bounds(nr, nc).then_some((nr, nc))
}

/// Whether `(r, c)` is a border square.
pub(crate) fn on_border(g: &Grid, (r, c): Cell) -> bool {
    r == 0 || c == 0 || r + 1 >= g.rows() || c + 1 >= g.cols()
}

/// Whether `(r, c)` is an interior odd/odd cell position.
pub(crate) fn is_cell(g: &Grid, (r, c): Cell) -> bool {
    r % 2 == 1 && c % 2 == 1 && g.in_bounds(r, c) && !on_border(g, (r, c))
}

/// Every interior cell position, row-major.
pub(crate) fn cells(g: &Grid) -> impl Iterator<Item = Cell> + '_ {
    (1..g.rows())
        .step_by(2)
        .flat_map(move |r| (1..g.cols()).step_by(2).map(move |c| (r, c)))
        .filter(move |&p| is_cell(g, p))
}

/// The cell-move neighbours of `cell`: open cells two squares away whose
/// connecting slot is open.
pub(crate) fn cell_neighbors(g: &Grid, cell: Cell) -> Vec<Cell> {
    DIRS.iter()
        .filter_map(|&d| {
            let slot = offset(g, cell, d, 1)?;
            let n = offset(g, cell, d, 2)?;
            (is_cell(g, n) && g.is_open(slot.0, slot.1) && g.is_open(n.0, n.1)).then_some(n)
        })
        .collect()
}

/// In-bounds orthogonally adjacent squares of `p`.
pub(crate) fn adjacent(g: &Grid, p: Cell) -> Vec<Cell> {
    DIRS.iter().filter_map(|&d| offset(g, p, d, 1)).collect()
}

/// Reject an entrance outside the grid.
pub(crate) fn check_entrance(g: &Grid, p: Option<Cell>, name: &str) -> Result<()> {
    match p {
        Some((r, c)) if !g.in_bounds(r, c) => Err(MazeError::input(format!(
            "{name} ({r}, {c}) lies outside the {}x{} grid",
            g.rows(),
            g.cols()
        ))),
        _ => Ok(()),
    }
}

/// Unit-step BFS distance from `start` to `end` through open squares; the
/// endpoints themselves may be walls.
pub(crate) fn distance(g: &Grid, start: Cell, end: Cell) -> Option<usize> {
    let mut seen = vec![false; g.rows() * g.cols()];
    let idx = |(r, c): Cell| r * g.cols() + c;
    if !g.in_bounds(start.0, start.1) || !g.in_bounds(end.0, end.1) {
        return None;
    }
    seen[idx(start)] = true;
    let mut q = VecDeque::from([(start, 0usize)]);
    while let Some((p, d)) = q.pop_front() {
        if p == end {
            return Some(d);
        }
        for n in adjacent(g, p) {
            if !seen[idx(n)] && (g.is_open(n.0, n.1) || n == end) {
                seen[idx(n)] = true;
                q.push_back((n, d + 1));
            }
        }
    }
    None
}
