use std::sync::Arc;
use vulkano::{
    image::view::ImageView,
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline},
    swapchain::Swapchain,
    sync::GpuFuture,
};

pub struct RenderContext {
    pub swapchain: Arc<Swapchain>,
    pub attachment_image_views: Vec<Arc<ImageView>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl RenderContext {}
