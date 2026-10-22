# Feature inventory and parity matrix

Source: john-science/mazelib@56cd88294a36746bbd324c2c9f7e4ab26095deb3 (mazelib 0.9.17, MIT) · Target: Rust (idiomatic, zero-dep lib crate `mazelib` + `mazelib-adapter` bin) · Mode: cleanroom · Profile: `cr`

Status: `todo` → `spec` → `building` → `verified@Lx` (or `deviates` with a note).

Public surface lists documented class names and constructor parameters only (with documented defaults / value domains). Generators all take the maze size in **cells** as their first two positional arguments, rows first then columns, and produce a grid of `(2*rows+1) x (2*cols+1)`. Some classes name these two parameters `(h, w)` and some `(w, h)`, but positionally the first one always means rows. Solvers and transmuters take no size; they act on the facade's grid/start/end.

| # | Module / feature | Public surface | Depends on | Level | Status | Level reached | Deviations |
|---|---|---|---|---|---|---|---|
| 1 | RNG layer (Python `random` + NumPy legacy `RandomState` compatibility) | Seeding of both global streams from one integer seed. Python-stream draws used by the library: `random()`, `randrange`, `randint`, `choice`, `shuffle`. NumPy-stream draw used by the library: in-place `shuffle` of a sequence. Exercised directly by the `rng` op in the goldens | — | L2 | todo | — | |
| 2 | Grid type | 2-D row-major array of cells, 1 = wall, 0 = passage. Positions are `(row, col)` | — | L2 | todo | — | |
| 3 | Generator base | Shared constructor contract: size in cells (rows, cols), both must be at least 3 (otherwise AssertionError); output grid dimensions `(2r+1, 2c+1)` | 1, 2 | L2 | todo | — | |
| 4 | Solver base | Shared contract: needs a grid plus start and end set; start/end must lie inside the grid bounds (AssertionError otherwise); returns a list of solutions, each a list of `(row, col)` positions | 1, 2 | L2 | todo | — | |
| 5 | Transmuter base | Shared contract: acts in place on the facade's grid (and start/end when they are set), leaving the maze solvable | 1, 2 | L2 | todo | — | |
| 6 | AldousBroder | Size args only (rows, cols) | 3 | L2 | todo | — | |
| 7 | BacktrackingGenerator | Size args only (rows, cols) | 3 | L2 | todo | — | |
| 8 | BinaryTree | Size args (rows, cols); optional `skew`, one of `'NW'`, `'NE'`, `'SE'`, `'SW'` (docs say default `'NW'`; observed: when omitted or unrecognized, one of the four is picked at random from the Python stream at construction) | 3 | L2 | todo | — | |
| 9 | CellularAutomaton | Size args (rows, cols); float params `complexity` and `density`, each default 1.0 (docs give range [0,1]; goldens also use complexity 2.0) | 3 | L2 | todo | — | |
| 10 | Division | Size args only (rows, cols); recursive division | 3 | L2 | todo | — | |
| 11 | DungeonRooms | Size args (rows, cols); optional `rooms` (default none): a list of rooms, each given as two grid coordinates (upper-left corner, lower-right corner); optional `grid` (default none): a starting grid; `hunt_order` `'random'` (default) or `'serpentine'`. Invalid input grid raises AssertionError | 3 | L2 | todo | — | |
| 12 | Ellers | Size args (rows, cols); float params `xskew` and `yskew`, each default 0.5, range [0,1]. Original overflows (OverflowError) on some valid inputs; see goldens KNOWN_BUGS | 3 | L2 | todo | — | |
| 13 | GrowingTree | Size args (rows, cols); float `backtrack_chance`, default 1.0, range [0,1] | 3 | L2 | todo | — | |
| 14 | HuntAndKill | Size args (rows, cols); `hunt_order` `'random'` (default) or `'serpentine'` | 3 | L2 | todo | — | |
| 15 | Kruskal | Size args only (rows, cols) | 3 | L2 | todo | — | |
| 16 | Prims | Size args only (rows, cols) | 3 | L2 | todo | — | |
| 17 | Sidewinder | Size args (rows, cols); float `skew`, default 0.5, range [0,1] | 3 | L2 | todo | — | |
| 18 | TrivialMaze | Size args (rows, cols); `maze_type` `'spiral'` (default) or `'serpentine'` | 3 | L2 | todo | — | |
| 19 | Wilsons | Size args (rows, cols); `hunt_order` `'random'` (default) or `'serpentine'` | 3 | L2 | todo | — | |
| 20 | BacktrackingSolver | No constructor params | 4 | L2 | todo | — | |
| 21 | Chain | `turn` `'right'` (default) or `'left'` | 4 | L2 | todo | — | |
| 22 | Collision | No constructor params. Original raises TypeError on mazes with loops/rooms; see KNOWN_BUGS | 4 | L2 | todo | — | |
| 23 | RandomMouse | No constructor params | 4 | L2 | todo | — | |
| 24 | ShortestPath | No constructor params. Raises AssertionError when the maze gives no usable start; original sometimes returns no solution where one exists (recorded as behavior) | 4 | L2 | todo | — | |
| 25 | ShortestPaths | No constructor params. Raises AssertionError when no usable start or no solution is found | 4 | L2 | todo | — | |
| 26 | Tremaux | No constructor params | 4 | L2 | todo | — | |
| 27 | CuldeSacFiller | No constructor params | 5 | L2 | todo | — | |
| 28 | DeadEndFiller | Int `iterations`, default 1, positive | 5 | L2 | todo | — | |
| 29 | Perturbation | Ints `repeat` and `new_walls`, each default 1, positive | 5 | L2 | todo | — | |
| 30 | Maze facade | `Maze(seed=None)`; static `set_seed(seed)`; attributes `grid`, `start`, `end`, `generator`, `solver`, `solutions`, `transmuters` (list), `prune` (bool, default true); methods `generate()`, `generate_entrances(start_outer=True, end_outer=True)`, `solve()`, `transmute()`, `tostring` (two boolean flags, `entrances` and `solutions`, both default false), and `generate_monte_carlo` (positional `repeat`; optional `entrances` default 3, `difficulty` float default 1.0 in [0,1], `reducer` default = solution length) | 1-29 | L2 | todo | — | |

## Cross-cutting decisions
| Concern | Source behavior | Target strategy |
|---|---|---|
| Randomness / RNG | Two global streams: Python's `random` module (most draws: floats, ranged ints, choices) and NumPy's legacy global `RandomState` (in-place shuffles). Seeding the facade (constructor seed or `set_seed`) seeds both streams with the same integer; with no seed, nothing is reseeded. No per-instance RNG objects. | Rust, zero-dep lib crate. Bit-compatible MT19937 for both streams: one reproducing CPython `random` (seeding from int, `random()`, `getrandbits`-based ranged ints, `choice`, `shuffle`) and one reproducing NumPy legacy `RandomState` (seeding, shuffle). Exact match required only for the `rng` op; how algorithms consume the streams is the builder's own design. |
| Numeric types | Grids are 2-D signed 8-bit arrays, 1 = wall, 0 = passage; coordinates are integer `(row, col)` tuples; skews/chances/difficulty are floats; solutions are lists of coordinate lists. | `Grid` struct over `Vec<u8>`, row-major, with rows/cols; `(usize, usize)` positions; `f64` params. Avoid fixed-width overflow (the source's 8-bit overflow is a known bug, ports must not crash). |
| Errors | Invalid input raises Python `AssertionError` (maze size below 3x3, no generator/solver set, no grid to transmute, entrances unset or outside the grid, Monte Carlo difficulty outside [0,1], invalid DungeonRooms grid, no solution found for ShortestPaths). `ValueError` surfaces from the RNG layer on invalid ranges / empty sequences, and on one invalid DungeonRooms input. `TypeError` (Collision) and `OverflowError` (Ellers) are crashes on valid input listed in KNOWN_BUGS: ports must return a valid result there instead. | Typed error enum in the lib (`Result` for invalid input, no panics on valid input); adapter maps variants to the protocol's `error.kind` strings (`AssertionError`, `ValueError`, ...). |
| I/O, time, concurrency | None. Pure in-memory, single-threaded; `tostring` renders the grid as text (wall/passage/entrance/solution characters, one line per row). | Pure library, no I/O; adapter does JSON lines on stdin/stdout per goldens PROTOCOL.md. |

## Source tests (L1)
| Source test | Target test | Status |
|---|---|---|
| Generator tests (source repo test suite) | — | not targeted (L2 goal; goldens are the oracle) |
| Solver tests | — | not targeted |
| Transmuter tests | — | not targeted |
| Maze facade tests | — | not targeted |
