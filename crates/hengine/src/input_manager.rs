use std::collections::{HashMap, HashSet};

use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InputAction(pub String);

#[derive(Debug, Clone, Copy)]
pub enum InputValue {
    Button(bool),
    Scalar(f32),
    Vector2(f32, f32),
}

impl Default for InputValue {
    fn default() -> Self {
        InputValue::Button(false)
    }
}

pub struct ActionMap {
    name: String,
    actions: HashSet<InputAction>,
    enabled: bool,
}

impl ActionMap {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            actions: HashSet::new(),
            enabled: true,
        }
    }

    pub fn add_action(&mut self, action: InputAction) {
        self.actions.insert(action);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

pub struct InputManager {
    action_maps: HashMap<String, ActionMap>,
    action_values: HashMap<InputAction, InputValue>,
    key_bindings: HashMap<KeyCode, InputAction>,
    mouse_bindings: HashMap<MouseButton, InputAction>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    
    action_callbacks: HashMap<InputAction, Vec<Box<dyn Fn(InputValue) + Send + Sync>>>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            action_maps: HashMap::new(),
            action_values: HashMap::new(),
            key_bindings: HashMap::new(),
            mouse_bindings: HashMap::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            action_callbacks: HashMap::new(),
        }
    }

    pub fn create_action_map(&mut self, name: &str) -> &mut ActionMap {
        let map = ActionMap::new(name);
        self.action_maps.insert(name.to_string(), map);
        self.action_maps.get_mut(name).unwrap()
    }

    pub fn bind_key(&mut self, key: KeyCode, action: InputAction) {
        self.key_bindings.insert(key, action.clone());
        self.action_values.insert(action, InputValue::default());
    }

    pub fn bind_mouse_button(&mut self, button: MouseButton, action: InputAction) {
        self.mouse_bindings.insert(button, action.clone());
        self.action_values.insert(action, InputValue::default());
    }

    pub fn bind_mouse_motion(&mut self, action: InputAction) {
        self.action_values.insert(action, InputValue::Vector2(0.0, 0.0));
    }

    pub fn register_action_callback<F>(&mut self, action: InputAction, callback: F)
    where
        F: Fn(InputValue) + Send + Sync + 'static,
    {
        let callbacks = self.action_callbacks.entry(action).or_insert_with(Vec::new);
        callbacks.push(Box::new(callback));
    }

    pub fn get_action_value(&self, action: &InputAction) -> Option<InputValue> {
        self.action_values.get(action).copied()
    }

    pub fn process_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(action) = self.mouse_bindings.get(button) {
                    let value = match state {
                        ElementState::Pressed => InputValue::Button(true),
                        ElementState::Released => InputValue::Button(false),
                    };
                    self.update_action_value(action.clone(), value);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = (position.x as f32, position.y as f32);
                self.mouse_position = new_pos;
            }
            _ => {}
        }
    }

    pub fn process_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta = (delta.0 as f32, delta.1 as f32);
                
                for (action, value) in self.action_values.iter_mut() {
                    if let InputValue::Vector2(_, _) = value {
                        *value = InputValue::Vector2(delta.0 as f32, delta.1 as f32);
                        
                        if let Some(callbacks) = self.action_callbacks.get(action) {
                            for callback in callbacks {
                                callback(*value);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_keyboard_input(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            if let Some(action) = self.key_bindings.get(&key_code) {
                let value = match event.state {
                    ElementState::Pressed => InputValue::Button(true),
                    ElementState::Released => InputValue::Button(false),
                };
                self.update_action_value(action.clone(), value);
            }
        }
    }

    fn update_action_value(&mut self, action: InputAction, value: InputValue) {
        self.action_values.insert(action.clone(), value);
        
        if let Some(callbacks) = self.action_callbacks.get(&action) {
            for callback in callbacks {
                callback(value);
            }
        }
    }

    pub fn update(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }
}