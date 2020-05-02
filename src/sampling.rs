use std::f64::consts::PI;

use rand;
use rand::Rng;
use rand::rngs::StdRng;
use rand::distributions::Uniform;
use rand::SeedableRng;

pub trait IntoPattern<S> {
    fn pattern<D>(self) -> Pattern<D, S>
    where Pattern<D, S> : CreatePattern<D, S>;
}

impl <S> IntoPattern<S> for S {
    fn pattern<D>(self) -> Pattern<D, S>
    where Pattern<D, S> : CreatePattern<D, S> {
        Pattern::new(self)
    }
}

#[derive(Clone, Debug)]
pub struct Pattern<Domain, T> {
    domain: Domain,
    sampler: T,
}

pub trait CreatePattern<D, S> {
    fn new(sampler: S) -> Pattern<D, S>;
}

impl <S> CreatePattern<Square, S> for Pattern<Square, S> {
    fn new(sampler: S) -> Pattern<Square, S> { 
        Pattern{ domain: Square, sampler }
    }
}

impl <S> CreatePattern<Disk, S> for Pattern<Disk, S> {
    fn new(sampler: S) -> Pattern<Disk, S> { 
        Pattern{ domain: Disk, sampler }
    }
}

impl <T> Iterator for Pattern<Square, T>
where Square : SampleFrom<T> {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<(f64, f64)> {
        Square::sample_from(&mut self.sampler)
    }
}

impl <T> Iterator for Pattern<Disk, T>
where Disk : SampleFrom<T> {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<(f64, f64)> {
        Disk::sample_from(&mut self.sampler)
    }
}

pub struct Square;
pub struct Disk;

pub trait SampleFrom<T> {
    fn sample_from(sampler: &mut T) -> Option<(f64, f64)>;
}

impl <T> SampleFrom<T> for Square
where T : SquareSampler {
    fn sample_from(sampler: &mut T) -> Option<(f64, f64)> {
        sampler.next_sample_square()
    }
}

impl <T> SampleFrom<T> for Disk
where T : DiskSampler {
    fn sample_from(sampler: &mut T) -> Option<(f64, f64)> {
        sampler.next_sample_disk()
    }
}

pub trait SquareSampler {
    fn next_sample_square(&mut self) -> Option<(f64, f64)>;
}

pub trait DiskSampler {
    fn next_sample_disk(&mut self) -> Option<(f64, f64)>;
}

#[derive(Clone, Debug)]
pub enum Sampler {
    Uniform(UniformSampler),
    CMJ(CorrelatedMultiJitteredSampler),
}

impl SquareSampler for Sampler {
    fn next_sample_square(&mut self) -> Option<(f64, f64)> {
        match self {
            Sampler::Uniform(s) => s.next_sample_square(),
            Sampler::CMJ(s) => s.next_sample_square(),
        }
    }
}

impl DiskSampler for Sampler {
    fn next_sample_disk(&mut self) -> Option<(f64, f64)> {
        match self {
            Sampler::Uniform(s) => s.next_sample_disk(),
            Sampler::CMJ(s) => s.next_sample_disk(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UniformSampler {
    rng: rand::rngs::StdRng,
    distribution: Uniform<f64>,
    remaining_samples: u32,
}

impl UniformSampler {
    pub fn new(p: u32, m: u32, n: u32) -> UniformSampler {
        let rng: StdRng = SeedableRng::seed_from_u64(p as u64);
        let distribution = Uniform::new(0.0, 1.0);
        UniformSampler{
            rng,
            distribution,
            remaining_samples: m * n,
        }
    }

    pub fn random(m: u32, n: u32) -> UniformSampler {
        let p: u32 = rand::thread_rng().gen();
        UniformSampler::new(p, m, n)
    }

    fn random_number(&mut self) -> f64 {
        self.rng.sample(self.distribution)
    }
}

impl SquareSampler for UniformSampler {
    fn next_sample_square(&mut self) -> Option<(f64, f64)> {
        return if self.remaining_samples == 0 {
            None
        } else {
            self.remaining_samples -= 1;
            Some((self.random_number(), self.random_number()))
        }
    }
}

impl DiskSampler for UniformSampler {
    fn next_sample_disk(&mut self) -> Option<(f64, f64)> {
        return if self.remaining_samples == 0 {
            None
        } else {
            self.remaining_samples -= 1;
            let r = self.random_number();
            let theta = self.random_number();
            Some((r * theta.cos(), r * theta.sin()))
        }
    }
}

// This is the algorithm developed by Pixar for their RenderMan engine.
// All the random looking equations are part of the hash function they developed.
// See https://graphics.pixar.com/library/MultiJitteredSampling/paper.pdf for details.
#[derive(Clone, Debug)]
pub struct CorrelatedMultiJitteredSampler {
    p: u32,  // Pattern seed
    m: u32,  // Width
    n: u32,  // Height
    s: u32,  // Sample index
}

impl CorrelatedMultiJitteredSampler {
    pub fn new(p: u32, m: u32, n: u32) -> CorrelatedMultiJitteredSampler {
        CorrelatedMultiJitteredSampler{ p, m, n, s: 0 }
    }

    pub fn random(m: u32, n: u32) -> CorrelatedMultiJitteredSampler {
        let p: u32 = rand::thread_rng().gen();
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
            i ^= p;             i = i.wrapping_mul(0xe170_893d);
            i ^= p       >> 16;
            i ^= (i & w) >> 4;
            i ^= p       >> 8;  i = i.wrapping_mul(0x0929_eb3f);
            i ^= p       >> 23;
            i ^= (i & w) >> 1;  i = i.wrapping_mul(1 | p >> 27);
                                i = i.wrapping_mul(0x6935_fa69);
            i ^= (i & w) >> 11; i = i.wrapping_mul(0x74dc_b303);
            i ^= (i & w) >> 2;  i = i.wrapping_mul(0x9e50_1cc3);
            i ^= (i & w) >> 2;  i = i.wrapping_mul(0xc860_a3df);
            i &= w;
            i ^= i       >> 5;
        }

        (i + p) % l
    }

    fn rand_float(mut i: u32, p: u32) -> f64 {
        i ^= p;
        i ^= i >> 17;
        i ^= i >> 10; i = i.wrapping_mul(0xb365_34e5);
        i ^= i >> 12;
        i ^= i >> 21; i = i.wrapping_mul(0x93fc_4795);
        i ^= 0xdf6e_307f;
        i ^= i >> 17; i = i.wrapping_mul(1 | p >> 18);
        (i as f64) * (1.0 / 4_294_967_808.0)
    }

    // s = sample index
    // m, n = sample dimensions (m x n = N = number of samples total)
    // p = pattern index (like a seed)
    fn cmj(s: u32, m: u32, n: u32, p: u32) -> (f64, f64) {
        let ps: u32 = CorrelatedMultiJitteredSampler::permute(s, m*n, p.wrapping_mul(0xa73b_d290));
        let sx: f64 = CorrelatedMultiJitteredSampler::permute(ps % m, m, p.wrapping_mul(0xa511_e9b3)) as f64;
        let sy: f64 = CorrelatedMultiJitteredSampler::permute(ps / m, n, p.wrapping_mul(0x63d8_3595)) as f64;
        let jx: f64 = CorrelatedMultiJitteredSampler::rand_float(s, p.wrapping_mul(0xa399_d265));
        let jy: f64 = CorrelatedMultiJitteredSampler::rand_float(s, p.wrapping_mul(0x711a_d6a5));
        let x: f64 = (((s % m) as f64) + (sy + jx) / (n as f64)) / (m as f64);
        let y: f64 = (((s / m) as f64) + (sx + jy) / (m as f64)) / (n as f64);
        (x, y)
    }
}

impl SquareSampler for CorrelatedMultiJitteredSampler {
    fn next_sample_square(&mut self) -> Option<(f64, f64)> {
        if self.s >= self.m * self.n {
            None
        } else {
            let sample = CorrelatedMultiJitteredSampler::cmj(self.s, self.m, self.n, self.p);
            self.s += 1;
            Some(sample)
        }
    }
}

impl DiskSampler for CorrelatedMultiJitteredSampler {
    fn next_sample_disk(&mut self) -> Option<(f64, f64)> {
        if self.s >= self.m * self.n {
            None
        } else {
            // Sample the square and then map to the disk.
            // Only works if m ~= n
            let (x, y) = CorrelatedMultiJitteredSampler::cmj(self.s, self.m, self.n, self.p);
            self.s += 1;

            let theta = 2.0 * PI * x;
            let r = y.sqrt();
            Some((r * theta.cos(), r * theta.sin()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::sampling::*;

    fn is_in_unit_disk(x: f64, y: f64) -> bool {
        x * x + y * y <= 1.0
    }

    fn is_in_unit_square(x: f64, y: f64) -> bool {
        x >= 0.0 && x <= 1.0 && y >= 0.0 && y <= 1.0
    }

    #[test]
    fn test_uniform_disk() {
        let pattern = UniformSampler::new(0, 2, 3).pattern::<Disk>();

        // Hard-code expected values to ensure that the seed is stable across test runs.
        let expected = vec![
            (0.27099483228008736, 0.3541936719985136),
            (0.3199761067608373, 0.0034989080440785106),
            (0.00018841126354844867, 0.00005758516026271694),
            (0.4444775002102809, 0.35185244122547316),
            (0.4535960961658139, 0.19369113347312825),
            (0.34648254086248437, 0.32805505516760064),
        ];

        let actual = pattern.collect::<Vec<(f64, f64)>>();
        assert_eq!(actual, expected);
        for (x, y) in actual {
            assert_eq!(is_in_unit_disk(x, y), true);
        }
    }

    #[test]
    fn test_uniform_square() {
        let pattern = UniformSampler::new(0, 2, 3).pattern::<Square>();

        // Hard-code expected values to ensure that the seed is stable across test runs.
        let expected = vec![
            (0.44597237179706917, 0.9176988201469243),
            (0.3199952363009857, 0.010934468302049138),
            (0.00019701485962841936, 0.29661887443159096),
            (0.5668865747127068, 0.6696035669674656),
            (0.49321970119103264, 0.4035738877175896),
            (0.47714805914259006, 0.7580862631517262),
        ];

        let actual = pattern.collect::<Vec<(f64, f64)>>();
        assert_eq!(actual, expected);
        for (x, y) in actual {
            assert_eq!(is_in_unit_square(x, y), true);
        }
    }

    #[test]
    fn test_cmj_disk() {
        let pattern = CorrelatedMultiJitteredSampler::new(0, 2, 3).pattern::<Disk>();

        // Hard-code expected values to ensure that the seed is stable across test runs.
        let expected = vec![
            (0.23288271976954444, 0.3020407408384594),
            (-0.41231103969933375, -0.00884025347340132),
            (-0.01713192576599384, 0.6485187612468607),
            (0.38017576583611823, -0.7185092520948844),
            (-0.7994905690029475, 0.35683991876591936),
            (0.9139355167587502, -0.3308265058968712)
        ];

        let actual = pattern.collect::<Vec<(f64, f64)>>();
        assert_eq!(actual, expected);

        // Now test all values are within bounds using a very large pattern.
        let large_pattern = CorrelatedMultiJitteredSampler::new(0, 100, 100).pattern::<Disk>();
        for (x, y) in large_pattern {
            assert_eq!(is_in_unit_disk(x, y), true);
        }
    }

    #[test]
    fn test_cmj_square() {
        let pattern = CorrelatedMultiJitteredSampler::new(0, 2, 3).pattern::<Square>();

        // Hard-code expected values to ensure that the seed is stable across test runs.
        let expected = vec![
            (0.14546297029350555, 0.14546297029350555),
            (0.5034118768727529, 0.17007854353941954),
            (0.25420341990294765, 0.4208700865696143),
            (0.8274558249416957, 0.660789158275029),
            (0.43318656421619134, 0.7665198975495247),
            (0.9447243057970165, 0.9447243057970164)
        ];

        let actual = pattern.collect::<Vec<(f64, f64)>>();
        assert_eq!(actual, expected);
        
        // Now test all values are within bounds using a very large pattern.
        let large_pattern = CorrelatedMultiJitteredSampler::new(0, 100, 100).pattern::<Square>();
        for (x, y) in large_pattern {
            assert_eq!(is_in_unit_square(x, y), true);
        }
    }
}
