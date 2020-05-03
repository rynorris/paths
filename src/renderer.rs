use std::sync::{Arc};

use crossbeam::channel;
use threadpool::ThreadPool;

use crate::camera::{Camera, Image};
use crate::matrix::Matrix3;
use crate::pixels::Estimator;
use crate::scene::Scene;
use crate::vector::Vector3;
use crate::worker;

const PREVIEW_GRID_SIZE: usize = 8;

pub struct Renderer {
    width: u32,
    height: u32,
    estimator: Estimator,
    epoch: u64,
    pool: ThreadPool,
    request_tx: channel::Sender<worker::RenderRequest>,
    result_rx: channel::Receiver<worker::RenderResult>,
    control_txs: Vec<channel::Sender<worker::ControlMessage>>,
    
    // Request iteration state.
    block_num: u32,
    quick_render: bool,

    // Stats.
    num_rays_cast: u64,
}

impl Renderer {
    pub fn new(camera: Camera, scene: Arc<Scene>, num_workers: usize) -> Renderer {
        let estimator = Estimator::new(camera.width as usize, camera.height as usize, PREVIEW_GRID_SIZE);
        let pool = ThreadPool::new(num_workers);

        let (request_tx, request_rx) = channel::bounded::<worker::RenderRequest>(200);
        let (result_tx, result_rx) = channel::unbounded::<worker::RenderResult>();
        let mut control_txs: Vec<channel::Sender<worker::ControlMessage>> = Vec::with_capacity(num_workers);

        // Spin up 4 workers.
        for _ in 0..num_workers {
            let (control_tx, control_rx) = channel::unbounded::<worker::ControlMessage>();
            let mut worker = worker::Worker::new(
                request_rx.clone(),
                result_tx.clone(),
                control_rx.clone(),
                scene.clone(),
                camera.clone(),
            );
            control_txs.push(control_tx);
            pool.execute(move|| worker.run_forever());
        }

        Renderer{
            width: camera.width,
            height: camera.height,
            estimator,
            epoch: 0,
            pool,
            request_tx,
            result_rx,
            control_txs,
            block_num: 0,
            quick_render: true,
            num_rays_cast: 0,
        }
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
            // Ignore results from a different epoch.
            if result.epoch != self.epoch {
                return;
            }

            self.num_rays_cast += result.samples.len() as u64;
            result.samples.iter().for_each(|(x, y, colour)| {
                self.estimator.update_pixel(*x as usize, *y as usize, *colour);
            });
        });

        if self.pool.panic_count() > 0 {
            panic!("{} rendering threads panicked while rendering.", self.pool.panic_count());
        }
    }

    pub fn reorient_camera(&mut self, orientation: Matrix3) {
        let epoch = self.new_epoch();
        self.broadcast_command(worker::ControlMessage::new()
            .cmd(worker::Command::ReorientCamera(orientation))
            .cmd(worker::Command::SetEpoch(epoch))
        );
        self.request_preview();
    }

    pub fn reposition_camera(&mut self, location: Vector3) {
        let epoch = self.new_epoch();
        self.broadcast_command(worker::ControlMessage::new()
            .cmd(worker::Command::RepositionCamera(location))
            .cmd(worker::Command::SetEpoch(epoch))
        );
        self.request_preview();
    }

    pub fn reset(&mut self) {
        let epoch = self.new_epoch();
        self.broadcast_command(worker::ControlMessage::new()
            .cmd(worker::Command::SetEpoch(epoch))
        );
        self.request_preview();
    }

    pub fn shutdown(&mut self) {
        println!("Signaling workers to shut down.");
        self.broadcast_command(worker::ControlMessage::new()
            .cmd(worker::Command::Shutdown)
        );

        // Wait for workers to shutdown.
        println!("Waiting for workers to close...");
        self.pool.join();

        println!("Shutdown complete!");
    }

    fn new_epoch(&mut self) -> u64 {
        self.block_num = 0;
        self.num_rays_cast = 0;
        self.quick_render = true;
        self.estimator = Estimator::new(self.width as usize, self.height as usize, PREVIEW_GRID_SIZE);
        self.epoch += 1;
        self.epoch
    }

    fn request_preview(&mut self) {
        for x in (0..self.width).step_by(PREVIEW_GRID_SIZE) {
            for y in (0..self.height).step_by(PREVIEW_GRID_SIZE) {
                let req = worker::RenderRequest{
                    epoch: self.epoch,
                    top_left: (x, y),
                    bottom_right: (x, y),
                    pattern_size: (1, 1),
                };
                self.request_tx.send(req).expect("Can send request.");
            }
        }
    }

    fn next_request(&mut self) -> worker::RenderRequest {
        // Start from the center, since that's the most interesting part of the image probably.
        let w = self.width;
        let n = self.block_num;
        let x = if n % 2 == 0 { (w + n) / 2 } else { (w - n) / 2 };

        // Want the image to appear quickly after a reset.
        // So use a small pattern size for the first few samples after a new epoch.
        let pattern_size: (u32, u32) = if self.quick_render {
            (1, 1)
        } else {
            (5, 5)
        };

        self.block_num += 1;
        if self.block_num >= self.width {
            self.block_num = 0;
            self.quick_render = false;
        }

        worker::RenderRequest{
            epoch: self.epoch,
            top_left: (x, 0),
            bottom_right: (x, self.height - 1),
            pattern_size,
        }
    }

    fn broadcast_command(&self, msg: worker::ControlMessage) {
        println!("Sending command to workers: {:?}", msg);
        self.control_txs.iter().for_each(|tx| {
            tx.send(msg.clone()).expect("Should succeed to send control messages.");
        });
    }
}
