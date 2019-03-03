use std::ops;

#[derive(Clone, Copy, Debug)]
pub struct Colour {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Colour {
    pub const BLACK: Colour = Colour { r: 0.0, g: 0.0, b: 0.0 };
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

impl ops::AddAssign<Colour> for Colour {
    fn add_assign(&mut self, other: Colour) {
        self.r += other.r;
        self.g += other.g;
        self.b += other.b;
    }
}

impl ops::Div<u32> for Colour {
    type Output = Colour;

    fn div(self, x: u32) -> Colour {
        Colour {
            r: self.r / (x as f64),
            g: self.g / (x as f64),
            b: self.b / (x as f64),
        }
    }
}
