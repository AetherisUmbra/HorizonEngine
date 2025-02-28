use anyhow::Result;
use hengine::engine::Engine;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

pub struct Application {
    window: Option<Arc<Window>>,
    engine: Engine,
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        Ok(Application {
            window: None,
            engine: Engine::new(event_loop)?,
        })
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Horizon")
                        .with_inner_size(LogicalSize::new(800.0, 600.0)),
                )
                .unwrap(),
        ));

        if let Some(window) = self.window.as_ref() {
            _ = self
                .engine
                .renderer()
                .create_render_context(Arc::clone(window));
        }

        self.engine.initialize();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                self.engine.resize();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    self.engine.draw(Arc::clone(&window));
                }
            }
            _ => {
                self.engine.handle_window_event(&event);
            }
        }
    }

    fn device_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _device_id: winit::event::DeviceId,
            event: winit::event::DeviceEvent,
        ) {
        self.engine.handle_device_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.engine.update();
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        println!("Goodbye!");
    }
}
