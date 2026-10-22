# Builder questions

## rng
- Q1 (spec rng item 14): When the seed is out of range for NumPy, is the Python stream seeded before the error is raised? Conservative choice: `Rng::set_seed` checks the range first and leaves both streams unchanged on error. The case fails either way, so the goldens can't tell the difference. (`// SLOPFORK-Q:` in src/rng.rs)
- Q2 (spec rng item 5): The spec only covers `randbelow(n)` for n >= 1. Conservative choice: `randbelow(0)` / `below(0)` return 0 without drawing, so there's no panic or infinite loop.
