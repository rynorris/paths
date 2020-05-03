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

    pub fn count(&self, ix: usize) -> u32 {
        self.counts[ix]
    }
}

pub struct Estimator {
    width: usize,
    height: usize,
    preview_grid_size: usize,
    means: MeanVec<Colour>,
}

impl Estimator {
    pub fn new(width: usize, height: usize, preview_grid_size: usize) -> Estimator {
        Estimator {
            width, height,
            preview_grid_size,
            means: MeanVec::new(width * height, Colour::BLACK),
        }
    }

    pub fn update_pixel(&mut self, x: usize, y: usize, colour: Colour) {
        self.means.update(x + y * self.width, colour);
    }

    pub fn render(&self) -> Image {
        let mut buffer = Vec::with_capacity(self.width * self.height);
        for ix in 0 .. self.width * self.height {
            if self.means.count(ix) == 0 {
                // No samples, fill using preview grid.
                let x = ix % self.width;
                let y = ix / self.width;
                let grid_size = self.preview_grid_size;

                if x % grid_size == 0 && y % grid_size == 0 {
                    buffer.push(self.means.get(ix));
                } else {
                    let grid_x = x - (x % grid_size);
                    let grid_y = y - (y % grid_size);
                    let grid_ix = grid_x + grid_y * self.width;
                    buffer.push(self.means.get(grid_ix));
                }
            } else {
                buffer.push(self.means.get(ix));
            }
        }
        Image {
            width: self.width as u32,
            height: self.height as u32,
            pixels: buffer,
        }
    }
}
