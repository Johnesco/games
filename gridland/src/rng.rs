// Tiny deterministic PRNG (xorshift64) so we avoid pulling in getrandom under wasm.

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let s = seed.wrapping_mul(0x9E3779B97F4A7C15) ^ 0xDEADBEEFCAFEBABE;
        Self { state: s | 1 }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn gen_f32(&mut self) -> f32 {
        // 24 high bits → [0,1)
        ((self.next_u64() >> 40) as f32) / ((1u64 << 24) as f32)
    }

    pub fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo) as u64;
        lo + (self.next_u64() % span) as i32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.gen_f32() < p
    }
}
