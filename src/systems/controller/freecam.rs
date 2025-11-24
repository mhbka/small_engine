use cgmath::{Deg, InnerSpace, Quaternion, Rad, Rotation3, Vector3, Zero};
use winit::keyboard::KeyCode;

use crate::{core::world::{World, WorldEntityId}, input::state::InputState};

static MOVE_SPEED: f32 = 5.0;
static LOOK_SENS: f32 = 10.0;

/// Just a free-moving controller for an entity, ala freecam.
pub struct FreecamController {
    entity: WorldEntityId,
    enabled: bool
}

impl FreecamController {
    pub fn new(entity: WorldEntityId) -> Self {
        Self {
            entity,
            enabled: true
        }
    } 

    /// Update the entity for this freecam controller.
    pub fn update(&self, input: &InputState, world: &mut World, delta_time: f32) -> Result<(), &'static str> {
        if !self.enabled {
            return Ok(());
        }

        let entity = world
            .entity_mut(self.entity)
            .ok_or("Freecam controller couldn't find the entity")?;
        
        let mut movement: Vector3<f32> = Vector3::zero();
        let scaled_move_speed = MOVE_SPEED * delta_time;
        if input.key_held(KeyCode::KeyW) {
            movement.z += scaled_move_speed;
        }
        if input.key_held(KeyCode::KeyS) {
            movement.z -= scaled_move_speed;
        }
        if input.key_held(KeyCode::KeyA) {
            movement.x += scaled_move_speed;
        }
        if input.key_held(KeyCode::KeyD) {
            movement.x -= scaled_move_speed;
        }
        if input.key_held(KeyCode::Space) {
            movement.y += scaled_move_speed;
        }
        if input.key_held(KeyCode::ShiftLeft) {
            movement.y -= scaled_move_speed;
        }
        entity.update_local_transform(|transform| transform.position += transform.rotation * movement);

        if input.cursor_locked() {
            let mouse_delta = input.mouse_delta();
            let yaw = -mouse_delta.x * LOOK_SENS * delta_time;
            let pitch = mouse_delta.y * LOOK_SENS * delta_time;

            let yaw_q = Quaternion::from_angle_y(Deg(yaw));
            let pitch_q = Quaternion::from_angle_x(Deg(pitch));

            entity.update_local_transform(|transform| transform.rotation = (yaw_q * transform.rotation * pitch_q).normalize());
        }

        Ok(())
    }
}