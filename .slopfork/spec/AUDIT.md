
## maze
- File: `.slopfork/spec/maze.md` (Maze facade: construction/seeding, generate, generate_entrances, solve + prune, transmute, tostring, generate_monte_carlo).
- Command: `leak_audit.py --source /home/user/mazelib-source --against .slopfork/spec/maze.md --mode spec --allow-identifiers-from <source>/docs --allow-identifiers-from <source>/README.md --allow-identifiers-from <goldens>`
- First run: 1 finding (a 12-token run: a worked-example coordinate list that also appears in a source test). Fixed by describing the path in words instead.
- Final run (2026-10-22): 0 findings across 65 source files. Clean.
