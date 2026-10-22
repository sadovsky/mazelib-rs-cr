# Spec: generate/_common (shared generator contract)

Provenance: documented (docs/MAZE_GEN_ALGOS.md, docs/API.md, goldens PROTOCOL.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Every maze generator takes a maze size measured in cells and returns a fresh grid. This file fixes what all fourteen generators share: the size contract, the grid encoding, the validation order and the error kinds, the meaning of "perfect", and how a generator may use the two RNG streams. Each per-algorithm spec in this directory only adds what is specific to that algorithm.

## Public interface
- Every generator is constructed as `Algo(h, w, ...options)`. The first positional argument is the number of cell rows (`h`), the second the number of cell columns (`w`). Some documented signatures spell the pair as `(w, h)`; that is cosmetic. Positionally the first argument is always rows. The goldens confirm this: case `h3w5` yields 7 rows of 11 characters.
- `generate() -> grid`, which returns a new grid each time it is called. A generator object may be asked to generate more than once (the Monte Carlo facade does this), and every call must produce a valid maze on its own.
- Options are keyword parameters with defaults; see each algorithm.
- Adapter protocol: op `generate`, `args = {algo, h, w, params}`, result `{grid: list<string>}`.

## Grid encoding
1. The grid has `2h+1` rows and `2w+1` columns, stored row-major. Value 1 is wall and value 0 is passage (protocol strings use `#` for wall and `.` for open). Row 0 is the top.
2. Cell `(i, j)`, with `0 <= i < h` and `0 <= j < w`, sits at grid position `(2i+1, 2j+1)`. Positions where both coordinates are odd are cells.
3. A position with one odd and one even coordinate is a wall slot between two orthogonally adjacent cells. Opening it joins the two cells.
4. Positions where both coordinates are even are posts (corners). In a perfect maze every post is a wall.
5. The outer frame (row 0, row 2h, column 0, column 2w) is all wall in every generator's output. Entrances are added later by the facade, never by a generator.

## Invariant vocabulary (as computed by the goldens checker)
The names below are the property names in `invariants.json`. Each algorithm spec lists which ones it must satisfy.
- `shape`: the grid has exactly `2h+1` rows, each with exactly `2w+1` columns.
- `alphabet`: only wall/open values appear.
- `border_walls`: the outer frame is all wall.
- `corners_walls`: every even/even position is wall.
- `cells_open`: every odd/odd position is open.
- `connected`: all open squares form one 4-connected component.
- `acyclic`: the number of open-square adjacencies (horizontal plus vertical pairs of open squares) equals the number of open squares minus the number of components.
- `perfect`: connected and acyclic. With cells_open and corners_walls this means the open wall slots form a spanning tree over the `h*w` cells, so exactly `h*w - 1` wall slots are open.

## Validation and error kinds (compared exactly, profile cr)
Validation runs before any random draw and before any other work.

| Condition | Error kind | Generators |
|---|---|---|
| `h < 3` or `w < 3` (includes 0 and negatives) | AssertionError | all except DungeonRooms |
| No `grid` option and `h < 0` or `w < 0`, i.e. `2h+1` or `2w+1` would be negative. This check comes first, so `(h=-1, w=4)` is a ValueError and not an AssertionError. | ValueError | DungeonRooms only |
| No `grid` option, `h` and `w` both at least 0, and either below 3 (e.g. `(0,3)`, `(1,1)`, `(2,5)`) | AssertionError | DungeonRooms |
| A `grid` option is given and the size derived from it (`(rows-1) div 2`, `(cols-1) div 2`) is below 3 | AssertionError | DungeonRooms |
| After rooms are carved, no cell is still a wall, so there is nowhere for the hunt to start (see DungeonRooms.md) | AssertionError (raised from `generate`) | DungeonRooms |

The goldens' `invalid` cases are `h1w1`, `h2w2`, `h2w5`, `h5w2`, `h0w3` and `h-1w4`, run for every generator. All of them give AssertionError, except DungeonRooms `h-1w4`, which gives ValueError. Messages are informational only. The original's size message is "Mazes cannot be smaller than 3x3." and its negative-dimension message is "negative dimensions are not allowed".

In the Rust library these are typed errors, e.g. a "size too small" variant and an "invalid dimensions" variant. The adapter maps them to the kind strings above.

## Option parsing shared by several generators
- String options (`hunt_order`, `maze_type`) are matched case-insensitively after trimming surrounding whitespace. Any string other than the non-default value (`serpentine` in both cases) selects the default. So `"Serpentine "` means serpentine, while `"bogus"` and `""` mean the default.
- Float options are not validated. Values outside the documented range never raise. Their effect is described per algorithm: clamped for Ellers, and for the other float options a probability compare that saturates.

## RNG usage (profile cr)
- The adapter seeds both streams (the Python-random-compatible one and the NumPy-legacy-compatible one) before the generator is constructed. Construction itself may draw (BinaryTree with no recognized skew does).
- Under profile cr no generator output is compared exactly. A port may draw from either stream, in any order and any amount, as long as the result satisfies the listed invariants and follows the stated distributional intent. Determinism for a given seed is expected: the same seed and inputs give the same grid within the port.
- The randomized hunt in HuntAndKill, Wilsons and DungeonRooms is probabilistic in the original, so in principle it can miss cells. A port must make coverage certain, for example by falling back to an exhaustive scan once the random budget runs out.

## Known original crashes (KNOWN_BUGS.json)
- `gen/Ellers/v2/h12w17/s0` through `s7` (Ellers, xskew 0.9, yskew 0.1, 12x17): the original raises OverflowError because a set label overflowed a signed 8-bit store. The port must return a valid perfect maze for these cases, and the checker marks any error there as a failure. More generally, no generator may fail on valid input for any size, so labels and counters must be wide enough.

## Open questions
None yet.
