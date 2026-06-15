use slotmap::{SlotMap, new_key_type};
use crate::graphics::render::renderable::{model::{Material, Mesh}, skybox::Skybox, sprite::Sprite};

new_key_type! {
    pub struct MeshId;
    pub struct MaterialId;
    pub struct SpriteId;
    pub struct SkyboxId;
}

pub struct AssetStore {
    meshes: SlotMap<MeshId, Mesh>,
    materials: SlotMap<MaterialId, Material>,
    sprites: SlotMap<SpriteId, Sprite>,
    skyboxes: SlotMap<SkyboxId, Skybox>
}

impl AssetStore {
    /// Initialize the asset store.
    pub fn new() -> Self {
        Self {
            meshes: SlotMap::with_key(),
            materials: SlotMap::with_key(),
            sprites: SlotMap::with_key(),
            skyboxes: SlotMap::with_key()
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

    /// Add sprites to the store.
    pub fn add_sprites(&mut self, sprites: Vec<Sprite>) -> Vec<SpriteId> {
        sprites
            .into_iter()
            .map(|s| self.sprites.insert(s))
            .collect()
    }

    /// Add skyboxes to the store.
    pub fn add_skyboxes(&mut self, skyboxes: Vec<Skybox>) -> Vec<SkyboxId> {
        skyboxes
            .into_iter()
            .map(|s| self.skyboxes.insert(s))
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

    /// Get a sprite.
    pub fn sprite(&self, id: SpriteId) -> Option<&Sprite> {
        self.sprites.get(id)
    }

    /// Get a skybox.
    pub fn skybox(&self, id: SkyboxId) -> Option<&Skybox> {
        self.skyboxes.get(id)
    }
}
