use cgmath::{Vector3, Zero};

use crate::debug_menu::DebugMenuData;

/// The debug state for our game.
pub struct DebugState {
    camera_position: Vector3<f32> 
}

impl DebugState {
    /// Instantiate.
    pub fn new() -> Self {
        Self {
            camera_position: Vector3::zero()
        }
    }

    /// Update the debug state.
    pub fn update(&mut self, camera_position: Vector3<f32>) {
        self.camera_position = camera_position;
    }
}

impl DebugMenuData for DebugState {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Camera Position: ");
        ui.label(format!("{:.3}, {:.3}, {:.3}", self.camera_position.x, self.camera_position.y, self.camera_position.z));
        ui.end_row();
    }
}