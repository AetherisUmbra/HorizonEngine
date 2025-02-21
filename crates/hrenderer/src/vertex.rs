use hmath::vector::Vector3;
use std::alloc::Layout;
use std::collections::HashMap;
use std::ptr::NonNull;
use vulkano::buffer::{BufferContents, BufferContentsLayout};
use vulkano::format::Format;
use vulkano::pipeline;
use vulkano::pipeline::graphics::vertex_input::{
    Vertex as VulkanoVertex, VertexBufferDescription, VertexMemberInfo,
};

#[repr(C)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

unsafe impl BufferContents for Vertex {
    const LAYOUT: BufferContentsLayout = BufferContentsLayout::from_sized(Layout::new::<Vertex>());

    unsafe fn ptr_from_slice(slice: NonNull<[u8]>) -> *mut Self {
        <*mut [u8]>::cast::<Vertex>(slice.as_ptr())
    }
}

unsafe impl VulkanoVertex for Vertex {
    #[inline(always)]
    fn per_vertex() -> VertexBufferDescription {
        VertexBufferDescription {
            members: HashMap::from([
                (
                    String::from("position"),
                    VertexMemberInfo {
                        offset: 0,
                        format: Format::R32G32B32_SFLOAT,
                        num_elements: 1,
                        stride: 0,
                    },
                ),
                (
                    String::from("color"),
                    VertexMemberInfo {
                        offset: 12,
                        format: Format::R32G32B32_SFLOAT,
                        num_elements: 1,
                        stride: 0,
                    },
                ),
            ]),
            stride: ::std::mem::size_of::<Vertex>() as u32,
            input_rate: pipeline::graphics::vertex_input::VertexInputRate::Vertex,
        }
    }

    #[inline(always)]
    fn per_instance() -> VertexBufferDescription {
        Self::per_vertex().per_instance()
    }

    #[inline(always)]
    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        Self::per_vertex().per_instance_with_divisor(divisor)
    }
}
