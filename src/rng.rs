//! RNG layer: two independent MT19937 streams.
//!
//! [`PyRandom`] reproduces CPython's `random` module and [`NpRandom`]
//! reproduces NumPy's legacy `RandomState`, bit for bit, for the calls this
//! crate needs. [`Rng`] bundles one of each; `set_seed` seeds both, the way
//! `Maze.set_seed` does.

use crate::error::{MazeError, Result};

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// The MT19937 Mersenne Twister core (Matsumoto & Nishimura, 1998).
#[derive(Clone)]
pub struct Mt19937 {
    state: [u32; N],
    index: usize,
}

impl std::fmt::Debug for Mt19937 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mt19937")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

impl Mt19937 {
    /// A generator initialised with the reference `init_genrand(seed)`.
    pub fn new(seed: u32) -> Self {
        let mut mt = Mt19937 {
            state: [0; N],
            index: N,
        };
        mt.init_genrand(seed);
        mt
    }

    /// A generator initialised with the reference `init_by_array(key)`.
    pub fn from_key(key: &[u32]) -> Self {
        let mut mt = Mt19937::new(0);
        mt.init_by_array(key);
        mt
    }

    /// Reference `init_genrand`.
    pub fn init_genrand(&mut self, seed: u32) {
        self.state[0] = seed;
        for i in 1..N {
            let prev = self.state[i - 1];
            self.state[i] = 1_812_433_253u32
                .wrapping_mul(prev ^ (prev >> 30))
                .wrapping_add(i as u32);
        }
        self.index = N;
    }

    /// Reference `init_by_array`. An empty key is treated as `[0]`.
    pub fn init_by_array(&mut self, key: &[u32]) {
        let zero = [0u32];
        let key = if key.is_empty() { &zero[..] } else { key };
        self.init_genrand(19_650_218);
        let mt = &mut self.state;
        let mut i = 1usize;
        let mut j = 0usize;
        for _ in 0..N.max(key.len()) {
            let prev = mt[i - 1];
            mt[i] = (mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            let prev = mt[i - 1];
            mt[i] =
                (mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_566_083_941)).wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
        }
        mt[0] = 0x8000_0000;
        self.index = N;
    }

    fn twist(&mut self) {
        let mt = &mut self.state;
        for k in 0..N {
            let y = (mt[k] & UPPER_MASK) | (mt[(k + 1) % N] & LOWER_MASK);
            let mut v = mt[(k + M) % N] ^ (y >> 1);
            if y & 1 != 0 {
                v ^= MATRIX_A;
            }
            mt[k] = v;
        }
        self.index = 0;
    }

    /// Reference `genrand_int32`: the next tempered 32-bit output.
    pub fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    /// 53-bit float in `[0, 1)` (`genrand_res53`), two draws.
    pub fn next_f64(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64;
        let b = (self.next_u32() >> 6) as f64;
        (a * 67_108_864.0 + b) / 9_007_199_254_740_992.0
    }
}

fn empty_range() -> MazeError {
    MazeError::value("empty range for randrange()")
}

fn bit_length(n: u128) -> u32 {
    128 - n.leading_zeros()
}

/// A CPython-`random`-compatible stream.
#[derive(Debug, Clone)]
pub struct PyRandom {
    mt: Mt19937,
}

impl Default for PyRandom {
    fn default() -> Self {
        PyRandom::new(0)
    }
}

impl PyRandom {
    /// Equivalent to `random.Random(seed)` for an integer seed.
    pub fn new(seed: i128) -> Self {
        let mut r = PyRandom {
            mt: Mt19937::new(0),
        };
        r.seed(seed);
        r
    }

    /// `random.seed(seed)` for an integer: `|seed|` split into little-endian
    /// 32-bit words fed to `init_by_array`. Negative seeds are accepted.
    pub fn seed(&mut self, seed: i128) {
        let mut a = seed.unsigned_abs();
        let mut words = Vec::with_capacity(4);
        while a > 0 {
            words.push(a as u32);
            a >>= 32;
        }
        if words.is_empty() {
            words.push(0);
        }
        self.mt.init_by_array(&words);
    }

    /// Raw 32-bit draw.
    pub fn next_u32(&mut self) -> u32 {
        self.mt.next_u32()
    }

    /// `random.random()`: a float in `[0, 1)`.
    pub fn random(&mut self) -> f64 {
        self.mt.next_f64()
    }

    /// `random.getrandbits(k)`. `k == 0` yields 0 with no draw; `k` above
    /// 128 is saturated to 128 (the widest value this type can hold).
    pub fn getrandbits(&mut self, k: u32) -> u128 {
        let k = k.min(128);
        if k == 0 {
            return 0;
        }
        if k <= 32 {
            return (self.mt.next_u32() >> (32 - k)) as u128;
        }
        let words = k.div_ceil(32);
        let mut out: u128 = 0;
        for i in 0..words {
            let mut w = self.mt.next_u32();
            if i == words - 1 && !k.is_multiple_of(32) {
                w >>= 32 - k % 32;
            }
            out |= (w as u128) << (32 * i);
        }
        out
    }

    /// CPython `_randbelow(n)`: uniform in `[0, n)` by rejection sampling.
    /// `n == 0` returns 0 without drawing.
    pub fn randbelow(&mut self, n: u128) -> u128 {
        if n == 0 {
            return 0;
        }
        let k = bit_length(n);
        loop {
            let r = self.getrandbits(k);
            if r < n {
                return r;
            }
        }
    }

    /// `random.randrange(start, stop, step)`; `stop == None` means
    /// `randrange(start)` (and `step` is ignored).
    pub fn randrange(&mut self, start: i128, stop: Option<i128>, step: i128) -> Result<i128> {
        let Some(stop) = stop else {
            if start > 0 {
                return Ok(self.randbelow(start as u128) as i128);
            }
            return Err(empty_range());
        };
        let width = stop.checked_sub(start).ok_or_else(empty_range)?;
        if step == 1 {
            if width > 0 {
                let r = self.randbelow(width as u128) as i128;
                return Ok(start + r);
            }
            return Err(empty_range());
        }
        if step == 0 {
            return Err(MazeError::value("zero step for randrange()"));
        }
        let adj = if step > 0 { step - 1 } else { step + 1 };
        let n = width
            .checked_add(adj)
            .and_then(|num| floor_div(num, step))
            .ok_or_else(empty_range)?;
        if n <= 0 {
            return Err(empty_range());
        }
        let r = self.randbelow(n as u128) as i128;
        step.checked_mul(r)
            .and_then(|v| start.checked_add(v))
            .ok_or_else(empty_range)
    }

    /// `random.randint(a, b)`: uniform in `[a, b]`.
    pub fn randint(&mut self, a: i128, b: i128) -> Result<i128> {
        let stop = b.checked_add(1).ok_or_else(empty_range)?;
        self.randrange(a, Some(stop), 1)
    }

    /// The index `random.choice(seq)` would pick from a sequence of `len`.
    pub fn choice_index(&mut self, len: usize) -> Result<usize> {
        if len == 0 {
            return Err(MazeError::empty("Cannot choose from an empty sequence"));
        }
        Ok(self.randbelow(len as u128) as usize)
    }

    /// `random.shuffle(x)` (Fisher-Yates from the end).
    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        for i in (1..x.len()).rev() {
            let j = self.randbelow((i + 1) as u128) as usize;
            x.swap(i, j);
        }
    }

    /// `randbelow(n)` for an index range; `n == 0` returns 0 without drawing.
    pub fn below(&mut self, n: usize) -> usize {
        self.randbelow(n as u128) as usize
    }

    /// `random.choice(seq)`, or `None` on an empty slice (no draw).
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        let i = self.below(items.len());
        items.get(i)
    }

    /// `random.random() < p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.random() < p
    }

    /// `random.uniform(a, b)`: `a + (b - a) * random()`.
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }
}

/// Python-style floor division; `None` on overflow or zero divisor.
fn floor_div(a: i128, b: i128) -> Option<i128> {
    let q = a.checked_div(b)?;
    let r = a.checked_rem(b)?;
    if r != 0 && ((r < 0) != (b < 0)) {
        q.checked_sub(1)
    } else {
        Some(q)
    }
}

/// A NumPy-legacy-`RandomState`-compatible stream.
#[derive(Debug, Clone)]
pub struct NpRandom {
    mt: Mt19937,
}

impl Default for NpRandom {
    fn default() -> Self {
        NpRandom {
            mt: Mt19937::new(0),
        }
    }
}

impl NpRandom {
    /// `numpy.random.RandomState(seed)`; the seed must be in `[0, 2^32 - 1]`.
    pub fn new(seed: i128) -> Result<Self> {
        let mut r = NpRandom::default();
        r.seed(seed)?;
        Ok(r)
    }

    /// Check that a seed is acceptable to the legacy NumPy stream.
    pub fn check_seed(seed: i128) -> Result<u32> {
        u32::try_from(seed).map_err(|_| MazeError::value("Seed must be between 0 and 2**32 - 1"))
    }

    /// `numpy.random.seed(seed)`: `init_genrand(seed)` after a range check.
    pub fn seed(&mut self, seed: i128) -> Result<()> {
        let s = Self::check_seed(seed)?;
        self.mt.init_genrand(s);
        Ok(())
    }

    /// Raw 32-bit draw.
    pub fn next_u32(&mut self) -> u32 {
        self.mt.next_u32()
    }

    /// Raw 64-bit draw: high word first.
    pub fn next_u64(&mut self) -> u64 {
        let hi = self.mt.next_u32() as u64;
        let lo = self.mt.next_u32() as u64;
        (hi << 32) | lo
    }

    /// Legacy `random_interval(max)`: uniform in `[0, max]` by masked
    /// rejection; `max == 0` returns 0 without drawing.
    pub fn random_interval(&mut self, max: u64) -> u64 {
        if max == 0 {
            return 0;
        }
        let mask = u64::MAX >> max.leading_zeros();
        if max <= 0xffff_ffff {
            loop {
                let v = (self.mt.next_u32() as u64) & mask;
                if v <= max {
                    return v;
                }
            }
        }
        loop {
            let v = self.next_u64() & mask;
            if v <= max {
                return v;
            }
        }
    }

    /// Legacy `shuffle(x)` for a 1-D sequence.
    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        for i in (1..x.len()).rev() {
            let j = self.random_interval(i as u64) as usize;
            x.swap(i, j);
        }
    }

    /// `random_sample()`: a 53-bit float in `[0, 1)`.
    pub fn random_sample(&mut self) -> f64 {
        self.mt.next_f64()
    }
}

/// One CPython-compatible stream plus one NumPy-compatible stream.
#[derive(Debug, Clone)]
pub struct Rng {
    /// The `random`-module stream.
    pub py: PyRandom,
    /// The `numpy.random` stream.
    pub np: NpRandom,
}

impl Default for Rng {
    fn default() -> Self {
        Rng::new()
    }
}

impl Rng {
    /// Both streams seeded with 0.
    pub fn new() -> Self {
        Rng {
            py: PyRandom::new(0),
            np: NpRandom::default(),
        }
    }

    /// A pair of streams seeded via [`Rng::set_seed`].
    pub fn seeded(seed: i128) -> Result<Self> {
        let mut r = Rng::new();
        r.set_seed(seed)?;
        Ok(r)
    }

    /// Seed both streams. A seed outside `[0, 2^32 - 1]` is rejected
    /// (ValueError) and leaves both streams untouched.
    pub fn set_seed(&mut self, seed: i128) -> Result<()> {
        // SLOPFORK-Q: spec item 14 says the case fails if the NumPy seed is out
        // of range; we validate first so neither stream is modified on error.
        NpRandom::check_seed(seed)?;
        self.py.seed(seed);
        self.np.seed(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed0_floats() {
        let mut r = Rng::seeded(0).unwrap();
        let got: Vec<f64> = (0..4).map(|_| r.py.random()).collect();
        assert_eq!(
            got,
            [
                0.8444218515250481,
                0.7579544029403025,
                0.420571580830845,
                0.25891675029296335
            ]
        );
    }

    #[test]
    fn seed1_floats() {
        let mut r = Rng::seeded(1).unwrap();
        assert_eq!(r.py.random(), 0.13436424411240122);
        assert_eq!(r.py.random(), 0.8474337369372327);
    }

    fn ranges(seed: i128) -> Vec<i128> {
        let mut r = Rng::seeded(seed).unwrap();
        let p = &mut r.py;
        vec![
            p.randrange(10, None, 1).unwrap(),
            p.randrange(1, Some(11), 2).unwrap(),
            p.randrange(0, Some(1 << 40), 1).unwrap(),
            p.randrange(1, Some(1_000_000_007), 1).unwrap(),
            p.randrange(5, Some(6), 1).unwrap(),
            p.randint(1, 6).unwrap(),
            p.randint(-3, 3).unwrap(),
        ]
    }

    #[test]
    fn seed_ranges() {
        assert_eq!(ranges(0), [6, 7, 567109562164, 548977049, 5, 4, 3]);
        assert_eq!(ranges(1), [2, 9, 140719340484, 273878288, 5, 4, 3]);
    }

    #[test]
    fn other_seeds() {
        assert_eq!(
            ranges(2_147_483_647),
            [5, 1, 636498465001, 949060031, 5, 1, 3]
        );
        let mut r = Rng::seeded(4_294_967_295).unwrap();
        assert_eq!(r.py.random(), 0.6353574441341173);
        assert_eq!(r.py.random(), 0.20319993954407756);
        let mut r = Rng::seeded(42).unwrap();
        let mut a = vec![0u8];
        r.py.shuffle(&mut a);
        let mut b: Vec<u8> = (0..10).collect();
        r.py.shuffle(&mut b);
        assert_eq!(b, [7, 3, 2, 8, 5, 6, 9, 4, 0, 1]);
    }

    #[test]
    fn seed0_choice() {
        let mut r = Rng::seeded(0).unwrap();
        let got: Vec<usize> = [1, 2, 7, 1000]
            .iter()
            .map(|&n| r.py.choice_index(n).unwrap())
            .collect();
        assert_eq!(got, [0, 1, 0, 265]);
        assert!(matches!(r.py.choice_index(0), Err(MazeError::Empty(_))));
    }

    #[test]
    fn seed0_shuffles() {
        let mut r = Rng::seeded(0).unwrap();
        let mut a: Vec<u32> = vec![];
        r.py.shuffle(&mut a);
        let mut b = vec![0u32];
        r.py.shuffle(&mut b);
        let mut c: Vec<u32> = (0..10).collect();
        r.py.shuffle(&mut c);
        assert_eq!(c, [7, 8, 1, 5, 3, 4, 2, 0, 9, 6]);

        let mut r = Rng::seeded(0).unwrap();
        let mut d: Vec<u32> = (0..10).collect();
        r.np.shuffle(&mut d);
        assert_eq!(d, [2, 8, 4, 9, 1, 6, 7, 3, 0, 5]);
    }

    #[test]
    fn seed0_mixed() {
        let mut r = Rng::seeded(0).unwrap();
        assert_eq!(r.py.random(), 0.8444218515250481);
        let mut a: Vec<u32> = (0..5).collect();
        r.np.shuffle(&mut a);
        assert_eq!(a, [2, 0, 1, 3, 4]);
        assert_eq!(r.py.randrange(100, None, 1).unwrap(), 97);
        let mut b: Vec<u32> = (0..5).collect();
        r.py.shuffle(&mut b);
        assert_eq!(b, [4, 2, 1, 0, 3]);
        assert_eq!(r.py.choice_index(3).unwrap(), 1);
        let mut c: Vec<u32> = (0..5).collect();
        r.np.shuffle(&mut c);
        assert_eq!(c, [0, 2, 1, 4, 3]);
    }

    #[test]
    fn seed_errors() {
        for s in [-7i128, 4_294_967_301] {
            let e = Rng::seeded(s).unwrap_err();
            assert_eq!(
                e,
                MazeError::InvalidValue("Seed must be between 0 and 2**32 - 1".into())
            );
        }
        assert!(Rng::seeded(4_294_967_295).is_ok());
        // The Python stream alone accepts any integer seed.
        let _ = PyRandom::new(-7);
        let _ = PyRandom::new(4_294_967_301);
    }

    #[test]
    fn randrange_errors() {
        let mut r = Rng::seeded(0).unwrap();
        let e = r.py.randrange(0, None, 1).unwrap_err();
        assert_eq!(
            e,
            MazeError::InvalidValue("empty range for randrange()".into())
        );
        assert!(r.py.randrange(5, Some(5), 1).is_err());
        assert!(r.py.randrange(0, Some(10), 0).is_err());
        assert!(r.py.randrange(0, Some(10), -1).is_err());
        let v = r.py.randrange(10, Some(0), -3).unwrap();
        assert!([10, 7, 4, 1].contains(&v));
    }

    #[test]
    fn floor_div_matches_python() {
        assert_eq!(floor_div(7, 2), Some(3));
        assert_eq!(floor_div(-7, 2), Some(-4));
        assert_eq!(floor_div(7, -2), Some(-4));
        assert_eq!(floor_div(-7, -2), Some(3));
    }

    #[test]
    fn interval_large() {
        let mut np = NpRandom::new(5).unwrap();
        for _ in 0..100 {
            assert!(np.random_interval(0x1_0000_0005) <= 0x1_0000_0005);
        }
        assert_eq!(np.random_interval(0), 0);
    }
}
