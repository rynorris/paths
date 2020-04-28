use std::ops;

use crate::vector::Vector3;

// 3x3 Matrix
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3 {
    components: [f64; 9],
}

// Methods
impl Matrix3 {
    #[inline]
    pub fn get(&self, r: usize, c: usize) -> f64 {
        self.components[r * 3 + c]
    }

    #[inline]
    fn set(&mut self, r: usize, c: usize, v: f64) {
        self.components[r * 3 + c] = v;
    }
}

// Constructors.
impl Matrix3 {
    pub fn zero() -> Matrix3 {
        Matrix3{ components: [0.0; 9] }
    }

    pub fn rotation(yaw: f64, pitch: f64, roll: f64) -> Matrix3 {
        let m_pitch = Matrix3::rotation_x(pitch);
        let m_yaw = Matrix3::rotation_y(yaw);
        let m_roll = Matrix3::rotation_z(roll);
        m_pitch * m_yaw * m_roll
    }

    pub fn rotation_x(angle: f64) -> Matrix3 {
        let sin = angle.sin();
        let cos = angle.cos();
        Matrix3{ 
            components: [
                1.0, 0.0, 0.0,
                0.0, cos, -sin,
                0.0, sin, cos,
            ] 
        }
    }

    pub fn rotation_y(angle: f64) -> Matrix3 {
        let sin = angle.sin();
        let cos = angle.cos();
        Matrix3{ 
            components: [
                cos, 0.0, sin,
                0.0, 1.0, 0.0,
                -sin, 0.0, cos,
            ] 
        }
    }

    pub fn rotation_z(angle: f64) -> Matrix3 {
        let sin = angle.sin();
        let cos = angle.cos();
        Matrix3{ 
            components: [
                cos, -sin, 0.0,
                sin, cos, 0.0,
                0.0, 0.0, 1.0,
            ] 
        }
    }
}

impl ops::Mul<Matrix3> for Matrix3 {
    type Output = Matrix3;

    fn mul(self, other: Matrix3) -> Matrix3 {
        let mut out = Matrix3::zero();
        for r in 0 .. 3 {
            for c in 0 .. 3 {
                let mut v = 0.0;
                for k in 0 .. 3 {
                    v += self.get(r, k) * other.get(k, c);
                }
                out.set(r, c, v);
            }
        }
        out
    }
}

impl ops::Mul<Vector3> for Matrix3 {
    type Output = Vector3;

    fn mul(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.get(0, 0) * other.x + self.get(0, 1) * other.y + self.get(0, 2) * other.z,
            y: self.get(1, 0) * other.x + self.get(1, 1) * other.y + self.get(1, 2) * other.z,
            z: self.get(2, 0) * other.x + self.get(2, 1) * other.y + self.get(2, 2) * other.z,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::matrix::Matrix3;

    #[test]
    fn test_matrix_multiply() {
        let m1 = Matrix3 {
            components: [
                1.0, 2.0, 3.0,
                4.0, 5.0, 6.0,
                7.0, 8.0, 9.0,
            ]
        };

        let m2 = Matrix3 {
            components: [
                9.0, 8.0, 7.0,
                6.0, 5.0, 4.0,
                3.0, 2.0, 1.0,
            ]
        };

        let expected = Matrix3 {
            components: [
                30.0, 24.0, 18.0,
                84.0, 69.0, 54.0,
                138.0, 114.0, 90.0,
            ]
        };

        assert_eq!(m1 * m2, expected);
    }
}
