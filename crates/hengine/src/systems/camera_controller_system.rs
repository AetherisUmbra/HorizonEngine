use crate::components::camera_component::CameraComponent;
use crate::components::transform_component::TransformComponent;
use crate::input_manager::{InputAction, InputManager, InputValue};
use hecs::World;
use hmath::quaternion::Quaternion;
use hmath::vector::Vector3d;

pub struct CameraControllerConfig {
    pub move_speed: f64,
    pub rotation_speed: f64,
}

impl Default for CameraControllerConfig {
    fn default() -> Self {
        Self {
            move_speed: 1.0,
            rotation_speed: 0.0,
        }
    }
}

pub struct CameraControllerSystem {
    config: CameraControllerConfig,
    move_forward: InputAction,
    move_backward: InputAction,
    move_left: InputAction,
    move_right: InputAction,
    move_up: InputAction,
    move_down: InputAction,
    look_action: InputAction,
}

impl CameraControllerSystem {
    pub fn new(config: CameraControllerConfig) -> Self {
        Self {
            config,
            move_forward: InputAction("MoveForward".to_string()),
            move_backward: InputAction("MoveBackward".to_string()),
            move_left: InputAction("MoveLeft".to_string()),
            move_right: InputAction("MoveRight".to_string()),
            move_up: InputAction("MoveUp".to_string()),
            move_down: InputAction("MoveDown".to_string()),
            look_action: InputAction("Look".to_string()),
        }
    }

    pub fn setup_input(&self, input_manager: &mut InputManager) {
        use winit::keyboard::KeyCode;
        
        input_manager.bind_key(KeyCode::KeyW, self.move_forward.clone());
        input_manager.bind_key(KeyCode::KeyS, self.move_backward.clone());
        input_manager.bind_key(KeyCode::KeyA, self.move_left.clone());
        input_manager.bind_key(KeyCode::KeyD, self.move_right.clone());
        input_manager.bind_key(KeyCode::Space, self.move_up.clone());
        input_manager.bind_key(KeyCode::ShiftLeft, self.move_down.clone());
        
        input_manager.bind_mouse_motion(self.look_action.clone());
    }

    pub fn update(&self, world: &mut World, input_manager: &InputManager, delta_time: f32) {
        for (_, (transform, _)) in world.query_mut::<(&mut TransformComponent, &CameraComponent)>() {
            let mut movement = Vector3d::zero();
            
            if let Some(InputValue::Button(true)) = input_manager.get_action_value(&self.move_forward) {
                movement.z += 1.0;
            }
            if let Some(InputValue::Button(true)) = input_manager.get_action_value(&self.move_backward) {
                movement.z -= 1.0;
            }
            if let Some(InputValue::Button(true)) = input_manager.get_action_value(&self.move_right) {
                movement.x -= 1.0;
            }
            if let Some(InputValue::Button(true)) = input_manager.get_action_value(&self.move_left) {
                movement.x += 1.0;
            }
            if let Some(InputValue::Button(true)) = input_manager.get_action_value(&self.move_up) {
                movement.y += 1.0;
            }
            if let Some(InputValue::Button(true)) = input_manager.get_action_value(&self.move_down) {
                movement.y -= 1.0;
            }
            
            if movement.norm_squared() > 0.0 {
                movement = movement.normalize();
            }
            
            let rotation = transform.rotation;
            let rotated_movement = rotation.rotate_vector(&movement);
            
            let delta_movement = rotated_movement * self.config.move_speed * delta_time as f64;
            transform.position += delta_movement;
            
            if let Some(InputValue::Vector2(dx, dy)) = input_manager.get_action_value(&self.look_action) {
                let yaw = -dx as f64 * self.config.rotation_speed;
                let pitch = -dy as f64 * self.config.rotation_speed;
                
                let yaw_quat = Quaternion::from_axis_angle(Vector3d::new(0.0, 1.0, 0.0), yaw);
                let right = rotation.rotate_vector(&Vector3d::new(1.0, 0.0, 0.0));
                let pitch_quat = Quaternion::from_axis_angle(right, pitch);
                
                transform.rotation = yaw_quat * pitch_quat * transform.rotation;
                transform.rotation = transform.rotation.normalize();
            }
        }
    }
} 