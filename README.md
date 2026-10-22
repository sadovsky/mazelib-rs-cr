# mazelib-rs-cr

An independent clean-room implementation of the [mazelib](https://github.com/john-science/mazelib) API in Rust. Not affiliated with mazelib (see `NOTICE`).

It was built with the [slopfork](https://github.com/sadovsky/slopfork) process in **cleanroom** mode. A spec-writer agent read the original and wrote behavioral specs (`.slopfork/spec/`, leak-audited). Builder agents then implemented the crate from those specs and from the shared [goldens](https://github.com/sadovsky/slopfork-mazelib-goldens), without ever seeing the original source. `.slopfork/REPORT.md` has the results.

- `mazelib` is the library crate: stable Rust 2021, `#![forbid(unsafe_code)]`, **zero dependencies**.
- `mazelib-adapter` is a JSON-lines adapter for the goldens protocol, built with `cargo run --release --bin mazelib-adapter`.

## Example

```rust
use mazelib::{generate::Prims, solve::ShortestPath, Maze};

let mut m = Maze::with_seed(42)?;
m.generator = Some(Box::new(Prims::new(10, 10)?));
m.generate()?;
m.generate_entrances(true, true)?;
m.solver = Some(Box::new(ShortestPath::new()));
m.solve()?;
println!("{}", m.tostring(true, true));
# Ok::<(), mazelib::MazeError>(())
```

## Python → Rust mapping

| Python (mazelib) | Rust (`mazelib`) |
|---|---|
| `Maze()` | `Maze::new()` |
| `Maze(seed)` / `Maze.set_seed(seed)` | `Maze::with_seed(seed)?` / `maze.set_seed(seed)?` (seeds `maze.rng`, not global state) |
| `m.grid` (NumPy int8 array) | `m.grid: Option<Grid>` (`Grid` is row-major `Vec<u8>`, 1 = wall, 0 = open) |
| `m.start`, `m.end` | `m.start`, `m.end: Option<(usize, usize)>` |
| `m.generator = Prims(h, w)` | `m.generator = Some(Box::new(Prims::new(h, w)?))` |
| `m.solver = ShortestPath()` | `m.solver = Some(Box::new(ShortestPath::new()))` |
| `m.transmuters = [DeadEndFiller(3)]` | `m.transmuters = vec![Box::new(DeadEndFiller::new(3))]` |
| `m.solutions` | `m.solutions: Option<Vec<Path>>` |
| `m.prune` | `m.prune: bool` |
| `m.generate()` | `m.generate()?` |
| `m.generate_entrances(start_outer, end_outer)` | `m.generate_entrances(start_outer, end_outer)?` |
| `m.solve()` | `m.solve()?` |
| `m.transmute()` | `m.transmute()?` |
| `m.tostring(entrances, solutions)` / `str(m)` | `m.tostring(entrances, show_solutions)` / `Display` |
| `m.generate_monte_carlo(repeat, entrances, difficulty)` | `m.generate_monte_carlo(repeat, entrances, difficulty)?` (also `generate_monte_carlo_with` for a custom reducer) |
| `AldousBroder(h, w)` | `generate::AldousBroder::new(h, w)?` |
| `BacktrackingGenerator(h, w)` | `generate::BacktrackingGenerator::new(h, w)?` |
| `BinaryTree(h, w, skew=None)` | `generate::BinaryTree::new(h, w, Skew::parse("NW"))?` (`None` = random) |
| `CellularAutomaton(h, w, complexity=1.0, density=1.0)` | `generate::CellularAutomaton::new(h, w, complexity, density)?` |
| `Division(h, w)` | `generate::Division::new(h, w)?` |
| `DungeonRooms(h, w, rooms=None, grid=None, hunt_order='random')` | `generate::DungeonRooms::new(h, w, rooms, grid, HuntOrder::parse(..))?` |
| `Ellers(h, w, xskew=0.5, yskew=0.5)` | `generate::Ellers::new(h, w, xskew, yskew)?` |
| `GrowingTree(h, w, backtrack_chance=1.0)` | `generate::GrowingTree::new(h, w, backtrack_chance)?` |
| `HuntAndKill(h, w, hunt_order='random')` | `generate::HuntAndKill::new(h, w, HuntOrder::parse(..))?` |
| `Kruskal(h, w)` | `generate::Kruskal::new(h, w)?` |
| `Prims(h, w)` | `generate::Prims::new(h, w)?` |
| `Sidewinder(h, w, skew=0.5)` | `generate::Sidewinder::new(h, w, skew)?` |
| `TrivialMaze(h, w, maze_type='spiral')` | `generate::TrivialMaze::new(h, w, MazeType::parse(..))?` |
| `Wilsons(h, w, hunt_order='random')` | `generate::Wilsons::new(h, w, HuntOrder::parse(..))?` |
| `BacktrackingSolver()`, `Collision()`, `RandomMouse()`, `ShortestPath()`, `ShortestPaths()`, `Tremaux()` | `solve::<Name>::new()` |
| `Chain(turn='right')` | `solve::Chain::new(Turn::parse(..))` |
| `CuldeSacFiller()` | `transmute::CuldeSacFiller::new()` |
| `DeadEndFiller(iterations=1)` | `transmute::DeadEndFiller::new(iterations)` |
| `Perturbation(repeat=1, new_walls=1)` | `transmute::Perturbation::new(repeat, new_walls)` |
| global `random` / `numpy.random` | `Rng { py: PyRandom, np: NpRandom }`, bit-compatible MT19937 streams |
| `AssertionError` / `ValueError` / `IndexError` | `MazeError::InvalidInput` / `InvalidValue` / `Empty` |

Generators, solvers and transmuters implement the `Generator`, `Solver` and `Transmuter` traits, and take an explicit `&mut Rng`. `Solver::solve` returns raw paths, which `Maze::solve` prunes with `solve::prune_solutions` when `prune` is set.

## Checks

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Golden parity, profile `cr`, for each case set:

```
H=.claude/skills/slopfork/scripts/diff_harness.py
G=../slopfork-mazelib-goldens
A=target/release/mazelib-adapter
python3 $H compare --batch --profile cr --cases $G/cases/<set>.jsonl --expected $G/expected/<set>.jsonl --cmd $A
python3 $H check   --batch --profile cr --cases $G/cases/<set>.jsonl --cmd $A --checker "python3 $G/checkers/invariants.py"
```

CI (`.github/workflows/ci.yml`) runs all of these.

## License

MIT. Implements the API of mazelib (MIT, © 2014 John Stilley); contains none of its code.
