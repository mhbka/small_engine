use crate::graphics::scene::spatial_transform::{RawSpatialTransform, SpatialTransform};
use cgmath::Zero;
use cgmath::{InnerSpace, Rotation3, Vector3};
use slotmap::new_key_type;

new_key_type! {
    /// Used to reference a `SceneNode`.
    pub struct SceneNodeId;
}

/// A node in the scene graph.
///
/// Mostly just contains spatial information.
pub struct SceneNode {
    parent: Option<SceneNodeId>,
    children: Vec<SceneNodeId>,
    global_transform: SpatialTransform,
    local_transform: SpatialTransform,
    propagated_global_to_children: bool,
}

impl SceneNode {
    /// Create a new scene node.
    pub(super) fn new(
        parent: Option<SceneNodeId>,
        children: Vec<SceneNodeId>,
        local_transform: SpatialTransform,
        global_transform: SpatialTransform,
    ) -> Self {
        Self {
            parent,
            children,
            local_transform,
            global_transform,
            propagated_global_to_children: true,
        }
    }

    /// Get the overall transform for this node. For use in shader.
    pub fn transform_raw(&self) -> RawSpatialTransform {
        self.global_transform.combine_raw(&self.local_transform)
    }

    /// Get the overall transform for this node. For propagation.
    pub fn transform(&self) -> SpatialTransform {
        self.global_transform.combine(&self.local_transform)
    }

    /// Get the parent of the node.
    pub fn parent(&self) -> &Option<SceneNodeId> {
        &self.parent
    }

    /// Get the children.
    pub fn children(&self) -> &Vec<SceneNodeId> {
        &self.children
    }

    /// Get the local transform.
    pub fn local_transform(&self) -> SpatialTransform {
        self.local_transform
    }

    /// Returns `true` if the node's transform has been propagated to its children.
    pub fn propagated_global_to_children(&self) -> bool {
        self.propagated_global_to_children
    }

    /// Update the node's local transform.
    pub fn update_local_transform<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut SpatialTransform),
    {
        update(&mut self.local_transform);
        self.propagated_global_to_children = false;
    }

    /// Update the node's global transform.
    pub fn update_global_transform<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut SpatialTransform),
    {
        update(&mut self.global_transform);
        self.propagated_global_to_children = false;
    }

    /// Set a new parent. Only for the graph to use.
    pub(super) fn set_parent(&mut self, parent: SceneNodeId) {
        self.parent = Some(parent)
    }

    /// Set propagated value. Only for the graph to use.
    pub(super) fn set_propagated(&mut self, val: bool) {
        self.propagated_global_to_children = val;
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
                let transform = SpatialTransform {
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    position,
                    rotation,
                };
                SceneNode::new(None, vec![], transform, SpatialTransform::identity())
            })
        })
        .collect::<Vec<_>>();

    transforms
}
