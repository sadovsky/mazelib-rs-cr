# Spec: maze (the Maze facade)

Provenance: derived-from-source (cleanroom spec-writer), cross-checked against the shared goldens (`entrances`, `tostring`, `monte_carlo` sets) and the public docs · Audit: clean @ 2026-10-22

## Purpose
The facade object that holds one rectangular maze: a grid, an optional start and end cell, the configured generator, solver and transmuters, and the most recent solutions. It wires the algorithm modules together (generate, place entrances, solve, transmute), renders the maze as text, and offers a Monte Carlo helper that builds several mazes and picks one by difficulty.

## Public interface
Documented Python surface, language-neutral types. `cell` = `(int row, int col)`, 0-based, row 0 at the top. `grid` = 2-D array of 0/1, 1 = wall.

- `Maze(seed: optional<int> = None)`: constructor.
- `Maze.set_seed(seed: optional<int>)`: static; seeds the global RNG streams.
- Attributes (all publicly readable and assignable):
  - `grid: optional<grid>`, initially none.
  - `start: optional<cell>`, initially none.
  - `end: optional<cell>`, initially none.
  - `generator: optional<Generator>`, initially none.
  - `solver: optional<Solver>`, initially none.
  - `transmuters: list<Transmuter>`, initially empty.
  - `solutions: optional<list<list<cell>>>`, initially none.
  - `prune: bool`, initially true.
- `generate() -> none`
- `generate_entrances(start_outer: bool = True, end_outer: bool = True) -> none`
- `solve() -> none`
- `transmute() -> none`
- `tostring(entrances: bool = False, solutions: bool = False) -> string`. The protocol calls the second flag `show_solutions`.
- `generate_monte_carlo(repeat: int, entrances: int = 3, difficulty: float = 1.0, reducer: function(list<cell>) -> number = length) -> none`
- The Python object's default string conversion (printing the maze) equals `tostring(True, True)`.

The idiomatic Rust shape (an owned RNG handle, `Result` returns, builder methods) is the builder's choice. This spec only fixes observable behavior.

## Behavior

### Construction and seeding
1. The constructor sets every attribute to its initial value listed above, then calls `set_seed(seed)`.
2. `set_seed(seed)`: if `seed` is none, it does nothing. Both RNG streams keep whatever state they had, so an unseeded Maze does not reset the randomness. Otherwise it seeds the Python-`random`-compatible stream with `seed` and then the NumPy-legacy-compatible stream with the same `seed`, in that order. A seed of 0 is a real seed; only "none" skips seeding.
3. The streams are global, not per Maze. Seeding one Maze affects every later draw by any generator, solver, transmuter or facade method, and constructing a second Maze with a seed reseeds everything. Generator constructors that draw randomness (BinaryTree with no `skew`) draw at their own construction time, so constructing such an object before vs. after seeding changes what it gets. The protocol's rule (seed first, then construct) reflects this.
4. A seed outside NumPy's accepted range (negative, or above 2^32-1) fails with ValueError from the NumPy stream. At that point the Python stream has already been reseeded. This can't be observed through the protocol, because the whole case fails (goldens: seeds -7 and 4294967301 give ValueError).

### generate()
5. Requires `generator` to be set, otherwise AssertionError.
6. Replaces `grid` with a freshly generated grid from the generator, then clears `start`, `end` and `solutions` to none. It does **not** run the transmuters and does not place entrances.

### generate_entrances(start_outer, end_outer)
Let the grid have H rows and W columns (for generated mazes H = 2r+1 and W = 2c+1, both odd, at least 7). Definitions:
- An **outer** entrance is a border cell on one of the four sides, never a corner, aligned with a cell row or column: `(0, c)` or `(H-1, c)` with c odd and 1 <= c <= W-2, or `(r, 0)` or `(r, W-1)` with r odd and 1 <= r <= H-2.
- An **inner** entrance is a cell `(r, c)` with r and c both odd and not on the border, so 1 <= r <= H-2 and 1 <= c <= W-2.

7. The grid must be set. Entrances are drawn from geometry only. The method never reads cell values, so an inner entrance may land on a wall cell in grids with rooms or caves, and outer entrances always sit on border wall cells. The grid is **not** modified: border entrance cells stay walls in `grid`. Only `start` and `end` are written. `solutions` is not cleared (it may be stale afterwards).
8. Both outer (the default): pick one of the four sides uniformly at random for `start`, at a uniformly random aligned position along that side. `end` goes on the **opposite** side (north with south, west with east) at an independently chosen uniformly random aligned position. The side is chosen uniformly among the four, not weighted by side length.
9. Both inner: `start` and `end` are independent uniform inner cells. `end` is redrawn until it differs from `start`.
10. Exactly one outer: the outer one (whichever of `start`/`end` was asked to be outer) is a uniform outer entrance (uniform side, then uniform aligned position). The other is a uniform inner cell. The two are drawn independently.
11. Distance rule, applied in every mode: if the Manhattan distance between `start` and `end` is less than 2 (same cell or orthogonally adjacent), the whole placement is redone from scratch in the same mode until the rule holds. In practice only the mixed mode can trigger it, when the inner cell is the one directly inside the outer entrance: two distinct odd/odd inner cells are always at least 2 apart, and opposite outer sides are at least H-1 or W-1 apart.
12. Resulting invariants (these are what the `cr` checker verifies): each entrance is legal for its requested kind; distance(start, end) >= 2; when both are outer they lie on opposite sides (rows 0 and H-1, or columns 0 and W-1).
13. Tiny or degenerate grids (none appear in the goldens; generators never produce them): with one cell (3x3 grid), inner/inner can never find two distinct cells, and mixed mode always gets an adjacent pair. The original then never terminates (inner/inner) or overflows its retry depth (mixed). If a dimension has no odd index in range (fewer than 2 rows or columns), the original fails with ValueError from the random range draw. Ports should return an error rather than loop. The kind there isn't compared; ValueError is a reasonable choice. A missing grid makes the original fail with a non-assertion attribute error; ports should return an error.

### solve()
14. Requires `solver` to be set (AssertionError), then requires both `start` and `end` to be set (AssertionError). The solver's own validation follows: grid present, and both entrances inside the grid bounds, otherwise AssertionError (see the solver base spec).
15. `solutions` becomes the solver's list of paths. The solver works on a copy of the grid, so the facade's grid is unchanged.
16. If `prune` is true, each path is cleaned independently, the list keeps the same number of paths in the same order, and empty paths are not removed:
    - Loop erasure. While the path has more than 2 cells and some cell occurs more than once: take the earliest position whose cell occurs again later, and delete everything from that position up to (but not including) the next occurrence of the same cell, so that one copy remains. Repeat. Each pass removes at least one cell. The number of passes is also capped at the original path length, which never binds in practice. A path of 2 or fewer cells is left as is, even if it repeats a cell.
    - Endpoint trimming, only if the path then has more than one cell: if its first cell equals `start`, drop it; then, if its last cell equals `end`, drop it. A single-cell path is never trimmed.
17. If `prune` is false, the solver's raw paths are stored unchanged. Whether they include the start/end cells, revisits and so on is defined by each solver's spec.

### transmute()
18. Requires `grid` to be set (AssertionError). An empty `transmuters` list is fine and does nothing.
19. Applies each transmuter in list order, in place on `grid`, passing the current `start` and `end`, which may be none. It does not clear or recompute `solutions`, and it does not move entrances.

### tostring(entrances, solutions)
This is compared **exactly** under profile `cr`.
20. If `grid` is none, returns the empty string.
21. Builds one text line per grid row, left to right: a wall cell (any non-zero value) becomes `#`, an open cell (0) becomes a single space. Line length is W, there are H lines.
22. Entrance overlay: only when the `entrances` flag is true **and** both `start` and `end` are set (not none). Then the character at `start` becomes `S`, and after that the character at `end` becomes `E`, so `E` wins if they coincide. If either one is none, neither letter is drawn. A cell like (0, 0) counts as set; only none suppresses the overlay.
23. Solution overlay: only when the solutions flag is true **and** `solutions` is set and non-empty. Then every cell of the **first** path only (`solutions[0]`) becomes `+`, overwriting whatever was there (`#`, space, `S` or `E`). Later paths are never drawn. An empty list of solutions, or an empty first path, draws nothing.
24. The overlay order is walls/spaces, then `S`, then `E`, then `+`. Precedence: `+` over `E` over `S` over the base character.
25. Lines are joined with a single `\n` (LF). There is **no** trailing newline and no trailing spaces are trimmed. The result for an H-row grid contains exactly H-1 newline characters.
26. The two flags are independent: `entrances=false, solutions=true` draws `+` but no `S`/`E`.
27. Positions outside the grid are not covered by the goldens. The original misbehaves there (a row index past the end raises IndexError; negative or too-large columns corrupt the line). Ports may return an error (IndexError is the closest kind) or ignore such cells. Not compared.

### generate_monte_carlo(repeat, entrances, difficulty, reducer)
28. Validates first, before any generation: `difficulty` must satisfy 0.0 <= difficulty <= 1.0, otherwise AssertionError. NaN also fails the check. Golden `mc/invalid-difficulty` (difficulty 1.5) expects AssertionError.
29. What it does, in terms of intent:
    - It builds `repeat` mazes. Each is made with `generate()`, which requires a generator (AssertionError) and clears entrances/solutions. Transmuters are **not** applied.
    - For each maze it tries `entrances` placements. Each one is `generate_entrances()` with the defaults (both outer, opposite sides) followed by `solve()`, which requires a solver (AssertionError) and honors `prune`. Each attempt is recorded as a candidate (grid, start, end, solutions).
    - It keeps one candidate per maze: the one with the **most solution paths** (the number of paths in `solutions`). On a tie, the earliest attempt wins. The docs describe this step as keeping the entrances that give the longest solution. The original actually ranks by number of paths, which for single-solution solvers is always a tie, so the first attempt wins. Under `cr` this choice isn't observable beyond the invariants below. Ranking by first-path length (the documented intent) is an acceptable design.
    - It orders the kept mazes ascending by `reducer(first path)`, which by default is the length of `solutions[0]`. The sort is stable, so mazes with equal scores keep their generation order.
    - It picks the element at index floor((n - 1) * difficulty) in that order, where n is the number of mazes: 0.0 picks the easiest, 1.0 the hardest, 0.5 the lower median (for n = 3 that is index 1).
    - It sets `grid`, `start`, `end` and `solutions` to the chosen candidate's values. They are left set, not none.
30. Other error conditions in the original (none of them in the goldens): `repeat` < 1 fails with IndexError (nothing to pick); `entrances` < 1 fails with ValueError (no candidate per maze); a kept candidate with an empty solution list fails with IndexError while ranking. Ports should reject `repeat` < 1 and `entrances` < 1 as invalid input. For robustness they should prefer candidates that have at least one non-empty solution, because the `cr` checker requires a non-empty `solutions` with a valid first path.
31. Invariants checked under `cr` for each successful `monte_carlo` case: the grid has the generator's shape and alphabet and satisfies the generator's discovered properties (for Prims: perfect maze, etc.); `start`/`end` are legal outer entrances on opposite sides with distance >= 2; `solutions` is non-empty; and the first path, if non-empty, consists of open cells and forms a unit-step chain from `start` through the path to `end` (start and end excluded from the path, which is what pruning produces).

## Invariants
- After `generate()`: `grid` is set; `start`, `end`, `solutions` are none.
- After `generate_entrances(a, b)`: see Behavior item 12. Grid unchanged.
- After `solve()` with prune: every path longer than 2 cells is free of repeated cells; a path with 2 or more cells after loop erasure never begins with `start` or ends with `end`.
- `tostring(false, false)` depends only on the grid. Its length is H*W + (H-1). Every character is `#` or space.
- `tostring` never changes any attribute.
- After `generate_monte_carlo`: grid, start, end and solutions are all set. Start and end are outer entrances on opposite sides.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| `generate()` with no generator | AssertionError | no |
| `solve()` with no solver | AssertionError | no |
| `solve()` with start or end unset | AssertionError | no |
| `solve()` with no grid, or start/end outside the grid | AssertionError (solver base) | no |
| `transmute()` with no grid | AssertionError | no |
| `generate_monte_carlo` with difficulty < 0, > 1, or NaN | AssertionError | no |
| `set_seed` / constructor seed outside 0..2^32-1 | ValueError (NumPy stream) | no |
| `generate_entrances` with no grid | original: attribute error (not compared); port: any error | no |
| `generate_entrances` on a grid too small for legal entrances | original: hangs / recursion overflow / ValueError; port: return an error | no |
| `generate_monte_carlo` with repeat < 1 or entrances < 1 | original: IndexError / ValueError; port: an invalid-input error | no |

Messages are never compared.

## Edge cases
| Input | Expected |
|---|---|
| `Maze()` with no seed | streams not reseeded; all attributes initial |
| `Maze(0)` | both streams seeded with 0 |
| `tostring` with grid none | `""` |
| `tostring(true, …)` with start set but end none (or vice versa) | no `S`/`E` drawn |
| `tostring(…, true)` with solutions none or `[]` | no `+` drawn |
| `tostring(true, true)` where the first path touches start/end | `+` overwrites the letter |
| `tostring` with several solutions | only the first path is drawn |
| `solve()` with prune on a 1-cell path equal to start | path kept as is |
| `generate_entrances(false, true)` | `start` inner, `end` outer |
| `generate_monte_carlo(…, difficulty=1.5)` | AssertionError before any generation |
| `generate_monte_carlo(3, 3, 0.5)` | the middle (index 1) of the 3 mazes sorted by first-path length |

## Worked examples
All examples are from `../slopfork-mazelib-goldens`. Grids in the protocol use `#` = wall and `.` = open. Rendered text uses `#` and a space. Below, `\n` is a single LF and rendered rows are listed one per bullet. Spaces matter.

### tostring (exact, profile cr)
Grid G1 (3x3 cells), rows: `#######`, `#.....#`, `#.#####`, `#...#.#`, `#.###.#`, `#.....#`, `#######`.

Case `str/Prims3x3/oo/*`: start (3,0), end (1,6), solutions = one path of 7 cells: up column 1 from row 3 to row 1 (rows 3, 2, 1), then right along row 1 through columns 2 to 5 (as protocol JSON it is the golden's `solutions` field verbatim).
- `e0s0` (entrances false, show_solutions false): `"#######\n#     #\n# #####\n#   # #\n# ### #\n#     #\n#######"`
- `e0s1` (false, true): `"#######\n#+++++#\n#+#####\n#+  # #\n# ### #\n#     #\n#######"`. There are no S/E, but the path is drawn.
- `e1s0` (true, false): `"#######\n#     E\n# #####\nS   # #\n# ### #\n#     #\n#######"`. S and E replace border walls.
- `e1s1` (true, true): `"#######\n#+++++E\n#+#####\nS+  # #\n# ### #\n#     #\n#######"`
- `no-entrances` (true, true, start/end/solutions all null): same as `e0s0`.

Case `str/Prims3x3/ii/*`: start (5,3), end (3,1), solutions `[]` (an empty list).
- `e0s0` and `e0s1`: same as G1 plain (an empty solutions list draws nothing).
- `e1s0` and `e1s1`: `"#######\n#     #\n# #####\n#E  # #\n# ### #\n#  S  #\n#######"`

Grid G2 (6x6 cells), rows: `#############`, `#...........#`, `#.###.#####.#`, `#.#.....#...#`, `###.#####.#.#`, `#.....#...#.#`, `#.#.###.###.#`, `#.#...#...#.#`, `#.#########.#`, `#.........#.#`, `#.#####.###.#`, `#.#.......#.#`, `#############`.

Case `str/Prims6x6/oo/e1s1`: start (11,12), end (7,0), one path: (11,11),(10,11),(9,11),(8,11),(7,11),(6,11),(5,11),(4,11),(3,11),(2,11),(1,11),(1,10),(1,9),(1,8),(1,7),(1,6),(1,5),(2,5),(3,5),(3,4),(3,3),(4,3),(5,3),(5,2),(5,1),(6,1),(7,1). Rendered rows:
- `#############`
- `#    +++++++#`
- `# ###+#####+#`
- `# #+++  #  +#`
- `###+##### #+#`
- `#+++  #   #+#`
- `#+# ### ###+#`
- `E+#   #   #+#`
- `# #########+#`
- `#         #+#`
- `# ##### ###+#`
- `# #       #+S`
- `#############`

Case `str/Prims6x6/ii/e1s0`: start (7,11), end (7,9), solutions `[]`. Row 7 renders as `# #   #  E#S#`, and all other rows are the plain G2 rendering.

### entrances (invariant under cr; exact values are the original's draws)
Grid G1 (7x7):
- `ent/Prims/h3w3/oo/s0` (outer/outer): start (3,6), end (1,0). East and west sides, odd rows.
- `ent/Prims/h3w3/oo/s2`: start (0,1), end (6,1). North and south.
- `ent/Prims/h3w3/oi/s0` (start outer, end inner): start (3,6), end (1,3).
- `ent/Prims/h3w3/oi/s3`: start (6,5), end (5,1).

7x11 grid (3x5 cells):
- `ent/Prims/h3w5/io/s2` (start inner, end outer): start (1,5), end (0,1).
- `ent/Prims/h3w5/ii/s4` (inner/inner): start (1,5), end (1,7). Distance 2 is allowed.

### monte_carlo (invariant under cr)
- `mc/Prims/ShortestPath/d0.0/s0`: generator Prims 4x4, solver ShortestPath, repeat 3, entrances 3, difficulty 0.0. Result: start (3,0), end (5,8), one path of 9 cells: (3,1),(3,2),(3,3),(3,4),(3,5),(4,5),(5,5),(5,6),(5,7).
- For the same generator/solver, the first-path lengths across difficulties 0.0 / 0.5 / 1.0 are 9 / 11 / 13 at seed 0 and 7 / 13 / 13 at seed 2. They are non-decreasing with difficulty for a fixed seed, as the sort-then-index rule implies.
- `mc/invalid-difficulty`: Prims 3x3, ShortestPath, repeat 1, entrances 1, difficulty 1.5 gives an AssertionError.

## Open questions / dirty-side notes on rng.md
- rng.md item 14 is consistent with the facade: seeding happens only for a non-none seed, the Python stream is seeded before the NumPy stream, and both get the same integer. With seed none, neither stream is touched. The library never seeds per object; there is only the global pair of streams.
- The library itself only draws from the Python stream for uniform ranged integers (`randrange`-style), floats, and choices. The NumPy stream is used for in-place shuffles of short neighbor lists. None of this is compared under `cr`; only the `rng` op is exact.
