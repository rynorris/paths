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
