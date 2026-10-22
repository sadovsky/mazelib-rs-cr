//! Helpers shared by the solvers: validation, entry squares, cell moves,
//! breadth-first searches and a bounded random walk.

use std::collections::VecDeque;

use crate::{Cell, Grid, MazeError, Path, Result, Rng};

/// Upper bound on the number of cell moves a single random walk may take.
/// When a walk runs this long it is completed with a breadth-first route, so
/// every walker terminates while still reaching the end.
pub(crate) const MAX_WALK_MOVES: usize = 500_000;

/// The common solver validation: both entrances inside the grid.
pub(crate) fn validate(grid: &Grid, start: Cell, end: Cell) -> Result<()> {
    if !grid.in_bounds(start.0, start.1) {
        return Err(MazeError::input("start must lie inside the maze grid"));
    }
    if !grid.in_bounds(end.0, end.1) {
        return Err(MazeError::input("end must lie inside the maze grid"));
    }
    Ok(())
}

/// Manhattan distance between two squares.
pub(crate) fn manhattan(a: Cell, b: Cell) -> usize {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}

/// A walk standing on `p` has arrived at `end`.
pub(crate) fn arrived(p: Cell, end: Cell) -> bool {
    manhattan(p, end) <= 1
}

/// The interior square directly inside an entrance (the entrance itself when
/// it is not on the border). `None` when that square is outside the grid.
pub(crate) fn entry_square(grid: &Grid, c: Cell) -> Option<Cell> {
    let (h, w) = grid.shape();
    let (r, col) = c;
    let e = if r == 0 {
        (1, col)
    } else if r + 1 == h {
        (r.checked_sub(1)?, col)
    } else if col == 0 {
        (r, 1)
    } else if col + 1 == w {
        (r, col.checked_sub(1)?)
    } else {
        c
    };
    grid.in_bounds(e.0, e.1).then_some(e)
}

/// The cell moves available from `p`, as `(slot, target)` pairs in N, S, W, E order.
pub(crate) fn cell_moves(grid: &Grid, p: Cell) -> Vec<(Cell, Cell)> {
    let (r, c) = p;
    let mut out = Vec::with_capacity(4);
    let mut push = |slot: Option<Cell>, tgt: Option<Cell>| {
        if let (Some(s), Some(t)) = (slot, tgt) {
            if grid.is_open(s.0, s.1) && grid.is_open(t.0, t.1) {
                out.push((s, t));
            }
        }
    };
    push(
        r.checked_sub(1).map(|x| (x, c)),
        r.checked_sub(2).map(|x| (x, c)),
    );
    push(Some((r + 1, c)), Some((r + 2, c)));
    push(
        c.checked_sub(1).map(|x| (r, x)),
        c.checked_sub(2).map(|x| (r, x)),
    );
    push(Some((r, c + 1)), Some((r, c + 2)));
    out
}

/// Breadth-first search over cell moves from `from` to the first square
/// satisfying `goal`. Returns the squares after `from` (slot, cell, slot,
/// cell, ...) up to and including the goal; empty when `from` is a goal.
pub(crate) fn cell_route(grid: &Grid, from: Cell, goal: &dyn Fn(Cell) -> bool) -> Option<Path> {
    if goal(from) {
        return Some(Vec::new());
    }
    let (h, w) = grid.shape();
    let idx = |p: Cell| p.0 * w + p.1;
    let mut prev: Vec<Option<Cell>> = vec![None; h * w];
    let mut seen = vec![false; h * w];
    let mut queue = VecDeque::new();
    if grid.in_bounds(from.0, from.1) {
        seen[idx(from)] = true;
    }
    queue.push_back(from);
    while let Some(p) = queue.pop_front() {
        for (_, t) in cell_moves(grid, p) {
            if seen[idx(t)] {
                continue;
            }
            seen[idx(t)] = true;
            prev[idx(t)] = Some(p);
            if goal(t) {
                let mut cells = vec![t];
                let mut cur = t;
                while let Some(q) = prev[idx(cur)] {
                    if q == from {
                        break;
                    }
                    cells.push(q);
                    cur = q;
                }
                cells.reverse();
                let mut out = Vec::with_capacity(cells.len() * 2);
                let mut at = from;
                for c in cells {
                    out.push(((at.0 + c.0) / 2, (at.1 + c.1) / 2));
                    out.push(c);
                    at = c;
                }
                return Some(out);
            }
            queue.push_back(t);
        }
    }
    None
}

/// Unit-step BFS distances from `start` through open squares; `end` may be
/// entered even when it is a wall, but is never expanded. `usize::MAX` = unreached.
pub(crate) fn unit_distances(grid: &Grid, start: Cell, end: Cell) -> Vec<usize> {
    let (h, w) = grid.shape();
    let mut dist = vec![usize::MAX; h * w];
    dist[start.0 * w + start.1] = 0;
    let mut queue = VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        if p == end {
            continue;
        }
        let d = dist[p.0 * w + p.1];
        for n in grid.neighbors(p, 1) {
            let i = n.0 * w + n.1;
            if dist[i] == usize::MAX && (grid.is_open(n.0, n.1) || n == end) {
                dist[i] = d + 1;
                queue.push_back(n);
            }
        }
    }
    dist
}

/// Distinct shortest unit-step routes from `start` to `end`, each listing the
/// squares strictly between them, at most `cap` of them, in a fixed order.
/// Empty when `end` is unreachable.
pub(crate) fn all_shortest(grid: &Grid, start: Cell, end: Cell, cap: usize) -> Vec<Path> {
    if start == end {
        return vec![Vec::new()];
    }
    let w = grid.cols();
    let dist = unit_distances(grid, start, end);
    let total = dist[end.0 * w + end.1];
    if total == usize::MAX || cap == 0 {
        return Vec::new();
    }
    // Depth-first enumeration backwards from `end` along strictly decreasing
    // distances. Every square with distance k >= 1 has a neighbour at k - 1, so
    // no branch dead-ends and the work is bounded by `cap * total`.
    let mut out = Vec::new();
    // Each frame: (square, its candidate predecessors, next index to try).
    let preds = |p: Cell| -> Vec<Cell> {
        let d = dist[p.0 * w + p.1];
        grid.neighbors(p, 1)
            .into_iter()
            .filter(|&n| n != start && d > 1 && dist[n.0 * w + n.1] == d - 1)
            .collect()
    };
    let mut frames: Vec<(Cell, Vec<Cell>, usize)> = vec![(end, preds(end), 0)];
    while let Some(top) = frames.last_mut() {
        let (p, cands, next) = (top.0, &top.1, top.2);
        if dist[p.0 * w + p.1] == 1 {
            // The next step is `start`: emit the squares strictly between.
            let path: Path = frames.iter().skip(1).rev().map(|f| f.0).collect();
            out.push(path);
            if out.len() >= cap {
                break;
            }
            frames.pop();
            continue;
        }
        if next >= cands.len() {
            frames.pop();
            continue;
        }
        let n = cands[next];
        top.2 += 1;
        frames.push((n, preds(n), 0));
    }
    out
}

/// One shortest unit-step route strictly between `start` and `end`.
pub(crate) fn one_shortest(grid: &Grid, start: Cell, end: Cell) -> Option<Path> {
    all_shortest(grid, start, end, 1).pop()
}

/// The answer a walker gives when cell moves can't be used: a shortest
/// unit-step route, or nothing when the end is unreachable.
pub(crate) fn fallback(grid: &Grid, start: Cell, end: Cell) -> Vec<Path> {
    one_shortest(grid, start, end).into_iter().collect()
}

/// A walk policy: given the available `(slot, target)` moves (never empty), the
/// cell the walker came from, and the RNG, return the index of the move to take.
pub(crate) type Policy<'a> = dyn FnMut(&[(Cell, Cell)], Option<Cell>, &mut Rng) -> usize + 'a;

/// Walk by cell moves from `from` until `goal` holds, choosing moves with
/// `policy`. Returns the squares after `from`. After [`MAX_WALK_MOVES`] moves
/// the walk is finished with a breadth-first route. `None` when the goal can't
/// be reached at all from `from`.
pub(crate) fn walk(
    grid: &Grid,
    from: Cell,
    first_prev: Option<Cell>,
    goal: &dyn Fn(Cell) -> bool,
    rng: &mut Rng,
    policy: &mut Policy<'_>,
) -> Option<Path> {
    // Checking reachability first guarantees termination on cut-off goals.
    cell_route(grid, from, goal)?;
    let mut out = Vec::new();
    let mut cur = from;
    let mut prev = first_prev;
    let mut moves_taken = 0;
    while !goal(cur) {
        if moves_taken >= MAX_WALK_MOVES {
            out.extend(cell_route(grid, cur, goal)?);
            return Some(out);
        }
        let moves = cell_moves(grid, cur);
        if moves.is_empty() {
            return None;
        }
        let i = policy(&moves, prev, rng).min(moves.len() - 1);
        let (slot, tgt) = moves[i];
        out.push(slot);
        out.push(tgt);
        prev = Some(cur);
        cur = tgt;
        moves_taken += 1;
    }
    Some(out)
}

/// The non-reversing policy shared by BacktrackingSolver and Chain's robots:
/// exclude the cell just left unless it is the only move, then pick uniformly.
pub(crate) fn non_reversing(moves: &[(Cell, Cell)], prev: Option<Cell>, rng: &mut Rng) -> usize {
    if moves.len() > 1 {
        if let Some(p) = prev {
            let keep: Vec<usize> = (0..moves.len()).filter(|&i| moves[i].1 != p).collect();
            if !keep.is_empty() {
                return keep[rng.py.below(keep.len())];
            }
        }
    }
    rng.py.below(moves.len())
}

/// Run a walker that starts on the start's entry square and walks until it
/// arrives at `end`. Handles the degenerate inputs shared by all walkers.
/// Returns the raw walk (entry square first) as the single solution.
pub(crate) fn solve_walker(
    grid: &Grid,
    start: Cell,
    end: Cell,
    rng: &mut Rng,
    policy: &mut Policy<'_>,
) -> Result<Vec<Path>> {
    validate(grid, start, end)?;
    if manhattan(start, end) <= 1 {
        return Ok(vec![Vec::new()]);
    }
    let Some(entry) = entry_square(grid, start).filter(|e| grid.is_open(e.0, e.1)) else {
        return Ok(fallback(grid, start, end));
    };
    let goal = |p: Cell| arrived(p, end);
    match walk(grid, entry, None, &goal, rng, policy) {
        Some(rest) => {
            let mut path = vec![entry];
            path.extend(rest);
            Ok(vec![path])
        }
        None => Ok(fallback(grid, start, end)),
    }
}

#[cfg(test)]
pub(crate) mod testutil {
    use super::*;

    /// Grid G1 from the specs (Prims 3x3).
    pub fn g1() -> Grid {
        Grid::from_rows(&[
            "#######", "#.....#", "#.#####", "#...#.#", "#.###.#", "#.....#", "#######",
        ])
    }

    /// Grid C1 from the specs (CellularAutomaton 6x6).
    pub fn c1() -> Grid {
        Grid::from_rows(&[
            "#############",
            "#...........#",
            "#.###.###.###",
            "#.#...#.#...#",
            "#.#.###.###.#",
            "#.#.#.......#",
            "###.#####.###",
            "#.......#...#",
            "###.###.###.#",
            "#.#.......#.#",
            "#.#######.#.#",
            "#.........#.#",
            "#############",
        ])
    }

    /// An open 4x4-cell room (lots of shortest routes, loops).
    pub fn room() -> Grid {
        let mut rows = vec!["#########".to_string()];
        for _ in 0..7 {
            rows.push("#.......#".to_string());
        }
        rows.push("#########".to_string());
        Grid::from_rows(&rows)
    }

    /// Two disconnected halves: the end can't be reached.
    pub fn split() -> Grid {
        Grid::from_rows(&[
            "#######", "#.#...#", "#.#.#.#", "#.#.#.#", "#.#...#", "#.#...#", "#######",
        ])
    }

    /// The checker's distance (unit steps; start and end may be walls).
    pub fn dist(grid: &Grid, start: Cell, end: Cell) -> Option<usize> {
        let d = unit_distances(grid, start, end)[end.0 * grid.cols() + end.1];
        (d != usize::MAX).then_some(d)
    }

    /// The checker's `paths_valid` for one path.
    pub fn valid(grid: &Grid, start: Cell, end: Cell, path: &Path) -> bool {
        if path.is_empty() {
            return true;
        }
        if !path.iter().all(|p| grid.is_open(p.0, p.1)) {
            return false;
        }
        let mut chain = vec![start];
        chain.extend(path.iter().copied());
        chain.push(end);
        chain.windows(2).all(|w| manhattan(w[0], w[1]) == 1)
    }

    /// Assert `nonempty_when_reachable` and, after pruning, `paths_valid`.
    pub fn check_walker(grid: &Grid, start: Cell, end: Cell, raw: &[Path]) {
        let d = dist(grid, start, end);
        if d.is_some() {
            assert!(!raw.is_empty(), "no solutions");
            assert!(raw.iter().all(|p| !p.is_empty() || d == Some(1)));
        }
        let pruned = crate::solve::prune_solutions(raw.to_vec(), start, end);
        assert_eq!(pruned.len(), raw.len());
        for p in &pruned {
            assert!(valid(grid, start, end, p), "invalid path {p:?}");
        }
    }

    /// Assert all four invariants (shortest solvers).
    pub fn check_shortest(grid: &Grid, start: Cell, end: Cell, sols: &[Path], all: bool) {
        let d = dist(grid, start, end).expect("reachable");
        assert!(!sols.is_empty());
        assert_eq!(sols[0].len(), d - 1);
        for p in sols {
            assert!(valid(grid, start, end, p), "invalid path {p:?}");
            if all {
                assert_eq!(p.len(), d - 1);
            }
        }
        let pruned = crate::solve::prune_solutions(sols.to_vec(), start, end);
        assert_eq!(pruned, sols, "pruning must not change clean paths");
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::*;
    use super::*;

    #[test]
    fn entry_squares() {
        let g = g1();
        assert_eq!(entry_square(&g, (3, 0)), Some((3, 1)));
        assert_eq!(entry_square(&g, (1, 6)), Some((1, 5)));
        assert_eq!(entry_square(&g, (0, 3)), Some((1, 3)));
        assert_eq!(entry_square(&g, (6, 3)), Some((5, 3)));
        assert_eq!(entry_square(&g, (5, 3)), Some((5, 3)));
    }

    #[test]
    fn validation() {
        let g = g1();
        assert!(matches!(
            validate(&g, (7, 0), (1, 1)),
            Err(MazeError::InvalidInput(_))
        ));
        assert!(matches!(
            validate(&g, (1, 1), (1, 7)),
            Err(MazeError::InvalidInput(_))
        ));
        assert!(validate(&g, (6, 6), (0, 0)).is_ok());
    }

    #[test]
    fn shortest_enumeration() {
        let g = c1();
        let s = all_shortest(&g, (3, 9), (9, 7), 64);
        assert_eq!(s.len(), 2);
        assert!(s.iter().all(|p| p.len() == 19));
        assert_ne!(s[0], s[1]);
        assert_eq!(
            all_shortest(&split(), (1, 1), (1, 5), 64),
            Vec::<Path>::new()
        );
    }
}
