use hmath::matrix::Matrix4x4;
use hmath::vector::Vector3d;

pub fn build_perspective_projection_matrix(fovy: f32, aspect: f32, near: f32, far: f32) -> Matrix4x4 {
    let f = 1.0 / (fovy / 2.0).tan();
    let nf = 1.0 / (near - far);

    Matrix4x4 {
        data: [
            f / aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            f,
            0.0,
            0.0,
            0.0,
            0.0,
            (far + near) * nf,
            -1.0,
            0.0,
            0.0,
            (2.0 * far * near) * nf,
            0.0,
        ],
    }
}

pub fn build_view_matrix(position: Vector3d, target: Vector3d, up: Vector3d) -> Matrix4x4 {
    let f = (target - position).normalize();
    let s = f.cross(&up).normalize();
    let u = s.cross(&f);

    Matrix4x4 {
        data: [
            s.x as f32,
            u.x as f32,
            -f.x as f32,
            0.0,
            s.y as f32,
            u.y as f32,
            -f.y as f32,
            0.0,
            s.z as f32,
            u.z as f32,
            -f.z as f32,
            0.0,
            -s.dot(&position) as f32,
            -u.dot(&position) as f32,
            f.dot(&position) as f32,
            1.0,
        ],
    }
}