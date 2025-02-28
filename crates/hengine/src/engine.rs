use crate::components::camera_component::CameraComponent;
use crate::components::transform_component::TransformComponent;
use crate::input_manager::InputManager;
use crate::systems::camera_controller_system::{CameraControllerConfig, CameraControllerSystem};
use anyhow::Result;
use hmath::vector::{Vector3d, Vector3f};
use std::sync::Arc;
use std::time::Instant;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;
use hmath::quaternion::Quaternion;

pub struct Engine {
    renderer: hrenderer::renderer::Renderer,
    world: hecs::World,
    input_manager: InputManager,
    camera_controller: CameraControllerSystem,
    last_update: Instant,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let camera_controller = CameraControllerSystem::new(CameraControllerConfig::default());
        let mut input_manager = InputManager::new();
        
        // Set up camera controller input
        camera_controller.setup_input(&mut input_manager);
        
        Ok(Self {
            renderer: hrenderer::renderer::Renderer::new(event_loop)?,
            world: hecs::World::new(),
            input_manager,
            camera_controller,
            last_update: Instant::now(),
        })
    }

    pub fn initialize(&mut self) {
        self.world.spawn((
            TransformComponent {
                position: Vector3d::new(0.0, 0.0, -5.0),
                rotation: Quaternion::identity(),
                scale: Vector3f::new(1.0, 1.0, 1.0),
            },
            CameraComponent {
                fov: 90.0,
                aspect: 1.0,
                near: 0.1,
                far: 100.0,
            },
        ));
    }
    
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        self.input_manager.process_window_event(event);
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        self.input_manager.process_device_event(event);
    }
    
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        
        self.camera_controller.update(&mut self.world, &self.input_manager, delta_time);
        
        self.input_manager.update();
    }

    pub fn resize(&mut self) {
        self.renderer.resize();
    }

    pub fn draw(&mut self, window: Arc<Window>) {
        self.world
            .query::<(&CameraComponent, &TransformComponent)>()
            .iter()
            .for_each(|(_, (camera, transform))| {
                let window_size = window.inner_size();
                let aspect = window_size.width as f32 / window_size.height as f32;
                
                let projection = renderer::camera_utils::build_perspective_projection_matrix(
                    camera.fov,
                    aspect,
                    camera.near,
                    camera.far,
                );
                
                let forward = transform.rotation.rotate_vector(&Vector3d::new(0.0, 0.0, 1.0));
                let up = transform.rotation.rotate_vector(&Vector3d::new(0.0, 1.0, 0.0));
                let target = transform.position + forward;
                
                let view = renderer::camera_utils::build_view_matrix(
                    transform.position,
                    target,
                    up,
                );

                self.renderer.set_camera_matrices(&view, &projection);
            });
            
        self.renderer.draw(window);
    }

    pub fn renderer(&mut self) -> &mut hrenderer::renderer::Renderer {
        &mut self.renderer
    }
    
    pub fn input_manager(&mut self) -> &mut InputManager {
        &mut self.input_manager
    }
}
