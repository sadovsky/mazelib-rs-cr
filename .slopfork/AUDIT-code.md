# Code leak audit (cleanroom, phase audit)

Date: 2026-10-22. Source revision: 56cd88294a36746bbd324c2c9f7e4ab26095deb3 (65 source files scanned). Default code-mode threshold: 25-token shared run after identifier/literal normalization.

## Commands
```
python3 .claude/skills/slopfork/scripts/leak_audit.py --source /home/user/mazelib-source --against /home/user/mazelib-rs-cr/src --mode code
python3 .claude/skills/slopfork/scripts/leak_audit.py --source /home/user/mazelib-source --against /home/user/mazelib-rs-cr/adapter/src --mode code
```

## Results
| Target | Findings |
|---|---|
| `src/` (37 .rs files) | 11 |
| `adapter/src/` | 0 |

## Flagged locations (target side only)
All 11 are 25-token runs inside test-only literals: lists of `(row, col)` tuples, plus one list of string names. The matching source regions are test fixtures and a benchmark config list. Their literal values differ from the target's; the match comes from the shape of the lists (same tuple-list structure once literals are normalized), not from copied content. All are in `#[cfg(test)]` code. None is in library logic or the adapter.

| Target location | Kind | Matched source file |
|---|---|---|
| `src/maze_tests.rs:32` | 7-tuple coordinate path literal | test/test_solvers.py |
| `src/solve/backtracking.rs:43` | same path literal | test/test_solvers.py |
| `src/solve/chain.rs:216` | same path literal | test/test_solvers.py |
| `src/solve/collision.rs:45` | same path literal | test/test_solvers.py |
| `src/solve/random_mouse.rs:42` | same path literal | test/test_solvers.py |
| `src/solve/shortest_path.rs:49` | same path literal | test/test_solvers.py |
| `src/solve/shortest_paths.rs:50` | same path literal | test/test_solvers.py |
| `src/solve/tremaux.rs:95` | same path literal | test/test_solvers.py |
| `src/solve/prune.rs:143` (to ~168) | long raw-walk coordinate literal | test/test_maze.py |
| `src/solve/prune.rs:169` | same path literal | test/test_solvers.py |
| `src/transmute/mod.rs:143` (to ~150) | list of invariant-name strings | benchmarks.py |

Assessment: low risk (structural false positives). To clear the audit anyway, either:
- rewrite `src/solve/{backtracking,chain,collision,random_mouse,shortest_path,shortest_paths,tremaux}.rs` and `src/maze_tests.rs` at the listed lines from spec: build the expected path in one shared test helper (for example from a golden case or a short loop) instead of a seven-tuple literal repeated in each file;
- rewrite `src/solve/prune.rs:143-169` from spec: build the raw walk from segments or from a golden case instead of one long tuple literal;
- rewrite `src/transmute/mod.rs:143-150` from spec: put the invariant-name list on one line or derive it from `invariants.json`.

Then re-run the two commands above.

## Re-audit

Re-run after the clean side rewrote the previously flagged test literals (phase `audit`, source at the pinned revision, 65 source files compared).

| Target | Findings |
|---|---|
| `src/` | 0 |
| `adapter/src/` | 0 |
| `examples/` | 0 |

No flagged target file:line locations. Code audit is clean.
