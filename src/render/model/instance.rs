use cgmath::{Vector3, prelude::*};
use slotmap::{SecondaryMap, new_key_type};
use wgpu::BufferSlice;
use crate::{gpu::{GpuContext, buffer::GpuBuffer}, render::assets::{MaterialId, MeshId}, scene::{node::SceneNodeId, spacial_transform::RawSpacialTransform}};

new_key_type! {
    /// To refer to a mesh instance.
    pub struct MeshInstanceId;
}

/// Represents an instance of a mesh.
/// 
/// The instance points to the actual mesh it is an instance of,
/// the scene node containing its spatial data,
/// and the material for it.
#[derive(Clone)]
pub struct MeshInstance {
    pub mesh: MeshId,
    pub node: SceneNodeId,
}

