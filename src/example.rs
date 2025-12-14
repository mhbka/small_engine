use cgmath::InnerSpace;
use cgmath::Quaternion;
use cgmath::Rad;
use cgmath::Rotation3;
use cgmath::Vector3;
use cgmath::Zero;
use crate::core::entity::spatial_transform::SpatialTransform;
use crate::core::world::World;
use crate::core::world::WorldEntityId;

pub fn generate_one_big_entity(world: &mut World) -> Vec<WorldEntityId> {
    let transform = SpatialTransform {
        scale: Vector3::new(1.0, 1.0, 1.0),
        position: Vector3::new(0.0, 0.0, 0.0),
        rotation: Quaternion::from_angle_z(Rad(0.0)),
    };
    let entity = world.add_entity(None, vec![], transform);

    vec![entity]
}

/// Just generate some spaced nodes as an example.
pub fn generated_spaced_entities(world: &mut World) -> Vec<WorldEntityId> {
    pub const NUM_INSTANCES_PER_ROW: u32 = 10;
    pub const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    pub const BOB_SPEED: f32 = 1.0;
    pub const ROTATION_SPEED: f32 = 1.0;
    pub const MAX_VERTICAL_OFFSET: f32 = 0.3;
    const SPACE_BETWEEN: f32 = 3.0;

    let entities = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(|x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = cgmath::Vector3 { x, y: 0.0, z };
                let rotation = if position.is_zero() {
                    Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };
                let scale = Vector3::new(1.0, 1.0, 1.0);
                let transform = SpatialTransform {
                    scale,
                    position,
                    rotation,
                };
                world.add_entity(None, vec![], transform)
            })
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    entities
}