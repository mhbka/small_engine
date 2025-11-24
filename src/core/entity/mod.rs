use crate::core::entity::spatial_transform::SpatialTransform;
use crate::core::world::WorldEntityId;
use crate::graphics::scene::raw_spatial_transform::RawSpatialTransform;

pub mod spatial_transform;

/// Represents an entity.
pub struct WorldEntity {
    parent: Option<WorldEntityId>,
    children: Vec<WorldEntityId>,
    parent_transform: SpatialTransform,
    local_transform: SpatialTransform,
    already_propagated: bool
}

impl WorldEntity {
    /// Create a new entity.
    pub(super) fn new(
        parent: Option<WorldEntityId>,
        children: Vec<WorldEntityId>,
        local_transform: SpatialTransform,
    ) -> Self {
        Self {
            parent,
            children,
            local_transform,
            parent_transform: SpatialTransform::identity(),
            already_propagated: false,
        }
    }

    /// Get the raw overall transform for this entity. For use in shader.
    pub fn transform_raw(&self) -> RawSpatialTransform {
        self.parent_transform.combine_raw(&self.local_transform)
    }

    /// Get the overall transform for this entity. For propagation.
    pub fn transform(&self) -> SpatialTransform {
        self.parent_transform.combine(&self.local_transform)
    }

    /// Get the parent of the node.
    pub fn parent(&self) -> &Option<WorldEntityId> {
        &self.parent
    }

    /// Get the children.
    pub fn children(&self) -> &Vec<WorldEntityId> {
        &self.children
    }

    /// Get the local transform.
    pub fn local_transform(&self) -> SpatialTransform {
        self.local_transform
    }

    /// Returns `true` if the node's children's parent transforms are up-to-date.
    pub fn already_propagated(&self) -> bool {
        self.already_propagated
    }

    /// Update the node's local transform.
    pub fn update_local_transform<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut SpatialTransform),
    {
        update(&mut self.local_transform);
        self.already_propagated = false;
    }

    /// Update the node's global transform.
    pub fn update_parent_transform<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut SpatialTransform),
    {
        update(&mut self.parent_transform);
        self.already_propagated = false;
    }

    /// Set a new parent.
    pub(super) fn set_parent(&mut self, parent: WorldEntityId) {
        self.parent = Some(parent)
    }

    /// Set the `already_propagated`` flag (ie whether the parent transform has been propagated to the children).
    pub(super) fn set_already_propagated(&mut self, val: bool) {
        self.already_propagated = val;
    }
}