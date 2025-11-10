use cgmath::{Vector3, prelude::*};
use slotmap::{SecondaryMap, new_key_type};
use wgpu::BufferSlice;
use crate::{gpu::{GpuContext, buffer::GpuBuffer}, render::assets::{MaterialId, MeshId}, scene::{node::SceneNodeId, spacial_transform::RawSpacialTransform}};

pub const NUM_INSTANCES_PER_ROW: u32 = 10;
pub const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub const BOB_SPEED: f32 = 1.0;
pub const ROTATION_SPEED: f32 = 1.0;
pub const MAX_VERTICAL_OFFSET: f32 = 0.3;

new_key_type! {
    /// To refer to a mesh instance.
    pub struct MeshInstanceId;
}

/// Just generate some spaced instances.
pub fn generate_instances() -> Vec<MeshInstance> {
    todo!();

    const SPACE_BETWEEN: f32 = 3.0;
    let instances = (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = cgmath::Vector3 { x, y: 0.0, z };

                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance {
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    position,
                    rotation,
                }
            })
        })
        .collect::<Vec<_>>();

    instances
}

/// Represents an instance of a mesh.
/// 
/// The instance points to the actual mesh it is an instance of,
/// the scene node containing its spatial data,
/// and the material for it.
pub struct MeshInstance {
    pub mesh: MeshId,
    pub node: SceneNodeId,
    pub material: MaterialId,
}

