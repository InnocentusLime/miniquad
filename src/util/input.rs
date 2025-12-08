use std::collections::HashSet;

use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

const KEY_SET_CAPACITY: usize = 255;

pub struct InputTracker {
    pressed_mouse_buttons: HashSet<MouseButton>,
    held_mouse_buttons: HashSet<MouseButton>,
    released_mouse_buttons: HashSet<MouseButton>,
    pressed_keys: HashSet<KeyCode>,
    held_keys: HashSet<KeyCode>,
    released_keys: HashSet<KeyCode>,
}

impl InputTracker {
    pub fn new() -> Self {
        InputTracker {
            pressed_keys: HashSet::with_capacity(KEY_SET_CAPACITY),
            held_keys: HashSet::with_capacity(KEY_SET_CAPACITY),
            released_keys: HashSet::with_capacity(KEY_SET_CAPACITY),
            pressed_mouse_buttons: HashSet::with_capacity(KEY_SET_CAPACITY),
            held_mouse_buttons: HashSet::with_capacity(KEY_SET_CAPACITY),
            released_mouse_buttons: HashSet::with_capacity(KEY_SET_CAPACITY),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => match (event.physical_key, event.state) {
                (PhysicalKey::Code(code), ElementState::Pressed)
                    if !self.held_keys.contains(&code) =>
                {
                    self.pressed_keys.insert(code);
                }
                (PhysicalKey::Code(code), ElementState::Released) => {
                    self.held_keys.remove(&code);
                    self.released_keys.insert(code);
                }
                _ => (),
            },
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed if !self.held_mouse_buttons.contains(button) => {
                    self.pressed_mouse_buttons.insert(*button);
                }
                ElementState::Released => {
                    self.held_mouse_buttons.remove(button);
                    self.released_mouse_buttons.insert(*button);
                }
                _ => (),
            },
            _ => (),
        }
    }

    /// Call at the end of frame
    pub fn update(&mut self) {
        self.released_mouse_buttons.clear();
        for button in self.pressed_mouse_buttons.drain() {
            self.held_mouse_buttons.insert(button);
        }

        self.released_keys.clear();
        for key in self.pressed_keys.drain() {
            self.held_keys.insert(key);
        }
    }

    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.held_keys.contains(&key)
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.released_keys.contains(&key)
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.held_mouse_buttons.contains(&button)
    }

    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    pub fn is_button_released(&self, button: MouseButton) -> bool {
        self.released_mouse_buttons.contains(&button)
    }
}
