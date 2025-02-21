use num_traits::Float;
use std::ops::{Add, Div, Mul, Sub};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Vector3d = Vector3<f64>;
pub type Vector3f = Vector3<f32>;

impl<T> Vector3<T>
where
    T: Float,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Vector3 { x, y, z }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline]
    pub fn norm_squared(&self) -> T {
        self.dot(self)
    }

    #[inline]
    pub fn length(&self) -> T {
        self.norm_squared().sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let length = self.length();
        if length == T::zero() {
            *self
        } else {
            *self / length
        }
    }

    #[inline]
    pub fn distance(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl<T: Float> Add for Vector3<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T: Float> Sub for Vector3<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T: Float> Mul<T> for Vector3<T> {
    type Output = Self;

    #[inline]
    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl<T: Float> Div<T> for Vector3<T> {
    type Output = Self;

    #[inline]
    fn div(self, scalar: T) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}
