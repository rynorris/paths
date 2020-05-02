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
    request_tx: channel::Sender<worker::RenderRequest>,
    result_rx: channel::Receiver<worker::RenderResult>,
    
    // Request iteration state.
    cur_x: u32,

    // Stats.
    num_rays_cast: u64,
}

impl Renderer {
    pub fn new(scene: Arc<Scene>, num_workers: usize) -> Renderer {
        let estimator = Estimator::new(scene.camera.width as usize, scene.camera.height as usize);
        let pool = ThreadPool::new(num_workers);

        let (request_tx, request_rx) = channel::bounded::<worker::RenderRequest>(200);
        let (result_tx, result_rx) = channel::unbounded::<worker::RenderResult>();

        // Spin up 4 workers.
        for _ in 0..num_workers {
            let worker = worker::Worker::new(request_rx.clone(), result_tx.clone(), scene.clone());
            pool.execute(move|| worker.run_forever());
        }

        Renderer{ scene, estimator, pool, request_tx, result_rx, cur_x: 0, num_rays_cast: 0 }
    }

    pub fn render(&self) -> Image {
        self.estimator.render()
    }

    pub fn num_rays_cast(&self) -> u64 {
        self.num_rays_cast
    }

    pub fn fill_request_queue(&mut self) {
        if self.request_tx.is_empty() {
            println!("[WARN] Request queue was empty");
        }

        while !self.request_tx.is_full() {
            let request = self.next_request();
            match self.request_tx.send(request) {
                Ok(_) => (),
                Err(err) => panic!(err),
            }
        }
    }

    pub fn drain_result_queue(&mut self) {
        let results = self.result_rx.try_iter().collect::<Vec<worker::RenderResult>>();
        results.iter().for_each(|result| {
            self.num_rays_cast += result.samples.len() as u64;
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

    fn next_request(&mut self) -> worker::RenderRequest {
        let pattern_size: (u32, u32) = (4, 4);
        let x = self.cur_x;

        self.cur_x += 1;
        if self.cur_x >= self.scene.camera.width {
            self.cur_x = 0;
        }

        worker::RenderRequest{
            top_left: (x, 0),
            bottom_right: (x, self.scene.camera.height - 1),
            pattern_size,
        }
    }
}
