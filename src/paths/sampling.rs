use std::f64::consts::PI;

use rand;
use rand::Rng;


pub trait Sampler : SamplerClone + Send {
    // Samples the unit square, returning x, y.
    fn sample_square(&mut self) -> (f64, f64);

    // Samples the unit disk, returning x, y.
    fn sample_disk(&mut self) -> (f64, f64);
}

pub trait SamplerClone {
    fn clone_box(&self) -> Box<Sampler>;
}

impl <T> SamplerClone for T where T: 'static + Sampler + Clone {
    fn clone_box(&self) -> Box<Sampler> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Sampler> {
    fn clone(&self) -> Box<Sampler> {
        self.clone_box()
    }
}

#[derive(Clone, Debug)]
pub struct UniformSampler {}

impl Sampler for UniformSampler {
    fn sample_square(&mut self) -> (f64, f64) {
        let mut rng = rand::thread_rng();
        (rng.gen(), rng.gen())
    }

    fn sample_disk(&mut self) -> (f64, f64) {
        let mut rng = rand::thread_rng();
        let r = rng.gen::<f64>();
        let theta = rng.gen::<f64>();
        (r * theta.cos(), r * theta.sin())
    }
}

// This is the algorithm developed by Pixar for their RenderMan engine.
// All the random looking equations are part of the hash function they developed.
// See https://graphics.pixar.com/library/MultiJitteredSampling/paper.pdf for details.
#[derive(Clone, Debug)]
pub struct CorrelatedMultiJitteredSampler {
    p: u32,
    m: u32,
    n: u32,
    s: u32,
}

impl CorrelatedMultiJitteredSampler {
    pub fn new(p: u32, m: u32, n: u32) -> CorrelatedMultiJitteredSampler {
        CorrelatedMultiJitteredSampler{ p, m, n, s: 0 }
    }

    fn permute(mut i: u32, l: u32, p: u32) -> u32 {
        let mut w = l - 1;
        w |= w >> 1;
        w |= w >> 2;
        w |= w >> 4;
        w |= w >> 8;
        w |= w >> 16;
        while i > l {
            i ^= p;             i *= 0xe170_893d;
            i ^= p       >> 16;
            i ^= (i & w) >> 4;
            i ^= p       >> 8;  i *= 0x0929_eb3f;
            i ^= p       >> 23;
            i ^= (i & w) >> 1;  i *= 1 | p >> 27;
                                i *= 0x6935_fa69;
            i ^= (i & w) >> 11; i *= 0x74dc_b303;
            i ^= (i & w) >> 2;  i *= 0x9e50_1cc3;
            i ^= (i & w) >> 2;  i *= 0xc860_a3df;
            i &= w;
            i ^= i       >> 5;
        }

        (i + p) % l
    }

    fn rand_float(mut i: u32, p: u32) -> f64 {
        i ^= p;
        i ^= i >> 17;
        i ^= i >> 10; i *= 0xb365_34e5;
        i ^= i >> 12;
        i ^= i >> 21; i *= 0x93fc_4795;
        i ^= 0xdf6e_307f;
        i ^= i >> 17; i *= 1 | p >> 18;
        (i as f64) * (1.0 / 4_294_967_808.0)
    }

    // s = sample index
    // m, n = sample dimensions (m x n = N = number of samples total)
    // p = pattern index (like a seed)
    fn cmj(s: u32, m: u32, n: u32, p: u32) -> (f64, f64) {
        let sx: f64 = CorrelatedMultiJitteredSampler::permute(s % m, m, p * 0xa511_e9b3) as f64;
        let sy: f64 = CorrelatedMultiJitteredSampler::permute(s / m, n, p * 0x63d8_3595) as f64;
        let jx: f64 = CorrelatedMultiJitteredSampler::rand_float(s, p * 0xa399_d265);
        let jy: f64 = CorrelatedMultiJitteredSampler::rand_float(s, p * 0x711a_d6a5);
        let x: f64 = (((s % m) as f64) + (sy + jx) / (n as f64)) / (m as f64);
        let y: f64 = (((s / m) as f64) + (sx + jy) / (m as f64)) / (n as f64);
        (x, y)
    }
}

impl Sampler for CorrelatedMultiJitteredSampler {
    fn sample_square(&mut self) -> (f64, f64) {
        let sample = CorrelatedMultiJitteredSampler::cmj(self.s, self.m, self.n, self.p);
        self.n += 1;
        sample
    }

    fn sample_disk(&mut self) -> (f64, f64) {
        // Sample the square and then map to the disk.
        // Only works if m ~= n
        let (x, y) = CorrelatedMultiJitteredSampler::cmj(self.s, self.m, self.n, self.p);
        self.n += 1;

        let theta = 2.0 * PI * x;
        let r = y.sqrt();
        (r * theta.cos(), r * theta.sin())
    }
}
