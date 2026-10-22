//! The facade's solution pruning (spec `maze.md` item 16).

use std::collections::HashMap;

use crate::{Cell, Path};

/// Clean each path independently: loop erasure, then endpoint trimming.
/// The list keeps its length and order; empty paths stay.
pub fn prune_solutions(paths: Vec<Path>, start: Cell, end: Cell) -> Vec<Path> {
    paths
        .into_iter()
        .map(|p| prune_one(p, start, end))
        .collect()
}

/// Loop erasure followed by endpoint trimming for one path.
fn prune_one(path: Path, start: Cell, end: Cell) -> Path {
    let mut out = erase_loops(path);
    if out.len() > 1 {
        if out.first() == Some(&start) {
            out.remove(0);
        }
        if out.last() == Some(&end) {
            out.pop();
        }
    }
    out
}

/// Repeatedly cut from the earliest square that occurs again later up to (not
/// including) its next occurrence, while the path is longer than 2 squares.
///
/// Squares before the earliest repeated one are unique and stay put, so the
/// passes advance a head pointer through the original path: a head with a
/// later occurrence jumps to that occurrence (one pass), a head without one
/// is final. This gives the pass-by-pass result in linear time.
fn erase_loops(path: Path) -> Path {
    let n = path.len();
    if n <= 2 {
        return path;
    }
    // next[i]: index of the next occurrence of path[i] after i.
    let mut next = vec![usize::MAX; n];
    let mut seen: HashMap<Cell, usize> = HashMap::new();
    for i in (0..n).rev() {
        if let Some(&j) = seen.get(&path[i]) {
            next[i] = j;
        }
        seen.insert(path[i], i);
    }
    // Is there any repeat at or after index i? (suffix flag)
    let mut repeat_from = vec![false; n + 1];
    for i in (0..n).rev() {
        repeat_from[i] = repeat_from[i + 1] || next[i] != usize::MAX;
    }
    let mut out: Path = Vec::with_capacity(n);
    let mut i = 0;
    let mut passes = 0;
    while i < n {
        // The remaining path is out ++ path[i..]. A pass is due only while some
        // square repeats and the path has more than 2 squares.
        let len = out.len() + (n - i);
        if !repeat_from[i] || len <= 2 || passes >= n {
            out.extend_from_slice(&path[i..]);
            break;
        }
        if next[i] == usize::MAX {
            out.push(path[i]);
            i += 1;
        } else {
            i = next[i];
            passes += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::common::testutil::{g1_route, straight_runs};
    use super::*;

    /// The pass-by-pass definition, for cross-checking.
    fn naive(mut p: Path) -> Path {
        let cap = p.len();
        let mut passes = 0;
        while p.len() > 2 && passes < cap {
            let hit =
                (0..p.len()).find_map(|i| (i + 1..p.len()).find(|&j| p[j] == p[i]).map(|j| (i, j)));
            match hit {
                Some((i, j)) => {
                    p.drain(i..j);
                }
                None => break,
            }
            passes += 1;
        }
        p
    }

    fn c(v: &[usize]) -> Path {
        v.iter().map(|&x| (x, 0)).collect()
    }

    #[test]
    fn matches_naive_definition() {
        let cases: Vec<Vec<usize>> = vec![
            vec![1, 2, 1],
            vec![1, 2, 1, 1],
            vec![1, 1],
            vec![1, 1, 2],
            vec![1, 2, 3, 2, 4, 1, 5],
            vec![1, 2, 3, 4, 2, 5, 3, 6],
            vec![1, 2, 3, 1, 3, 4],
            vec![5, 6, 5, 6, 5, 6, 7],
            vec![1, 2, 3, 4],
        ];
        for v in cases {
            assert_eq!(erase_loops(c(&v)), naive(c(&v)), "{v:?}");
        }
        // Pseudo-random sequences over a small alphabet.
        let mut x: u64 = 12345;
        for len in 0..40 {
            for _ in 0..20 {
                let v: Vec<usize> = (0..len)
                    .map(|_| {
                        x = x
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1442695040888963407);
                        ((x >> 33) % 5) as usize
                    })
                    .collect();
                assert_eq!(erase_loops(c(&v)), naive(c(&v)), "{v:?}");
            }
        }
    }

    #[test]
    fn golden_raw_walk_prunes_to_r1() {
        // solve/BacktrackingSolver/Prims3x3/oo/np/s0 (27 squares).
        // Out and back into the bottom-right dead end, out and back into the
        // middle dead end, then the true route.
        let round_trip = |corners: &[Cell]| {
            let out = straight_runs(corners);
            let back: Path = out.iter().rev().skip(1).copied().collect();
            out.into_iter().chain(back).collect::<Path>()
        };
        let entry = (3, 1);
        let (bottom_left, bottom_right, right_dead_end) = ((5, 1), (5, 5), (3, 5));
        let middle_dead_end = (3, 3);
        let mut raw = round_trip(&[entry, bottom_left, bottom_right, right_dead_end]);
        raw.extend(round_trip(&[entry, middle_dead_end]).into_iter().skip(1));
        raw.extend(g1_route().into_iter().skip(1));
        assert_eq!(raw.len(), 27);
        let r1: Path = g1_route();
        assert_eq!(prune_solutions(vec![raw], (3, 0), (1, 6)), vec![r1]);
    }

    #[test]
    fn trims_endpoints() {
        let raw: Path = vec![(5, 3), (5, 2), (5, 1), (4, 1), (3, 1)];
        assert_eq!(
            prune_solutions(vec![raw, vec![]], (5, 3), (3, 1)),
            vec![vec![(5, 2), (5, 1), (4, 1)], vec![]]
        );
        // A single square is never trimmed.
        assert_eq!(
            prune_solutions(vec![vec![(5, 3)]], (5, 3), (5, 3)),
            vec![vec![(5, 3)]]
        );
        // Two squares: both ends checked once.
        assert_eq!(
            prune_solutions(vec![vec![(1, 1), (1, 2)]], (1, 1), (1, 2)),
            vec![Vec::<Cell>::new()]
        );
    }
}
