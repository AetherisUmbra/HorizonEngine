#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4 {
    pub data: [f32; 16],
}

impl Matrix4x4 {
    pub fn new(data: [f32; 16]) -> Self {
        Matrix4x4 { data }
    }

    pub fn identity() -> Self {
        Matrix4x4 {
            data: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn transpose(&self) -> Self {
        let mut transposed = [0.0; 16];
        for i in 0..4 {
            for j in 0..4 {
                transposed[i * 4 + j] = self.data[j * 4 + i];
            }
        }
        Matrix4x4 { data: transposed }
    }

    pub fn multiply(&self, other: &Matrix4x4) -> Self {
        let mut result = [0.0; 16];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i * 4 + j] += self.data[i * 4 + k] * other.data[k * 4 + j];
                }
            }
        }
        Matrix4x4 { data: result }
    }
}

use std::ops::Mul;

impl Mul for Matrix4x4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        self.multiply(&other)
    }
}