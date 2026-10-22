# mazelib-rs-cr

A port of [mazelib](https://github.com/john-science/mazelib) (Python) to **rust** using the slopfork process in **cleanroom** mode. It's one arm of a four-way experiment: {TypeScript, Rust} x {transpile, cleanroom}, all checked against the same goldens.

The slopfork skill, agents, and guard hook are vendored in `.claude/` (pinned copy from `sadovsky/slopfork`, branch `claude/slopfork-plugin`). Follow `.claude/skills/slopfork/SKILL.md` for the phases. Keep `.slopfork/config.json` `phase` accurate: the guard enforces it.

## Shared inputs
- Goldens: clone `https://github.com/sadovsky/slopfork-mazelib-goldens` to `../slopfork-mazelib-goldens`. Read its `PROTOCOL.md` and `README.md` first. They define the adapter protocol, the 2,396 cases, and the `tx`/`cr` compare profiles.
- Harness: `.claude/skills/slopfork/scripts/diff_harness.py`.
- The original crashes on the cases in the goldens' `KNOWN_BUGS.json`. Ports must return a valid result there instead.

## Mode: cleanroom

You are running the **clean side** of a clean-room port. The guard hook in `.claude/hooks/` enforces it and logs every attempt to `.slopfork/provenance.jsonl`.

- **Only the `spec-writer` subagent** may read the original source, and only while `.slopfork/config.json` `phase` is `inventory`, `spec`, `harness`, or `audit`. The main thread and `builder`/`verifier` agents never read it, and never paste anything from it into a prompt.
- The spec-writer clones `https://github.com/john-science/mazelib` at `56cd88294a36746bbd324c2c9f7e4ab26095deb3` into `../mazelib-source` (sealed). **Never** `pip install mazelib`, and never fetch mazelib code from GitHub or PyPI on the clean side.
- Allowed on the clean side: mazelib's README and `docs/` (unsealed in config, also on GitHub's rendered pages), the goldens repo, and the general literature on maze algorithms.
- **Never read** the transpile repos (`mazelib-ts-tx`, `mazelib-rs-tx`); they're sealed too.
- Target profile: **`cr`**. Exact match where the output doesn't depend on implementation choices (the RNG layer, `tostring`, error kinds on invalid input); otherwise the discovered invariants. The RNG layer must still be bit-compatible with Python's `random` and NumPy's legacy `RandomState` (those are public library behaviors, not mazelib's code), but how your algorithms *use* the RNG is your own design.
- Specs live in `.slopfork/spec/`, are leak-audited (`.claude/skills/slopfork/scripts/leak_audit.py`), and are the only channel from the dirty side. Builder questions go in `.slopfork/questions.md`.
- Add `NOTICE`: "An independent clean-room implementation of the mazelib API (github.com/john-science/mazelib, MIT). Not affiliated with mazelib." 

## Target conventions
- Stable Rust, edition 2021, `#![forbid(unsafe_code)]`. The library crate `mazelib` has **zero dependencies**; the adapter binary may use `serde`/`serde_json`.
- Idiomatic Rust API (snake_case, `Result` for invalid input, no panics on valid input), with a mapping table to the Python API in the README.
- Grids: a `Grid` struct over `Vec<u8>` (row-major).
- Adapter: binary target `mazelib-adapter` (`cargo run --release --bin mazelib-adapter`), JSON lines on stdin, see the goldens `PROTOCOL.md`.
- Map Python exception kinds onto the protocol's `error.kind` strings (`AssertionError`, `ValueError`, ...) in the adapter only; the library uses typed errors.
- Checks: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

## Definition of done
For every case set in `../slopfork-mazelib-goldens/cases/`, with `P=cr`:
```
H=.claude/skills/slopfork/scripts/diff_harness.py
G=../slopfork-mazelib-goldens
python3 $H compare --batch --profile $P --cases $G/cases/<set>.jsonl --expected $G/expected/<set>.jsonl --cmd "<adapter>"
python3 $H check   --batch --profile $P --cases $G/cases/<set>.jsonl --cmd "<adapter>" --checker "python3 $G/checkers/invariants.py"
```
Both must pass on all 7 sets, and the language checks above must be clean. Then:
- A GitHub Actions workflow runs the language checks and both harness commands (it checks out the goldens repo).
- `.slopfork/features.md` is the parity matrix (every generator/solver/transmuter, plus the RNG and Maze facade).
- `.slopfork/REPORT.md` follows the slopfork report template.
- `.slopfork/metrics.json` records `{"started_at", "finished_at", "src_loc", "test_loc", "spec_questions", "guard_denials", "cases_exact", "cases_invariant", "cases_total", "deviations": [...]}`.

## Git
This repo exists only for this experiment. Commit in small, working steps and push to `main`.
