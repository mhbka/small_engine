use crate::{gpu::texture::GpuTexture, render::renderable::sprite::quad::QuadVertex, scene::node::SceneNodeId};

pub mod quad;
pub mod instance;

/// A sprite, which is just a texture on a quad.
pub struct Sprite {
    texture: GpuTexture,
}