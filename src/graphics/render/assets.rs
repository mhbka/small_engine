use slotmap::{SlotMap, new_key_type};

use crate::graphics::{
    gpu::texture::GpuTexture,
    render::renderable::model::{Material, Mesh},
};

new_key_type! {
    pub struct MeshId;
    pub struct MaterialId;
    pub struct SpriteTextureId;
}

pub struct AssetStore {
    meshes: SlotMap<MeshId, Mesh>,
    materials: SlotMap<MaterialId, Material>,
    sprite_textures: SlotMap<SpriteTextureId, GpuTexture>,
}

impl AssetStore {
    /// Initialize the asset store.
    pub fn new() -> Self {
        Self {
            meshes: SlotMap::with_key(),
            materials: SlotMap::with_key(),
            sprite_textures: SlotMap::with_key(),
        }
    }

    /// Add materials to the store.
    pub fn add_materials(&mut self, materials: Vec<Material>) -> Vec<MaterialId> {
        materials
            .into_iter()
            .map(|m| self.materials.insert(m))
            .collect()
    }

    /// Add meshes to the store.
    pub fn add_meshes(&mut self, meshes: Vec<Mesh>) -> Vec<MeshId> {
        meshes.into_iter().map(|m| self.meshes.insert(m)).collect()
    }

    /// Add meshes to the store.
    pub fn add_sprite_textures(&mut self, meshes: Vec<GpuTexture>) -> Vec<SpriteTextureId> {
        meshes
            .into_iter()
            .map(|s| self.sprite_textures.insert(s))
            .collect()
    }

    /// Get a material.
    pub fn material(&self, id: MaterialId) -> Option<&Material> {
        self.materials.get(id)
    }

    /// Get a mesh.
    pub fn mesh(&self, id: MeshId) -> Option<&Mesh> {
        self.meshes.get(id)
    }

    /// Get a sprite texture.
    pub fn sprite_texture(&self, id: SpriteTextureId) -> Option<&GpuTexture> {
        self.sprite_textures.get(id)
    }
}
