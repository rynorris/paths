use std::sync::{Arc};

use crossbeam::channel;
use threadpool::ThreadPool;

use crate::camera::Image;
use crate::pixels::Estimator;
use crate::scene::Scene;
use crate::worker;

pub struct Renderer {
    pub scene: Arc<Scene>,
    estimator: Estimator,
    pool: ThreadPool,
    num_workers: usize,
}

impl Renderer {
    pub fn new(scene: Arc<Scene>, num_workers: usize) -> Renderer {
        let estimator = Estimator::new(scene.camera.width as usize, scene.camera.height as usize);
        let pool = ThreadPool::new(num_workers);
        Renderer{ scene, estimator, pool, num_workers }
    }

    pub fn render(&self) -> Image {
        self.estimator.render()
    }

    pub fn trace_full_pass(&mut self) {
        let (request_tx, request_rx) = channel::unbounded::<worker::RenderRequest>();
        let (result_tx, result_rx) = channel::unbounded::<worker::RenderResult>();

        // Spin up 4 workers.
        for _ in 0..self.num_workers {
            let worker = worker::Worker::new(request_rx.clone(), result_tx.clone(), self.scene.clone());
            self.pool.execute(move|| worker.run_forever());
        }

        // Send requests off for slices of the image.
        let pattern_size: (u32, u32) = (5, 5);
        for x in 0 .. self.scene.camera.width {
            let request = worker::RenderRequest{
                top_left: (x, 0),
                bottom_right: (x, self.scene.camera.height - 1),
                pattern_size,
            };

            match request_tx.send(request) {
                Ok(_) => (),
                Err(err) => panic!(err),
            }
        }

        let num_results = self.scene.camera.width * pattern_size.0 * pattern_size.1;

        // Receive results.
        let mut n = 0;
        result_rx.iter().take((num_results) as usize).for_each(|result| {
            n += 1;
            result.samples.iter().for_each(|(x, y, colour)| {
                self.estimator.update_pixel(*x as usize, *y as usize, *colour);
            });
        });

        if self.pool.panic_count() > 0 {
            panic!("{} rendering threads panicked while rendering.", self.pool.panic_count());
        }
    }

    pub fn reset(&mut self) {
        self.estimator = Estimator::new(self.scene.camera.width as usize, self.scene.camera.height as usize);
    }

}
