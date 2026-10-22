
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
