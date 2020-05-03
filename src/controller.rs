use crate::camera::Image;
use crate::matrix::Matrix3;
use crate::renderer::Renderer;
use crate::vector::Vector3;

pub struct Controller {
    renderer: Renderer,
    location: Vector3,
    orientation: Matrix3,
}

impl Controller {
    pub fn new(renderer: Renderer, location: Vector3, orientation: Matrix3) -> Controller {
        Controller{ renderer, location, orientation, }
    }

    pub fn update(&mut self) {
        self.renderer.fill_request_queue();
        self.renderer.drain_result_queue();
    }

    pub fn render(&mut self) -> Image {
        self.renderer.render()
    }

    pub fn move_camera(&mut self, v: Vector3) {
        if v.x == 0.0 && v.y == 0.0 && v.z == 0.0 {
            return;
        }

        let movement = self.orientation * v;
        self.location = self.location + movement;
        self.renderer.reposition_camera(self.location);
    }

    pub fn rotate(&mut self, yaw: f64, pitch: f64, roll: f64) {
        let rot = Matrix3::rotation(yaw, pitch, roll);
        self.orientation = self.orientation * rot;
        self.renderer.reorient_camera(self.orientation);
    }

    pub fn reset(&mut self) {
        self.renderer.reset();
    }

    pub fn shutdown(&mut self) {
        self.renderer.shutdown();
    }

    pub fn num_rays_cast(&self) -> u64{
        self.renderer.num_rays_cast()
    }
}
