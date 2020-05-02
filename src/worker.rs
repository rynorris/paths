use std::sync::Arc;

use crossbeam::channel;
use crossbeam::channel::select;

use crate::camera::Camera;
use crate::colour::Colour;
use crate::scene::Scene;
use crate::sampling::{CorrelatedMultiJitteredSampler, Disk, IntoPattern, Square};
use crate::trace::trace_ray;

pub struct Worker {
    request_rx: channel::Receiver<RenderRequest>,
    result_tx: channel::Sender<RenderResult>,
    control_rx: channel::Receiver<ControlMessage>,
    scene: Arc<Scene>,
    camera: Camera,
    epoch: u64,
    is_running: bool,
}

impl Worker {
    pub fn new(
        request_rx: channel::Receiver<RenderRequest>,
        result_tx: channel::Sender<RenderResult>,
        control_rx: channel::Receiver<ControlMessage>,
        scene: Arc<Scene>
    ) -> Worker {
        let camera = scene.camera.clone();
        Worker{
            request_rx, result_tx, control_rx,
            scene,
            camera,
            epoch: 0,
            is_running: true,
        }
    }

    pub fn run_forever(&mut self) {
        while self.is_running {
            select! {
                recv(self.request_rx) -> res => match res {
                    Ok(req) => self.handle_render_req(req),
                    Err(err) => panic!("Error when receiving render request: {}", err),
                },
                recv(self.control_rx) -> res => match res {
                    Ok(msg) => self.handle_control_msg(msg),
                    Err(err) => panic!("Error when receiving control message: {}", err),
                },
            }
        }
    }

    fn handle_render_req(&self, req: RenderRequest) {
        // Ignore if from a different epoch.
        if req.epoch != self.epoch {
            return;
        }

        let (m, n) = req.pattern_size;
        let sensor_pattern = CorrelatedMultiJitteredSampler::random(m, n).pattern::<Square>();
        let lens_pattern = CorrelatedMultiJitteredSampler::random(m, n).pattern::<Disk>();
        let patterns = sensor_pattern.zip(lens_pattern);

        patterns.for_each(|(sensor_sample, lens_sample)| {
            let samples = req.iter_pixels().map(|(x, y)| {
                let (ray, weight) = self.camera.get_ray_for_pixel(x, y, sensor_sample, lens_sample);
                let colour = trace_ray(&self.scene, ray) * weight;
                (x, y, colour)
            }).collect();

            match self.result_tx.send(RenderResult{ epoch: self.epoch, samples }) {
                Ok(_) => (),
                Err(err) => {
                    panic!("Failed to send samples to main thread: {}", err);
                },
            }
        });
    }

    fn handle_control_msg(&mut self, msg: ControlMessage) {
        msg.commands.iter().for_each(|cmd| {
            match cmd {
                Command::Shutdown => self.is_running = false,
                Command::SetEpoch(epoch) => self.epoch = *epoch,
                Command::ReorientCamera(yaw, pitch, roll) => self.camera.set_orientation(*yaw, *pitch, *roll),
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct ControlMessage {
    pub commands: Vec<Command>,
}

impl ControlMessage {
    pub fn new() -> ControlMessage {
        ControlMessage{ commands: Vec::new() }
    }

    pub fn cmd(mut self, cmd: Command) -> ControlMessage {
        self.commands.push(cmd);
        self
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Shutdown,
    SetEpoch(u64),
    ReorientCamera(f64, f64, f64),
}


// API structs.
#[derive(Clone, Copy, Debug)]
pub struct RenderRequest {
    pub epoch: u64,
    pub top_left: (u32, u32),
    pub bottom_right: (u32, u32),
    pub pattern_size: (u32, u32),
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
    pub epoch: u64,
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
