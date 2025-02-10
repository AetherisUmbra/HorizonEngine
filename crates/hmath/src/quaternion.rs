use crate::vector::Vector3;
use num_traits::Float;
use std::f64::consts::PI;

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

    pub fn from_axis_angle(axis: &[T; 3], angle: T) -> Self {
        let half_angle = angle / (T::one() + T::one());
        let sin_half = half_angle.sin();
        Self {
            w: half_angle.cos(),
            x: axis[0] * sin_half,
            y: axis[1] * sin_half,
            z: axis[2] * sin_half,
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
}
