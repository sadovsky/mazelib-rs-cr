# Spec: transmute/_common (shared transmuter contract)

Provenance: documented (docs/MAZE_TRANSMUTE_ALGOS.md, docs/API.md "Optional: Transmuting a Maze", goldens PROTOCOL.md) + observed (goldens `transmute` set, `invariants.json`) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
A transmuter edits an existing maze grid **in place** and leaves it solvable. The docs put it this way: many classic solving methods amount to simplifying the maze and then solving it. There are three transmuters. CuldeSacFiller breaks loops that hang off the maze. DeadEndFiller seals dead ends. Perturbation adds a few random walls and then reconnects the maze. This file fixes the shared contract. The per-algorithm files add the specifics.

## Public interface
- `CuldeSacFiller()`, `DeadEndFiller(iterations: int = 1)`, `Perturbation(repeat: int = 1, new_walls: int = 1)`.
- `transmute(grid, start: optional<cell>, end: optional<cell>) -> none`. It mutates `grid`.
- Facade: `Maze.transmuters: list<Transmuter>` and `Maze.transmute()`. The facade applies each transmuter in list order to `Maze.grid`, passing the current `Maze.start` and `Maze.end` (either may be none). It does not clear `solutions` and does not move the entrances. See `maze.md` items 18 and 19.
- Adapter protocol: op `transmute`, `args = {algo, params, grid, start, end}`, result `{grid}` (the grid after transmutation). In the protocol, start and end are always given.

## Vocabulary
The same as `solve/_common.md`: H x W grid, 1 = wall. Cells are odd/odd positions. Wall slots have one odd and one even coordinate and lie between two orthogonally adjacent cells. Posts are even/even. The border is the outer frame. A cell's **neighbours** are the cells two positions away in the four directions such that the slot in between and the neighbour cell are both open (a "cell move" from `solve/_common.md`). A cell's **degree** is its number of neighbours.
- **Dead end**: an open cell with degree 0 or 1.
- **Junction**: an open cell with degree 3 or 4.
- **Corridor cell**: an open cell with degree exactly 2.
- Outer entrances (start/end on the border) are wall squares in generated grids. Degree counting never looks at the border, so an outer entrance never counts as a neighbour.

## Behavior
1. Transmuters work in place on the caller's grid and return nothing. The grid's shape never changes.
2. **Border**: no transmuter changes any border square. (`border_unchanged` holds on every golden case. Ports must guarantee it for every input.)
3. **Entrances**: no transmuter changes the value at `start` or `end`, and the transmuter never moves them. DeadEndFiller and CuldeSacFiller additionally never seal the cells at or right next to the entrances (see their specs).
4. **Solvability**: if a route from `start` to `end` existed before (unit steps through open squares, endpoints allowed to be walls, as in `solve/_common.md`), one exists afterwards (`keeps_route`).
5. Which squares each one may change:
   - CuldeSacFiller: only cells, wall-ward (open to wall).
   - DeadEndFiller: only cells and the slots around them, wall-ward. Border squares are never changed.
   - Perturbation: only interior wall slots, in both directions. Cells and posts are never changed.
6. Randomness: CuldeSacFiller and DeadEndFiller are deterministic. Their output depends only on the grid, start, end and parameters, so a port that follows their specs reproduces the golden grids exactly. Perturbation is random. It must draw only from the library's seeded RNG streams (deterministic per seed). Under `cr`, how it uses them is the port's own design.
7. Validation: the facade requires a grid (`Maze.transmute()` with no grid raises AssertionError). The transmuters themselves validate nothing. With start or end none, the original's DeadEndFiller fails with TypeError, while the other two run normally (they then protect nothing). For a port: when start/end are absent, CuldeSacFiller and Perturbation proceed. DeadEndFiller may either proceed with no entrance protection or return an error mapped to TypeError; the recommendation is the TypeError mapping, for parity. Start/end outside the grid are not covered by the goldens. Ports should return an error rather than panic.

## Invariants (cr, from `invariants.json`)
Checked by `checkers/invariants.py`, comparing the grid before and after:
- `shape`: same dimensions.
- `border_unchanged`: every border square is identical.
- `only_adds_walls`: no square goes from wall to open.
- `keeps_route`: reachable before implies reachable after.
- `keeps_shortest_length`: if reachable before, the start-to-end distance is identical after.
- `perfect_if_input_perfect`: if the input grid is a perfect maze (all open squares connected and acyclic, as defined in `generate/_common.md`), the output is too.

| Key | Required |
|---|---|
| CuldeSacFiller | border_unchanged, keeps_route, keeps_shortest_length, only_adds_walls, perfect_if_input_perfect, shape |
| DeadEndFiller (default, and `{"iterations": 3}`) | border_unchanged, keeps_route, keeps_shortest_length, only_adds_walls, perfect_if_input_perfect, shape |
| Perturbation (default) | border_unchanged, keeps_route, perfect_if_input_perfect, shape |
| Perturbation `{"new_walls": 2, "repeat": 3}` | border_unchanged, keeps_route, shape (ports may also keep perfection, which is recommended) |

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| `Maze.transmute()` with no grid | AssertionError | no |
| DeadEndFiller with start or end absent | TypeError (recommended mapping) | no |

The goldens' `transmute` set (120 cases) has no error cases and no known bugs.

## Golden inputs
Every transmute case uses one of six grids (Prims 3x3, 6x6 and 10x10, DungeonRooms 6x6 and 10x10, CellularAutomaton 6x6). The three Prims grids and the DungeonRooms 6x6 grid are perfect mazes. DungeonRooms 10x10 (open rooms) and CellularAutomaton 6x6 (loops) are not. Each comes with outer/outer (`oo`) and inner/inner (`ii`) entrances and seeds 0 and 1. In all of them the border is solid wall and every cell is open.

## Open questions
None yet.
