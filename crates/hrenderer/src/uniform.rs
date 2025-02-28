use hmath::matrix::Matrix4x4;
use vulkano::buffer::BufferContents;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, BufferContents)]
pub struct UniformBufferObject {
    pub view: Matrix4x4,
    pub proj: Matrix4x4,
}

impl UniformBufferObject {
    pub fn new(view: Matrix4x4, proj: Matrix4x4) -> Self {
        Self { view, proj }
    }
}
