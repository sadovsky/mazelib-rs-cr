
## maze
- File: `.slopfork/spec/maze.md` (Maze facade: construction/seeding, generate, generate_entrances, solve + prune, transmute, tostring, generate_monte_carlo).
- Command: `leak_audit.py --source /home/user/mazelib-source --against .slopfork/spec/maze.md --mode spec --allow-identifiers-from <source>/docs --allow-identifiers-from <source>/README.md --allow-identifiers-from <goldens>`
- First run: 1 finding (a 12-token run: a worked-example coordinate list that also appears in a source test). Fixed by describing the path in words instead.
- Final run (2026-10-22): 0 findings across 65 source files. Clean.

## generate
- Files: `.slopfork/spec/generate/_common.md` plus one spec per generator: AldousBroder, BacktrackingGenerator, BinaryTree, CellularAutomaton, Division, DungeonRooms, Ellers, GrowingTree, HuntAndKill, Kruskal, Prims, Sidewinder, TrivialMaze, Wilsons (15 files).
- Command: `leak_audit.py --source /home/user/mazelib-source --against .slopfork/spec/generate --mode spec --allow-identifiers-from <source>/docs --allow-identifiers-from <source>/README.md --allow-identifiers-from <goldens>`
- First run: 15 findings. 13 were 12-token runs where the worked-example grids, drawn in `#`, matched an ASCII maze literal in a source test file. Fixed by drawing the examples with `X` for wall, with a note that the protocol uses `#`. The other 2 were the room-corner parameter names written in snake_case; reworded as plain words. Also removed one source assertion message that is not in the goldens.
- Final run (2026-10-22): 0 findings across 65 source files. Clean.

## solve
- Files: `.slopfork/spec/solve/_common.md`, `BacktrackingSolver.md`, `Chain.md`, `Collision.md`, `RandomMouse.md`, `ShortestPath.md`, `ShortestPaths.md`, `Tremaux.md`.
- Command: `leak_audit.py --source /home/user/mazelib-source --against .slopfork/spec/solve --mode spec --allow-identifiers-from <source>/docs --allow-identifiers-from <source>/README.md --allow-identifiers-from <goldens>`
- First run: 8 findings, all 12-token runs shared with a source test file: the Prims3x3 example grid rows and its solution coordinate list. A second run flagged 4 more coordinate lists. Fixed by describing the grid, the solution and the raw walks in words and by referring to golden case ids.
- Final run (2026-10-22): 0 findings across 65 source files. Clean.

## transmute
- Files: `.slopfork/spec/transmute/_common.md`, `CuldeSacFiller.md`, `DeadEndFiller.md`, `Perturbation.md`.
- Command: as for solve, with `--against .slopfork/spec/transmute`.
- First run (2026-10-22): 0 findings across 65 source files. Clean.

## Re-audit after builder answers (2026-10-22)
- Spec changes: `rng.md` (item 14 seeding order note; new item 15 on randbelow(0)), `generate/Ellers.md` (Invariants for xskew 0/1 corrected; Q-gen-2), `generate/CellularAutomaton.md` (Q3: non-finite and huge parameters), `generate/DungeonRooms.md` (Q4, Q5), `transmute/_common.md` (item 7 out-of-grid entrances; Q-tr-1, Q-tr-3), `solve/ShortestPath.md` (Q-solve-1).
- Command (per target: `rng.md`, `generate`, `solve`, `transmute`, `maze.md`): `leak_audit.py --source /home/user/mazelib-source --against <target> --mode spec --allow-identifiers-from /home/user/mazelib-source/docs --allow-identifiers-from /home/user/mazelib-source/README.md --allow-identifiers-from /home/user/slopfork-mazelib-goldens`
- Result: 0 findings for each of the five targets, across 65 source files. Clean.
