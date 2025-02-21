use crate::components::camera_component::CameraComponent;
use crate::components::transform_component::TransformComponent;
use crate::renderer;
use anyhow::Result;
use hmath::vector::{Vector3d, Vector3f};
use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::window::Window;
use hmath::quaternion::Quaternion;

pub struct Engine {
    renderer: hrenderer::renderer::Renderer,
    world: hecs::World,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        Ok(Self {
            renderer: hrenderer::renderer::Renderer::new(event_loop)?,
            world: hecs::World::new(),
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

    pub fn resize(&mut self) {
        self.renderer.resize();
    }

    pub fn draw(&mut self, window: Arc<Window>) {
        //get the camera and make the projection matrix
        self.world
            .query::<(&CameraComponent, &TransformComponent)>()
            .iter()
            .for_each(|(_, (camera, transform))| {
                let projection = renderer::camera_utils::build_perspective_projection_matrix(
                    camera.fov,
                    camera.aspect,
                    camera.near,
                    camera.far,
                );
                let view = renderer::camera_utils::build_view_matrix(
                    transform.position,
                    Vector3d::new(0.0, 0.0, 0.0),
                    Vector3d::new(0.0, 1.0, 0.0),
                );

                self.renderer.set_projection_matrix(&projection);
                self.renderer.set_view_matrix(&view);
            });
        self.renderer.draw(window);
    }

    pub fn renderer(&mut self) -> &mut hrenderer::renderer::Renderer {
        &mut self.renderer
    }
}
