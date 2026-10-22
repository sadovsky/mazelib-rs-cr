# Spec: rng

Provenance: documented (public behavior of CPython 3.13 `random` and NumPy legacy `RandomState`; written on the clean side, not derived from mazelib) · Audit: n/a (no mazelib content)

## Purpose
Two independent Mersenne Twister (MT19937, Matsumoto & Nishimura 1998, reference `mt19937ar.c`) streams that reproduce, bit for bit, CPython's `random` module and NumPy's legacy `numpy.random` (RandomState) for the calls the adapter's `rng` op exposes. Algorithms in this crate may draw from either stream in any way they like; only the `rng` op is compared exactly.

## Public interface
- `PyRandom` — CPython-compatible stream: `seed(int)`, `random() -> float`, `getrandbits(k) -> int`, `randbelow(n) -> int`, `randrange(start, stop?, step?) -> int`, `randint(a, b) -> int`, `choice(n) -> index`, `shuffle(list)`.
- `NpRandom` — NumPy-legacy-compatible stream: `seed(int)`, `shuffle(list)`, `random_interval(max)`, plus whatever the algorithms want (e.g. `random_sample() -> float` = 53-bit double like CPython's).
- `Rng` — holds one of each; `set_seed(seed)` seeds both.

## Behavior
1. **MT19937 core**: N=624, standard tempering, `genrand_uint32`. `init_genrand(s)`: `mt[0]=s`, `mt[i] = 1812433253 * (mt[i-1] ^ (mt[i-1] >> 30)) + i` (mod 2^32). `init_by_array(key)`: the reference algorithm (start from `init_genrand(19650218)`, constants 1664525 and 1566083941, `mt[0] = 0x80000000` at the end).
2. **CPython seed(a)** for an int `a`: take `|a|`, split into 32-bit words little-endian (value 0 → the single word `[0]`), call `init_by_array(words)`. Negative seeds are therefore accepted by this stream.
3. **CPython random()**: `a = u32 >> 5`, `b = u32 >> 6` (two draws, in that order); result `(a * 67108864 + b) / 9007199254740992`.
4. **CPython getrandbits(k)**, k ≥ 1: if k ≤ 32 one draw `u32 >> (32 - k)`. Otherwise draw `ceil(k/32)` words; word i (in draw order) supplies bits `32*i ..`, i.e. the first draw is the least significant word; the last (most significant) word, if `k % 32 != 0`, is shifted right by `32 - k % 32`. Values up to 2^64 must work (case `randrange(0, 2**40)`); use u64/u128 internally.
5. **CPython randbelow(n)**, n ≥ 1: `k = bit_length(n)`; repeat `r = getrandbits(k)` until `r < n`. (n = 1 still draws: k = 1.)
6. **CPython randrange(start, stop=None, step=1)**:
   - one argument: if `start > 0` return `randbelow(start)`, else **ValueError** ("empty range for randrange()").
   - `step == 1`: `width = stop - start`; if `width > 0` return `start + randbelow(width)`, else ValueError.
   - other steps: `n = (width + step - 1) // step` for step > 0, `(width + step + 1) // step` for step < 0 (floor division), step 0 → ValueError; if `n <= 0` ValueError; return `start + step * randbelow(n)`.
7. **randint(a, b)** = `randrange(a, b + 1)`.
8. **choice(seq)** = `seq[randbelow(len(seq))]`; empty sequence → IndexError.
9. **CPython shuffle(x)**: for `i` from `len(x)-1` down to 1: `j = randbelow(i + 1)`; swap `x[i]`, `x[j]`.
10. **NumPy legacy seed(s)** for an int: must satisfy `0 <= s <= 2^32 - 1`, otherwise **ValueError** ("Seed must be between 0 and 2**32 - 1"). Then `init_genrand(s)` (NOT init_by_array).
11. **NumPy legacy random_interval(max)**: `max == 0` → 0 with no draw. Otherwise `mask` = `max` with every bit below its highest set bit also set; if `max <= 0xFFFFFFFF` repeat `v = u32 & mask` until `v <= max`; else the same with 64-bit draws (`(hi << 32) | lo`, first draw is the high word).
12. **NumPy legacy shuffle(x)** (1-D): for `i` from `len(x)-1` down to 1: `j = random_interval(i)`; swap `x[i]`, `x[j]`.
13. **NumPy random_sample()**: same 53-bit construction as item 3, on the NumPy stream.
14. **set_seed(seed)** (the `Maze.set_seed` / adapter `seed` semantics): seeds both streams; if the NumPy seed is out of range the whole case fails with ValueError (before the op runs). Observed order: the Python stream is seeded first, then the NumPy seed is rejected, so after the error the Python stream is reseeded and the NumPy stream is unchanged. The goldens cannot observe this (the case fails either way); checking the range first and leaving both streams unchanged is also acceptable.
15. **randbelow(0)** is never reached on valid input: every public path that could ask for it (`choice` on empty, empty `randrange`) raises first. A port may return 0 without drawing; the original library would not terminate there.

## Errors
| Condition | Error kind |
|---|---|
| seed < 0 or seed > 2^32-1 (NumPy stream) | ValueError |
| randrange/randint empty range, zero step | ValueError |
| choice on empty | IndexError |

## Worked examples (from `../slopfork-mazelib-goldens/expected/rng.jsonl`)
- seed 0, eight `random()` calls → `0.8444218515250481, 0.7579544029403025, 0.420571580830845, 0.25891675029296335, ...`
- seed 0, `[randrange 10], [randrange 1 11 2], [randrange 0 2^40], [randrange 1 1000000007], [randrange 5 6], [randint 1 6], [randint -3 3]` → `[6, 7, 567109562164, 548977049, 5, 4, 3]`
- seed -7 or 4294967301 → ValueError for every call list.
