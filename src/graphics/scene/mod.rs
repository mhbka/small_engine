pub mod instance_buffer;
pub mod light;
pub mod raw_spatial_transform;

use slotmap::{SecondaryMap, SlotMap, new_key_type};
use thiserror::Error;
use crate::{core::world::{World, WorldEntityId}, graphics::{
    gpu::GpuContext,
    render::{
        assets::{AssetStore, MaterialId, MeshId, SpriteId}, commands::{MeshRenderCommand, RenderCommandBuffer, SpriteRenderCommand}, renderable::{model::MeshInstance, skybox::SkyBox, sprite::SpriteInstance}, renderer::{BindGroupId, PipelineId}
    },
    scene::{
        instance_buffer::InstanceBuffer, light::point::{PointLight, PointLightCollection}, raw_spatial_transform::RawSpatialTransform
    },
},
    systems::camera::Camera};

new_key_type! {
    /// To refer to a mesh instance.
    pub struct MeshInstanceId;
    /// To refer to a sprite instance.
    pub struct SpriteInstanceId;
}

/// The main representation of "something" in the game.
pub struct Scene {
    mesh_instances: SlotMap<MeshInstanceId, MeshInstance>,
    instances_by_mesh: SecondaryMap<MeshId, Vec<MeshInstanceId>>, // optimization, so that we don't need to iterate `mesh_instances` to find all instances for a mesh.
    sprite_instances: SlotMap<SpriteInstanceId, SpriteInstance>, 
    instances_by_sprite: SecondaryMap<SpriteId, Vec<SpriteInstanceId>>, // optimization, same as ^
    camera: Camera, 
    point_lights: PointLightCollection,
    pipeline: PipelineId,
    camera_bind_group: BindGroupId,
    lighting_bind_group: BindGroupId,
    skybox: SkyBox,
    sky_pipeline: PipelineId,
    sky_bind_group: BindGroupId,
}

impl Scene {
    /// Construct a scene.
    pub fn new(
        camera: Camera,
        point_lights: PointLightCollection,
        pipeline: PipelineId,
        camera_bind_group: BindGroupId,
        lighting_bind_group: BindGroupId,
        skybox: SkyBox,
        sky_pipeline: PipelineId,
        sky_bind_group: BindGroupId,
    ) -> Self {
        Self {
            mesh_instances: SlotMap::with_key(),
            instances_by_mesh: SecondaryMap::new(),
            sprite_instances: SlotMap::with_key(),
            instances_by_sprite: SecondaryMap::new(),
            camera,
            point_lights,
            pipeline,
            skybox,
            sky_pipeline,
            sky_bind_group,
            camera_bind_group,
            lighting_bind_group,
        }
    }

    /// Convert the scene to render commands.
    pub fn to_commands<'a>(
        &'a self,
        world: &World,
        assets: &'a AssetStore,
        instance_buffer: &mut InstanceBuffer,
    ) -> Result<RenderCommandBuffer<'a>, SceneError> {
        let mesh_commands = self.mesh_commands(world, assets, instance_buffer)?;
        let sprite_commands = self.sprite_commands(world, assets, instance_buffer)?;
        let sky_command = self.skybox.to_render_command(
            self.sky_pipeline,
            self.sky_bind_group,
            self.camera_bind_group
        );
        let commands = RenderCommandBuffer {
            mesh: mesh_commands,
            sprite: sprite_commands,
            skybox: Some(sky_command)
        };
        Ok(commands)
    }

    /// Updates and writes updateable buffers.
    ///
    /// Currently, this is for the camera and light uniforms.
    pub fn update_and_write_buffers(&mut self, world: &World, gpu: &GpuContext) {
        self.camera.update_and_write_uniform_buffer(world, gpu);
        self.point_lights.update_and_write_buffer(world, gpu);
    }

    /// Add the mesh instances under that mesh, returning their IDs.
    pub fn add_mesh_instances(
        &mut self,
        mesh: MeshId,
        instances: Vec<MeshInstance>,
    ) -> Vec<MeshInstanceId> {
        let mut instance_ids = instances
            .into_iter()
            .map(|inst| self.mesh_instances.insert(inst))
            .collect();
        match self.instances_by_mesh.get_mut(mesh) {
            Some(cur_instances) => cur_instances.append(&mut instance_ids),
            None => self
                .instances_by_mesh
                .insert(mesh, instance_ids.clone())
                .map_or((), |_| ()),
        }
        instance_ids
    }

    /// Get the mesh commands for a scene.
    fn mesh_commands<'a>(
        &'a self,
        world: &World,
        assets: &'a AssetStore,
        instance_buffer: &mut InstanceBuffer,
    ) -> Result<Vec<MeshRenderCommand<'a>>, SceneError> {
        let mut mesh_commands = Vec::new();
        for (mesh_id, mesh_instances) in &self.instances_by_mesh {
            let mesh = assets
                .mesh(mesh_id)
                .ok_or(SceneError::MeshNotFound(mesh_id))?;
            let material = assets
                .material(mesh.material)
                .ok_or(SceneError::MaterialNotFound(mesh.material))?;
            let instance_transforms: Vec<RawSpatialTransform> = mesh_instances
                .iter()
                .map(|&inst_id| {
                    let instance = self
                        .mesh_instances
                        .get(inst_id)
                        .ok_or(SceneError::MeshInstanceNotFound(inst_id))?;
                    let entity_transform = world
                        .entity(instance.entity)
                        .ok_or(SceneError::EntityNotFound(instance.entity))?
                        .transform_raw();
                    Ok(entity_transform)
                })
                .collect::<Result<_, SceneError>>()?;
            *instance_buffer.get_mesh_data(mesh_id) = instance_transforms;
            let command = mesh.to_render_command(
                mesh_id,
                material,
                self.pipeline,
                self.camera_bind_group,
                self.lighting_bind_group,
            );
            mesh_commands.push(command);
        }
        Ok(mesh_commands)
    }

    /// Get the sprite commands for a scene.
    fn sprite_commands<'a>(
        &'a self,
        world: &World,
        assets: &'a AssetStore,
        instance_buffer: &mut InstanceBuffer,
    ) -> Result<Vec<SpriteRenderCommand<'a>>, SceneError> {
        let mut sprite_commands = Vec::new();
        for (sprite_id, sprite_instances) in &self.instances_by_sprite {
            let sprite = assets
                .sprite(sprite_id)
                .ok_or(SceneError::SpriteNotFound(sprite_id))?;
            let instance_transforms: Vec<RawSpatialTransform> = sprite_instances
                .iter()
                .map(|&inst_id| {
                    let instance = self
                        .sprite_instances
                        .get(inst_id)
                        .ok_or(SceneError::SpriteInstanceNotFound(inst_id))?;
                    let entity_transform = world
                        .entity(instance.entity)
                        .ok_or(SceneError::EntityNotFound(instance.entity))?
                        .transform_raw();
                    Ok(entity_transform)
                })
                .collect::<Result<_, SceneError>>()?;
            let instance_buffer_range = instance_buffer.add_sprite(instance_transforms, sprite_id);
            let command = sprite.to_render_command(
                sprite_id,
                self.pipeline,
                self.camera_bind_group,
                instance_buffer_range
            );
            sprite_commands.push(command);
        }
        Ok(sprite_commands)
    }
}


/// Error that occurred while converting a scene to render commands.
#[derive(Debug, Error)]
pub enum SceneError {
    #[error("Couldn't find mesh of ID {0:?}")]
    MeshNotFound(MeshId),
    #[error("Couldn't find sprite of ID {0:?}")]
    SpriteNotFound(SpriteId),
    #[error("Couldn't find material of ID {0:?}")]
    MaterialNotFound(MaterialId),
    #[error("Couldn't find mesh instance for ID {0:?}")]
    MeshInstanceNotFound(MeshInstanceId),
    #[error("Couldn't find sprite instance for ID {0:?}")]
    SpriteInstanceNotFound(SpriteInstanceId),
    #[error("Couldn't find the entity of ID {0:?}")]
    EntityNotFound(WorldEntityId)
}
