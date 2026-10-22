# Spec: solve/RandomMouse

Provenance: documented (docs/MAZE_SOLVE_ALGOS.md, "Random Mouse") + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `solve/_common.md` first.

## Purpose
The simplest solver: a mouse wanders at random until it finds the cheese. It returns one, typically very long, solution. The docs recommend running it with pruning on.

## Public interface
- `RandomMouse()`: no options.
- `solve(grid, start, end) -> list<list<cell>>` (protocol algo `RandomMouse`, `params: {}`).

## Behavior
1. Always returns exactly **one** solution when the end is reachable.
2. The walk starts on the start's entry square and moves only by cell moves.
3. Each step picks uniformly at random among **all** cell moves available from the current cell, including going straight back to where it came from. This is the only difference from BacktrackingSolver.
4. It stops as soon as it has arrived (current square equal or adjacent to `end`).
5. Raw output (prune = false): the whole walk, slots and cells, in order, with every back-and-forth. It is the same shape as BacktrackingSolver's raw output, starting at the entry square and ending at the arrival square. Pruning reduces it to a simple valid path.
6. Expected raw length grows quickly with maze size (it is a simple random walk on the cell graph). The golden 10x10 cases are small enough. A port may use any equivalent random walk.
7. Unreachable end: the original hangs (or raises IndexError when the start's entry square has no move). The port must terminate and return `[]`.

## Invariants (cr)
- With prune: `nonempty_when_reachable`, `paths_valid`.
- Without prune: `nonempty_when_reachable`.

## Errors
As in `_common.md`. No others.

## Edge cases
| Input | Expected |
|---|---|
| perfect maze | pruned result is the unique route |
| end unreachable | `[]` (port) |

## Worked examples (goldens)
Grid G1 is the 7x7 `grid` field of case `solve/BacktrackingSolver/Prims3x3/oo/p/s0` in the goldens' `cases/solve.jsonl` (a 3x3-cell Prims maze; every Prims3x3 solve and transmute case uses it). In words: row 1 is open from column 1 to 5; column 1 is open from row 1 to row 5; row 5 is open from column 1 to 5; column 5 is open from row 3 to row 5; and (3,2), (3,3) form a stub off column 1. Everything else is wall.
- `solve/RandomMouse/Prims3x3/oo/p/s0`: start (3,0), end (1,6). Result one solution R1: the 7 squares going up column 1 from (3,1) to (1,1) and then along row 1 to (1,5).
- `solve/RandomMouse/Prims3x3/ii/np/s0`: start (5,3), end (3,1). The raw walk has 9 squares: it starts on the start cell, goes west to (5,1), immediately returns to the start cell, goes west to (5,1) a second time, then north through (4,1) onto the end cell. Pruned (`.../ii/p/s0`): `[[ (5,2), (5,1), (4,1) ]]`.
- `solve/RandomMouse/Prims3x3/oo/np/s0`: an 87-square raw walk for the 7-square route.

## Open questions
None yet.
