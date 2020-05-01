use std::sync::Arc;

use crossbeam::channel;

use crate::colour::Colour;
use crate::scene::Scene;
use crate::trace::trace_ray;

pub struct Worker {
    request_rx: channel::Receiver<RenderRequest>,
    result_tx: channel::Sender<RenderResult>,
    scene: Arc<Scene>,
}

impl Worker {
    pub fn new(
        request_rx: channel::Receiver<RenderRequest>,
        result_tx: channel::Sender<RenderResult>,
        scene: Arc<Scene>
    ) -> Worker {
        Worker{ request_rx, result_tx, scene }
    }

    pub fn run_forever(&self) {
        self.request_rx.iter().for_each(|req| {
            let mut camera = self.scene.camera.clone();
            for _ in 0..req.num_samples {
                camera.init_bundle();
                let samples = req.iter_pixels().map(|(x, y)| {
                    let (ray, weight) = camera.get_ray_for_pixel(x, y);
                    let colour = trace_ray(&self.scene, ray) * weight;
                    (x, y, colour)
                }).collect();

                match self.result_tx.send(RenderResult{ samples }) {
                    Ok(_) => (),
                    Err(err) => {
                        panic!("Failed to send samples to main thread: {:?}", err);
                    },
                }
            }
        });
    }
}

// API structs.
#[derive(Clone, Copy, Debug)]
pub struct RenderRequest {
    pub top_left: (u32, u32),
    pub bottom_right: (u32, u32),
    pub num_samples: u32,
}

impl RenderRequest {
    pub fn iter_pixels(self) -> PixelGridIter {
        PixelGridIter::new(
            self.top_left.0,
            self.top_left.1,
            self.bottom_right.0 - self.top_left.0 + 1,
            self.bottom_right.1 - self.top_left.1 + 1,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PixelGridIter {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    pos: u32,
}

impl PixelGridIter {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> PixelGridIter {
        PixelGridIter{ x, y, w, h, pos: 0 }
    }
}

impl Iterator for PixelGridIter {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        if self.pos >= self.w * self.h {
            None
        } else {
            let x_offset = self.pos % self.w;
            let y_offset = self.pos / self.w;

            let (x, y) = (self.x + x_offset, self.y + y_offset);

            self.pos += 1;
            Some((x, y))
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderResult {
    pub samples: Vec<(u32, u32, Colour)>,
}

#[cfg(test)]
mod test {
    use crate::worker;

    #[test]
    fn test_pixel_grid_iter_x() {
        let mut grid = worker::PixelGridIter::new(0, 0, 3, 1);
        assert_eq!(grid.next(), Some((0, 0)));
        assert_eq!(grid.next(), Some((1, 0)));
        assert_eq!(grid.next(), Some((2, 0)));
        assert_eq!(grid.next(), None);
    }

    #[test]
    fn test_pixel_grid_iter_y() {
        let mut grid = worker::PixelGridIter::new(0, 0, 1, 3);
        assert_eq!(grid.next(), Some((0, 0)));
        assert_eq!(grid.next(), Some((0, 1)));
        assert_eq!(grid.next(), Some((0, 2)));
        assert_eq!(grid.next(), None);
    }

    #[test]
    fn test_pixel_grid_iter_square() {
        let mut grid = worker::PixelGridIter::new(0, 0, 2, 2);
        assert_eq!(grid.next(), Some((0, 0)));
        assert_eq!(grid.next(), Some((1, 0)));
        assert_eq!(grid.next(), Some((0, 1)));
        assert_eq!(grid.next(), Some((1, 1)));
        assert_eq!(grid.next(), None);
    }

    #[test]
    fn test_pixel_grid_iter_offset_square() {
        let mut grid = worker::PixelGridIter::new(4, 4, 2, 2);
        assert_eq!(grid.next(), Some((4, 4)));
        assert_eq!(grid.next(), Some((5, 4)));
        assert_eq!(grid.next(), Some((4, 5)));
        assert_eq!(grid.next(), Some((5, 5)));
        assert_eq!(grid.next(), None);
    }
}
