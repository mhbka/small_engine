use cgmath::{Vector2, Zero};
use rustc_hash::FxHashSet;
use winit::{event::{ElementState, MouseButton, MouseScrollDelta}, keyboard::KeyCode};

/// Contains the state of inputs for the current frame.
pub struct InputState {
    // keyboard
    keys_held: FxHashSet<KeyCode>,
    keys_pressed: FxHashSet<KeyCode>,
    keys_released: FxHashSet<KeyCode>,

    // mouse
    mouse_pos: Vector2<f32>,
    mouse_delta: Vector2<f32>,
    mouse_held: FxHashSet<MouseButton>,
    mouse_pressed: FxHashSet<MouseButton>,
    mouse_released: FxHashSet<MouseButton>,

    // mouse capture
    cursor_locked: bool
}

impl InputState {
    pub fn new(cursor_locked: bool) -> Self {
        Self {
            keys_held: FxHashSet::default(),
            keys_pressed: FxHashSet::default(),
            keys_released: FxHashSet::default(),
            mouse_pos: Vector2 { x: 0.0, y: 0.0 },
            mouse_delta: Vector2 { x: 0.0, y: 0.0 },
            mouse_held: FxHashSet::default(),
            mouse_pressed: FxHashSet::default(),
            mouse_released: FxHashSet::default(),
            cursor_locked
        }
    }

    /// Whether the given key is held at this frame.
    pub fn key_held(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    /// Whether the cursor is locked; typically for FPS style cameras.
    pub fn cursor_locked(&self) -> bool {self.cursor_locked }

    /// The mouse delta for the frame.
    pub fn mouse_delta(&self) -> &Vector2<f32> { &self.mouse_delta }

    /// Refresh the input state on a new frame.
    pub fn begin_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_delta = Vector2::zero();
    }

    pub fn process_key_event(&mut self, key_code: KeyCode, key_state: ElementState) {
        match key_state {
            ElementState::Pressed => {
                self.keys_pressed.insert(key_code);
                self.keys_held.insert(key_code);
            },
            ElementState::Released => {
                self.keys_released.insert(key_code);
                self.keys_held.remove(&key_code);
            }
        }
    }

    pub fn process_cursor_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_delta += Vector2 { x: delta_x, y: delta_y };
    }

    pub fn process_cursor_movement(&mut self, x: f32, y: f32) {
        self.mouse_pos = Vector2 { x, y }
    }

    pub fn process_mouse_scroll(&mut self, change: MouseScrollDelta) {
        log::warn!("mouse scroll input not implemented")
    }
}