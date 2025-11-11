use cgmath::Zero;
use cgmath::{InnerSpace, Rotation3, Vector3};
use slotmap::new_key_type;

use crate::scene::spacial_transform::{RawSpacialTransform, SpacialTransform};

new_key_type! {
    /// Used to reference a `SceneNode`.
    pub struct SceneNodeId;
}

pub struct SceneNode {
    parent: Option<SceneNodeId>,
    children: Vec<SceneNodeId>,
    local_transform: SpacialTransform,
    global_transform: SpacialTransform,
}

impl SceneNode {
    /// Create a new scene node.
    pub(super) fn new(
        parent: Option<SceneNodeId>,
        children: Vec<SceneNodeId>,
        local_transform: SpacialTransform,
        global_transform: SpacialTransform,
    ) -> Self {
        Self {
            parent,
            children,
            local_transform,
            global_transform,
        }
    }

    /// Get the overall transform for this node.
    pub fn get_transform(&self) -> RawSpacialTransform {
        self.global_transform.combine(&self.local_transform)
    }

    /// Get the node's local transform mutably.
    pub fn local_transform(&mut self) -> &mut SpacialTransform {
        &mut self.local_transform
    }
}

/// Just generate some spaced nodes as an example.
pub fn generate_example_nodes() -> Vec<SceneNode> {
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

    let transforms = (0..NUM_INSTANCES_PER_ROW)
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
                let transform = SpacialTransform {
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    position,
                    rotation,
                };
                SceneNode::new(None, vec![], transform, SpacialTransform::identity())
            })
        })
        .collect::<Vec<_>>();

    transforms
}
