use std::ops;

#[derive(Clone, Copy, Debug)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn dot(&self, other: Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn magnitude(&self) -> f64 {
        self.dot(*self)
    }

    pub fn normed(&self) -> Vector3 {
        (*self) / self.magnitude()
    }
}

impl ops::Add<Vector3> for Vector3 {
    type Output = Vector3;

    fn add(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl ops::Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl <T : Into<f64>> ops::Mul<T> for Vector3 {
    type Output = Vector3;

    fn mul(self, v: T) -> Vector3 {
        let v_f64: f64 = v.into();
        Vector3 {
            x: self.x * v_f64,
            y: self.y * v_f64,
            z: self.z * v_f64,
        }
    }
}

impl <T : Into<f64>> ops::Div<T> for Vector3 {
    type Output = Vector3;

    fn div(self, v: T) -> Vector3 {
        let v_f64: f64 = v.into();
        Vector3 {
            x: self.x / v_f64,
            y: self.y / v_f64,
            z: self.z / v_f64,
        }
    }
}
