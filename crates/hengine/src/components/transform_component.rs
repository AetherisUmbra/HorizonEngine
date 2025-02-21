use hmath::quaternion::Quaternion;
use hmath::vector::{Vector3d, Vector3f};

pub struct TransformComponent {
    pub position: Vector3d,
    pub rotation: Quaternion<f64>,
    pub scale: Vector3f,
}
