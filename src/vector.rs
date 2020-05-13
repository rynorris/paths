use std::ops;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub fn zero() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }

    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }

    pub fn dot(&self, other: Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn magnitude(&self) -> f64 {
        self.dot(*self)
    }

    pub fn max(&self) -> f64 {
        f64::max(self.x, f64::max(self.y, self.z))
    }

    pub fn min(&self) -> f64 {
        f64::min(self.x, f64::min(self.y, self.z))
    }

    pub fn normed(&self) -> Vector3 {
        (*self) / self.magnitude().sqrt()
    }

    pub fn cross(&self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.y * other.z - self.z * other.y,
            y: -(self.x * other.z - self.z * other.x),
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn form_basis(&self) -> (Vector3, Vector3, Vector3) {
        let j = *self;

        let i = if j.x.abs() == 0.0 {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            j.cross(Vector3::new(0.0, 1.0, 0.0)).normed()
        };
        let k = i.cross(j);
        (i, j, k)
    }

    pub fn invert(&self) -> Vector3 {
        Vector3::new(1.0 / self.x, 1.0 / self.y, 1.0 / self.z)
    }

    pub fn componentwise_max(v1: Vector3, v2: Vector3) -> Vector3 {
        Vector3::new(
            f64::max(v1.x, v2.x),
            f64::max(v1.y, v2.y),
            f64::max(v1.z, v2.z),
        )
    }

    pub fn componentwise_min(v1: Vector3, v2: Vector3) -> Vector3 {
        Vector3::new(
            f64::min(v1.x, v2.x),
            f64::min(v1.y, v2.y),
            f64::min(v1.z, v2.z),
        )
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

impl ops::AddAssign for Vector3 {
    fn add_assign(&mut self, other: Vector3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
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

impl ops::Mul<Vector3> for Vector3 {
    type Output = Vector3;

    fn mul(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
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
