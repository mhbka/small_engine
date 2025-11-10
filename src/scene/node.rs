use cgmath::Vector3;
use slotmap::new_key_type;

use crate::scene::spacial_transform::{RawSpacialTransform, SpacialTransform};

new_key_type! {
    /// Used to reference a `SceneNode`.
    pub struct SceneNodeId;
}

pub struct SceneNode {
    /// The parent node for this node.
    parent: Option<SceneNodeId>,
    /// Any children for this node.
    children: Vec<SceneNodeId>,
    /// The local spacial transform for this node.
    local_transform: SpacialTransform,
    /// The global transform for this node (from its parents; precomputed for efficiency).
    global_transform: SpacialTransform
}

impl SceneNode {
    /// Create a new scene node.
    pub(super) fn new(
        parent: Option<SceneNodeId>,
        children: Vec<SceneNodeId>,
        local_transform: SpacialTransform,
        global_transform: SpacialTransform
    ) -> Self {
        Self {
            parent,
            children,
            local_transform,
            global_transform
        }
    }

    /// Get the overall transform for this node.
    pub fn get_transform(&self) -> RawSpacialTransform {
        self.global_transform.combine(&self.local_transform)
    }

    /// Get the node's local transform mutably.
    pub fn local_transform(&mut self) -> &mut SpacialTransform { &mut self.local_transform } 
}