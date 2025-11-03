//! Simple, generic 3D neural network grid: layers (x), nodes per layer (y), and weights per node (z).
//! Designed to be easy to traverse for future 3D/isometric rendering.

use rand::distributions::{uniform::SampleUniform, Distribution, Uniform};
use std::str::FromStr;

/// A compact, flat 3D grid representing a neural network structure.
///
/// Dimensions:
/// - x: number of layers
/// - y: number of nodes per layer
/// - z: number of weights per node
///
/// Storage is laid out as [x][y][z] in a single Vec, with index formula:
/// idx = x * (y*z) + y * z + z
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeuralNetwork<T> {
    x_layers: usize,
    y_nodes: usize,
    z_weights: usize,
    data: Vec<T>,
}

// -------- RNG Abstraction (no new external libraries) --------

pub trait RngSource {
    fn next_f32(&mut self, low: f32, high: f32) -> f32;
    fn next_f64(&mut self, low: f64, high: f64) -> f64;
}

/// Default RNG: uses rand::thread_rng() with no seed for non-deterministic output.
pub struct DefaultRng(rand::rngs::ThreadRng);
impl Default for DefaultRng {
    fn default() -> Self { Self(rand::thread_rng()) }
}
impl RngSource for DefaultRng {
    fn next_f32(&mut self, low: f32, high: f32) -> f32 {
        use rand::Rng;
        self.0.gen_range(low..high)
    }
    fn next_f64(&mut self, low: f64, high: f64) -> f64 {
        use rand::Rng;
        self.0.gen_range(low..high)
    }
}

/// Mulberry64-style simple PRNG (fast, not cryptographic). Seeded.
/// Implementation uses a SplitMix64-like sequence to produce 64-bit values,
/// then maps to f32 in [low, high).
pub struct Mulberry64 {
    state: u64,
}
impl Mulberry64 { pub fn new(seed: u64) -> Self { Self { state: seed } } }
impl RngSource for Mulberry64 {
    fn next_f32(&mut self, low: f32, high: f32) -> f32 {
        // SplitMix64 step (good-quality non-crypto RNG)
        let mut z = { self.state = self.state.wrapping_add(0x9E3779B97F4A7C15); self.state };
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        // Map to [0,1)
        let u = (z >> 11) as f64 / (1u64 << 53) as f64; // 53-bit mantissa
        (low as f64 + u * (high as f64 - low as f64)) as f32
    }
    fn next_f64(&mut self, low: f64, high: f64) -> f64 {
        let mut z = { self.state = self.state.wrapping_add(0x9E3779B97F4A7C15); self.state };
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        let u = (z >> 11) as f64 / (1u64 << 53) as f64;
        low + u * (high - low)
    }
}

/// Minimal PCG64-like generator (not a full reference PCG variant, but seeded and stable).
pub struct Pcg64 {
    state: u128,
    inc: u128,
}
impl Pcg64 {
    pub fn new(seed: u64) -> Self {
        // Use 128-bit state; ensure odd increment
        let s = (seed as u128) ^ 0xDA3E_39CB_94B9_5BDB_CAF9_1234_5678_9ABCu128;
        let inc = ((seed as u128) << 1) | 1;
        Self { state: s, inc }
    }
    fn next_u64(&mut self) -> u64 {
        // LCG step with 128-bit multiplier constant (not canonical PCG, but stable)
        const MUL: u128 = 0x2360_ED05_1FC6_5DA4_4385_DF64_9FCC_F645u128;
        self.state = self.state.wrapping_mul(MUL).wrapping_add(self.inc);
        // XSL RR output: xorshift high, then rotate right by derived amount
        let xorshifted = (((self.state >> 64) as u64) ^ (self.state as u64)) >> 5;
        let rot = (self.state >> 122) as u32; // top 6 bits
        xorshifted.rotate_right(rot)
    }
}
impl RngSource for Pcg64 {
    fn next_f32(&mut self, low: f32, high: f32) -> f32 {
        let v = self.next_u64();
        let u = (v >> 11) as f64 / (1u64 << 53) as f64;
        (low as f64 + u * (high as f64 - low as f64)) as f32
    }
    fn next_f64(&mut self, low: f64, high: f64) -> f64 {
        let v = self.next_u64();
        let u = (v >> 11) as f64 / (1u64 << 53) as f64;
        low + u * (high - low)
    }
}

/// Minimal ChaCha8-like generator (reduced-round stream cipher as PRNG). Seeded; non-crypto.
pub struct ChaCha8Rng {
    state: [u32; 16],
    idx: usize,
    buf: [u32; 16],
}
impl ChaCha8Rng {
    pub fn new(seed: u64) -> Self {
        // Set up a 512-bit state with constants and seed-derived key/nonce
        let mut st = [0u32; 16];
        st[0] = 0x6170_7865; // "expa"
        st[1] = 0x3320_646e; // "nd 3"
        st[2] = 0x7962_2d32; // "2-by"
        st[3] = 0x6b20_6574; // "te k"
        // Key from seed expanded with splitmix-like
        let mut sm = Mulberry64::new(seed);
        for i in 0..8 { st[4 + i] = (sm.next_f32(0.0, f32::MAX) as u32) ^ (0x9E37_79B9u32.wrapping_mul(i as u32 + 1)); }
        st[12] = 0; // counter
        st[13] = 0;
        st[14] = (seed as u32) ^ 0xDEAD_BEEF;
        st[15] = ((seed >> 32) as u32) ^ 0xBADC_0FFE;
        let mut rng = Self { state: st, idx: 16, buf: [0; 16] };
        rng.refill();
        rng
    }
    fn quarter(x: &mut [u32; 16], ai: usize, bi: usize, ci: usize, di: usize) {
        // Work on local copies to avoid aliasing &mut borrows; write back at end.
        let mut a = x[ai];
        let mut b = x[bi];
        let mut c = x[ci];
        let mut d = x[di];
        a = a.wrapping_add(b); d ^= a; d = d.rotate_left(16);
        c = c.wrapping_add(d); b ^= c; b = b.rotate_left(12);
        a = a.wrapping_add(b); d ^= a; d = d.rotate_left(8);
        c = c.wrapping_add(d); b ^= c; b = b.rotate_left(7);
        x[ai] = a; x[bi] = b; x[ci] = c; x[di] = d;
    }
    fn refill(&mut self) {
        let mut x = self.state;
        for _ in 0..8 { // 8 rounds
            // column rounds
            Self::quarter(&mut x, 0, 4, 8, 12);
            Self::quarter(&mut x, 1, 5, 9, 13);
            Self::quarter(&mut x, 2, 6, 10, 14);
            Self::quarter(&mut x, 3, 7, 11, 15);
            // diagonal rounds
            Self::quarter(&mut x, 0, 5, 10, 15);
            Self::quarter(&mut x, 1, 6, 11, 12);
            Self::quarter(&mut x, 2, 7, 8, 13);
            Self::quarter(&mut x, 3, 4, 9, 14);
        }
        for i in 0..16 { self.buf[i] = x[i].wrapping_add(self.state[i]); }
        // increment counter
        self.state[12] = self.state[12].wrapping_add(1);
        if self.state[12] == 0 { self.state[13] = self.state[13].wrapping_add(1); }
        self.idx = 0;
    }
    fn next_u32(&mut self) -> u32 {
        if self.idx >= 16 { self.refill(); }
        let v = self.buf[self.idx];
        self.idx += 1;
        v
    }
}
impl RngSource for ChaCha8Rng {
    fn next_f32(&mut self, low: f32, high: f32) -> f32 {
        let v = ((self.next_u32() as u64) << 21) as f64 / (1u64 << 53) as f64;
        (low as f64 + v * (high as f64 - low as f64)) as f32
    }
    fn next_f64(&mut self, low: f64, high: f64) -> f64 {
        let v = ((self.next_u32() as u64) << 21) as f64 / (1u64 << 53) as f64;
        low + v * (high - low)
    }
}

/// Algorithm selector, driven by CLI.
pub enum RngKind { Default, Mulberry64(u64), Pcg64(u64), ChaCha8(u64) }
impl FromStr for RngKind {
    type Err = (); fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "default" => Ok(RngKind::Default),
            "mulberry64" => Ok(RngKind::Mulberry64(0)),
            "pcg64" => Ok(RngKind::Pcg64(0)),
            "chacha8" | "chacha8rng" => Ok(RngKind::ChaCha8(0)),
            _ => Err(()),
        }
    }
}

pub fn make_rng_from_args<I, S>(mut args: I) -> Box<dyn RngSource>
where I: Iterator<Item = S>, S: AsRef<str>
{
    // Accept flags: --rng=<name> and --seed=<u64>
    let mut kind = RngKind::Default;
    let mut seed: u64 = 0xDEAD_BEEF_F00D_BAAF;
    while let Some(a) = args.next() {
        let a = a.as_ref();
        if let Some(v) = a.strip_prefix("--rng=") {
            if let Ok(k) = RngKind::from_str(v) { kind = k; }
        } else if let Some(v) = a.strip_prefix("--seed=") {
            if let Ok(s) = v.parse::<u64>() { seed = s; }
        }
    }
    match kind {
        RngKind::Default => Box::new(DefaultRng::default()),
        RngKind::Mulberry64(_) => Box::new(Mulberry64::new(seed)),
        RngKind::Pcg64(_) => Box::new(Pcg64::new(seed)),
        RngKind::ChaCha8(_) => Box::new(ChaCha8Rng::new(seed)),
    }
}

/// Extract the RNG algorithm name from CLI args (default if not provided).
pub fn rng_name_from_args<I, S>(mut args: I) -> String
where I: Iterator<Item = S>, S: AsRef<str>
{
    let mut name = String::from("default");
    while let Some(a) = args.next() {
        let a = a.as_ref();
        if let Some(v) = a.strip_prefix("--rng=") {
            name = v.to_ascii_lowercase();
        }
    }
    name
}

impl<T> NeuralNetwork<T> {
    /// Create a new network filled with a cloned default value.
    pub fn new_with_value(x_layers: usize, y_nodes: usize, z_weights: usize, value: T) -> Self
    where
        T: Clone,
    {
        let size = x_layers
            .checked_mul(y_nodes)
            .and_then(|v| v.checked_mul(z_weights))
            .expect("dimensions too large (overflow)");
        Self {
            x_layers,
            y_nodes,
            z_weights,
            data: vec![value; size],
        }
    }

    /// Dimensions as (x_layers, y_nodes, z_weights)
    #[inline]
    pub fn dims(&self) -> (usize, usize, usize) {
        (self.x_layers, self.y_nodes, self.z_weights)
    }

    /// Total number of scalar elements (x * y * z).
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if there are no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Calculate flat index from 3D coordinates, if in-bounds.
    #[inline]
    fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x < self.x_layers && y < self.y_nodes && z < self.z_weights {
            let yz = self.y_nodes * self.z_weights;
            Some(x * yz + y * self.z_weights + z)
        } else {
            None
        }
    }

    /// Get immutable reference by 3D coordinate.
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        self.index(x, y, z).map(|i| &self.data[i])
    }

    /// Get mutable reference by 3D coordinate.
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut T> {
        self.index(x, y, z).map(move |i| &mut self.data[i])
    }

    /// Visit every element with its 3D coordinates.
    pub fn for_each<F: FnMut(usize, usize, usize, &T)>(&self, mut f: F) {
        let (x_max, y_max, z_max) = self.dims();
        for x in 0..x_max {
            for y in 0..y_max {
                for z in 0..z_max {
                    // Safety: checked bounds via loops
                    let idx = x * (y_max * z_max) + y * z_max + z;
                    f(x, y, z, &self.data[idx]);
                }
            }
        }
    }

    /// Mutable visit for every element with its 3D coordinates.
    pub fn for_each_mut<F: FnMut(usize, usize, usize, &mut T)>(&mut self, mut f: F) {
        let (x_max, y_max, z_max) = self.dims();
        for x in 0..x_max {
            for y in 0..y_max {
                for z in 0..z_max {
                    let idx = x * (y_max * z_max) + y * z_max + z;
                    f(x, y, z, &mut self.data[idx]);
                }
            }
        }
    }
}

impl<T> NeuralNetwork<T>
where
    T: Copy + SampleUniform,
{
    /// Create a network filled with random values uniformly sampled in [low, high].
    /// Requires T types supported by rand's Uniform (e.g., f32, f64, integers).
    pub fn random_uniform(
        x_layers: usize,
        y_nodes: usize,
        z_weights: usize,
        low: T,
        high: T,
    ) -> Self {
        let size = x_layers
            .checked_mul(y_nodes)
            .and_then(|v| v.checked_mul(z_weights))
            .expect("dimensions too large (overflow)");

    let mut rng = rand::thread_rng();
    let dist = Uniform::new(low, high);
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(dist.sample(&mut rng));
        }

        Self {
            x_layers,
            y_nodes,
            z_weights,
            data,
        }
    }
}

impl NeuralNetwork<f32> {
    /// Create a network filled with random f32 values in [low, high) from a custom RNG.
    pub fn random_uniform_f32_with(
        x_layers: usize,
        y_nodes: usize,
        z_weights: usize,
        low: f32,
        high: f32,
        rng: &mut dyn RngSource,
    ) -> Self {
        let size = x_layers
            .checked_mul(y_nodes)
            .and_then(|v| v.checked_mul(z_weights))
            .expect("dimensions too large (overflow)");
        let mut data = Vec::with_capacity(size);
        for _ in 0..size { data.push(rng.next_f32(low, high)); }
        Self { x_layers, y_nodes, z_weights, data }
    }
}

impl NeuralNetwork<f64> {
    /// Create a network filled with random f64 values in [low, high) from a custom RNG.
    pub fn random_uniform_f64_with(
        x_layers: usize,
        y_nodes: usize,
        z_weights: usize,
        low: f64,
        high: f64,
        rng: &mut dyn RngSource,
    ) -> Self {
        let size = x_layers
            .checked_mul(y_nodes)
            .and_then(|v| v.checked_mul(z_weights))
            .expect("dimensions too large (overflow)");
        let mut data = Vec::with_capacity(size);
        for _ in 0..size { data.push(rng.next_f64(low, high)); }
        Self { x_layers, y_nodes, z_weights, data }
    }
}

#[cfg(test)]
mod tests {
    use super::NeuralNetwork;

    #[test]
    fn dims_and_indexing() {
        let mut nn = NeuralNetwork::new_with_value(2, 3, 4, 0i32);
        assert_eq!(nn.dims(), (2, 3, 4));
        assert_eq!(nn.len(), 2 * 3 * 4);

        // Write specific coordinates and read back
        *nn.get_mut(1, 2, 3).unwrap() = 42;
        assert_eq!(*nn.get(1, 2, 3).unwrap(), 42);

        // Out-of-bounds
        assert!(nn.get(2, 0, 0).is_none());
        assert!(nn.get(0, 3, 0).is_none());
        assert!(nn.get(0, 0, 4).is_none());
    }
}
