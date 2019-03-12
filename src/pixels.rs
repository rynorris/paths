use std::ops;

use crate::camera::Image;
use crate::colour::Colour;

struct MeanVec<T> {
    sums: Vec<T>,
    counts: Vec<u32>,
}

impl <T : Copy + ops::AddAssign<T> + ops::Div<u32, Output = T>> MeanVec<T> {
    pub fn new(size: usize, initial: T) -> MeanVec<T> {
        MeanVec {
            sums: vec![initial; size],
            counts: vec![0; size],
        }
    }

    pub fn update(&mut self, ix: usize, value: T) {
        self.sums[ix] += value;
        self.counts[ix] += 1;
    }

    pub fn get(&self, ix: usize) -> T {
        self.sums[ix] / self.counts[ix]
    }
}

pub struct Estimator {
    width: usize,
    height: usize,
    means: MeanVec<Colour>,
}

impl Estimator {
    pub fn new(width: usize, height: usize) -> Estimator {
        Estimator {
            width, height,
            means: MeanVec::new(width * height, Colour::BLACK),
        }
    }

    pub fn update_pixel(&mut self, x: usize, y: usize, colour: Colour) {
        let reflected_y = self.height - y - 1;
        self.means.update(x + (reflected_y * self.width), colour);
    }

    pub fn render(&self) -> Image {
        let mut buffer = Vec::with_capacity(self.width * self.height);
        for ix in 0 .. self.width * self.height {
            buffer.push(self.means.get(ix));
        }
        Image {
            width: self.width as u32,
            height: self.height as u32,
            pixels: buffer,
        }
    }
}
