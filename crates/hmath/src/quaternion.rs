use crate::vector::Vector3;
use num_traits::Float;
use std::f64::consts::PI;
use std::ops::Mul;

#[derive(Debug, Clone, Copy)]
pub struct Quaternion<T> {
    pub w: T,
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Float> Quaternion<T> {
    pub fn new(w: T, x: T, y: T, z: T) -> Self {
        Self { w, x, y, z }
    }

    pub fn identity() -> Self {
        Self {
            w: T::one(),
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }

    pub fn conjugate(&self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    pub fn from_axis_angle(axis: Vector3<T>, angle: T) -> Self {
        let half_angle = angle / (T::one() + T::one());
        let sin_half = half_angle.sin();
        Self {
            w: half_angle.cos(),
            x: axis.x * sin_half,
            y: axis.y * sin_half,
            z: axis.z * sin_half,
        }
    }

    pub fn from_euler(euler: &Vector3<T>) -> Self {
        let roll = euler.x;
        let pitch = euler.y;
        let yaw = euler.z;
        let half_roll = roll / (T::one() + T::one());
        let half_pitch = pitch / (T::one() + T::one());
        let half_yaw = yaw / (T::one() + T::one());
        let cr = half_roll.cos();
        let sr = half_roll.sin();
        let cp = half_pitch.cos();
        let sp = half_pitch.sin();
        let cy = half_yaw.cos();
        let sy = half_yaw.sin();
        Self {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    pub fn to_euler(&self) -> Vector3<T> {
        let two = T::one() + T::one();
        let sinp = two * (self.w * self.x - self.y * self.z);
        let pitch = if sinp > T::one() {
            T::from(PI / 2.0).unwrap()
        } else if sinp < -T::one() {
            -T::from(PI / 2.0).unwrap()
        } else {
            sinp.asin()
        };
        let yaw = (two * (self.w * self.y + self.x * self.z))
            .atan2(T::one() - two * (self.x * self.x + self.y * self.y));
        let roll = (two * (self.w * self.z + self.x * self.y))
            .atan2(T::one() - two * (self.x * self.x + self.z * self.z));
        Vector3::new(roll, pitch, yaw)
    }

    pub fn normalize(&self) -> Self {
        let length_sq = self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z;
        let length = length_sq.sqrt();
        
        if length.is_zero() {
            return Self::identity();
        }
        
        Self {
            w: self.w / length,
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    pub fn rotate_vector(&self, v: &Vector3<T>) -> Vector3<T> {
        let qw = self.w;
        let qx = self.x;
        let qy = self.y;
        let qz = self.z;
        
        let two = T::one() + T::one();
        let one = T::one();
        
        let qxx = qx * qx;
        let qyy = qy * qy;
        let qzz = qz * qz;
        let qxz = qx * qz;
        let qxy = qx * qy;
        let qyz = qy * qz;
        let qwx = qw * qx;
        let qwy = qw * qy;
        let qwz = qw * qz;
        
        Vector3 {
            x: v.x * (one - two * (qyy + qzz)) + v.y * two * (qxy - qwz) + v.z * two * (qxz + qwy),
            y: v.x * two * (qxy + qwz) + v.y * (one - two * (qxx + qzz)) + v.z * two * (qyz - qwx),
            z: v.x * two * (qxz - qwy) + v.y * two * (qyz + qwx) + v.z * (one - two * (qxx + qyy)),
        }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }
}

impl<T: Float> Mul for Quaternion<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        self.multiply(&other)
    }
}

impl<T: Float> Mul for &Quaternion<T> {
    type Output = Quaternion<T>;

    fn mul(self, other: Self) -> Self::Output {
        self.multiply(other)
    }
}
