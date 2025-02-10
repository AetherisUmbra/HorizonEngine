use anyhow::Result;
use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::window::Window;

pub struct Engine {
    renderer: hrenderer::renderer::Renderer,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        Ok(Self {
            renderer: hrenderer::renderer::Renderer::new(event_loop)?,
        })
    }

    pub fn resize(&mut self) {
        self.renderer.resize();
    }

    pub fn draw(&mut self, window: Arc<Window>) {
        self.renderer.draw(window);
    }
    
    pub fn renderer(&mut self) -> &mut hrenderer::renderer::Renderer {
        &mut self.renderer
    }
}
