# Spec: solve/Chain

Provenance: documented (docs/MAZE_SOLVE_ALGOS.md, "Chain Algorithm") + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `solve/_common.md` first.

## Purpose
The chain algorithm splits the maze into a sequence of small sub-mazes along a straight-ish "guide line" from the start to the end. The walker follows the line. Where a wall blocks the next step, it sends out sub-solvers ("robots") to get around the obstacle to the next point of the line, and then continues. It returns one solution, not necessarily the shortest, and works on imperfect mazes.

## Public interface
- `Chain(turn: string = 'right')`. The documented values are `'left'` and `'right'`; the docs say turn picks which wall the robots follow. Any value other than `'left'` means `'right'`. It never raises.
- `solve(grid, start, end) -> list<list<cell>>` (protocol algo `Chain`, `params: {}` or `{"turn": "left"}`).

## Behavior
1. Always returns exactly **one** solution when the end is reachable.
2. **Guide line**: a sequence of cells beginning at the start's entry square. From each guide cell the next one is two rows toward the end's row (if the row differs by more than 1) and/or two columns toward the end's column (if the column differs by more than 1). So the line runs diagonally while both coordinates are far off and straight afterwards. It ends at the first guide cell that has arrived at the end (equal to it, or adjacent for an outer end). Walls are ignored when drawing the line.
3. **Following the line**: the solution starts at the first guide cell. To advance from guide cell k to guide cell k+1:
   - If they are in a straight line (two apart) and the slot between them is open, step through the slot.
   - If they are diagonal (two rows and two columns apart), go round the corner. Prefer the vertical-first L, which moves two rows first and then two columns. Use it if the slot leaving k vertically and the slot entering k+1 horizontally are both open. Otherwise use the horizontal-first L if its two slots are open. Either way, append slot, corner cell, slot, target.
   - Otherwise the step is blocked. Robots start from guide cell k, one per available cell move. Each robot walks like BacktrackingSolver (non-reversing random walk) until it arrives at guide cell k+1. The shortest robot walk is appended to the solution, and following resumes from guide cell k+1.
4. Raw output (prune = false): the concatenated walk, including the robots' backtracks, so it can be very long (goldens: a 1059-square raw walk on a 6x6 maze whose route is 27 squares). It starts at the start's entry square (the start cell itself if inner) and ends at the last guide cell. Pruning gives a valid simple path.
5. **`turn` has no observable effect in the original.** All 48 golden `Chain` / `Chain-left` case pairs with the same seed and maze give identical output. Under `cr` the port may use `turn` as documented (for example, robots that wall-follow with the right or left hand, or that try directions in clockwise vs counter-clockwise order). It may also ignore it. Either is conforming as long as the invariants hold.
6. Robustness requirement for the port (the original can hang): if guide cell k+1 is not reachable from guide cell k (it is a wall, or it is cut off), the port must not loop forever. It must still produce a route to the end whenever one exists, for example by skipping to a later reachable guide cell or by falling back to a direct search to the end. When the end is unreachable it returns `[]`.

## Invariants (cr)
- Both turns, with prune: `nonempty_when_reachable`, `paths_valid`.
- Both turns, without prune: `nonempty_when_reachable`.

## Errors
As in `_common.md`. An unknown `turn` value is not an error.

## Edge cases
| Input | Expected |
|---|---|
| `turn` = `'left'` | accepted |
| `turn` = anything else (including `'right'`, `'bogus'`) | behaves as `'right'` |
| start and end in the same row or column | guide line is straight |
| guide line fully open | solution is the guide line with its connecting slots, with no robots |

## Worked examples (goldens)
Grid G1 is the 7x7 `grid` field of case `solve/BacktrackingSolver/Prims3x3/oo/p/s0` in the goldens' `cases/solve.jsonl` (a 3x3-cell Prims maze; every Prims3x3 solve and transmute case uses it). In words: row 1 is open from column 1 to 5; column 1 is open from row 1 to row 5; row 5 is open from column 1 to 5; column 5 is open from row 3 to row 5; and (3,2), (3,3) form a stub off column 1. Everything else is wall.
- `solve/Chain/Prims3x3/oo/np/s0` and `solve/Chain-left/Prims3x3/oo/np/s0`: start (3,0), end (1,6). Both raw outputs are one solution R1: the 7 squares going up column 1 from (3,1) to (1,1) and then along row 1 to (1,5). The guide line runs (3,1), (1,3), (1,5). The first, diagonal step uses the vertical-first L: it goes up two rows through the slot to the corner cell (1,1), then right two columns to (1,3). The second step is straight, through one slot to (1,5). There is no backtracking, so pruning changes nothing.
- `solve/Chain/Prims3x3/ii/np/s0`: start (5,3), end (3,1). Raw `[[ (5,3), (5,2), (5,1), (4,1), (3,1) ]]`, pruned `[[ (5,2), (5,1), (4,1) ]]`.
- `solve/Chain/Prims6x6/oo/p/s0`: start (11,12), end (7,0). Pruned solution length 27. Raw length 1059.

## Open questions
None yet.
