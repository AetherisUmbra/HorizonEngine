use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;

#[derive(BufferContents, VulkanoVertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}
