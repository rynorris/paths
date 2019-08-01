use std::ops;

use rand;
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct Colour {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Colour {
    pub const BLACK: Colour = Colour { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE: Colour = Colour { r: 1.0, g: 1.0, b: 1.0 };

    pub fn random() -> Colour {
        let mut rng = rand::thread_rng();
        Colour {
            r: rng.gen(),
            g: rng.gen(),
            b: rng.gen(),
        }
    }

    pub fn rgb(r: f64, g: f64, b: f64) -> Colour {
        Colour{ r, g, b }
    }

    pub fn to_bytes(&self) -> (u8, u8, u8) {
        (
            Colour::component_to_byte(self.r),
            Colour::component_to_byte(self.g),
            Colour::component_to_byte(self.b),
            )
    }

    pub fn max(&self) -> f64 {
        let w = if self.r > self.g { self.r } else { self.g };
        if w > self.b { w } else { self.b }
    }

    pub fn clamped(self) -> Colour {
        Colour {
            r: 0f64.max(1f64.min(self.r)),
            g: 0f64.max(1f64.min(self.g)),
            b: 0f64.max(1f64.min(self.b)),
        }
    }

    fn component_to_byte(x: f64) -> u8 {
        let rounded = (x * 256.0) as i16;
        if rounded >= 256 {
            255
        } else if rounded <= 0 {
            0
        } else {
            rounded as u8
        }
    }
}

impl ops::Add<Colour> for Colour {
    type Output = Colour;

    fn add(self, other: Colour) -> Colour {
        Colour {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl ops::Mul<Colour> for Colour {
    type Output = Colour;

    fn mul(self, other: Colour) -> Colour {
        Colour {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
        }
    }
}

impl <T : Into<f64>> ops::Mul<T> for Colour {
    type Output = Colour;

    fn mul(self, x: T) -> Colour {
        let x_f64 = x.into();
        Colour {
            r: self.r * x_f64,
            g: self.g * x_f64,
            b: self.b * x_f64,
        }
    }
}

impl ops::AddAssign<Colour> for Colour {
    fn add_assign(&mut self, other: Colour) {
        self.r += other.r;
        self.g += other.g;
        self.b += other.b;
    }
}

impl <T : Into<f64>> ops::Div<T> for Colour {
    type Output = Colour;

    fn div(self, x: T) -> Colour {
        let x_f64 = x.into();
        Colour {
            r: self.r / x_f64,
            g: self.g / x_f64,
            b: self.b / x_f64,
        }
    }
}
