use crate::application::Application;
use anyhow::Result;
use winit::event_loop::EventLoop;

mod application;

fn main() -> Result<()> {
    println!("Welcome to Horizon Engine!");

    let event_loop = EventLoop::new()?;
    let mut app = Application::new(&event_loop)?;

    event_loop.run_app(&mut app)?;

    Ok(())
}
