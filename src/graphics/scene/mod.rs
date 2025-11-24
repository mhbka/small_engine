pub mod instance_buffer;
pub mod lighting;
pub mod raw_spatial_transform;

use slotmap::{SecondaryMap, SlotMap, new_key_type};
use thiserror::Error;
use crate::{core::world::{World, WorldEntityId}, graphics::{
    gpu::GpuContext,
    render::{
        assets::{AssetStore, MaterialId, MeshId},
        commands::RenderCommand,
        renderable::{model::MeshInstance, sprite::SpriteInstance},
        renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId},
    },
    scene::{
        instance_buffer::InstanceBuffer,
        lighting::Lighting,
        raw_spatial_transform::RawSpatialTransform,
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
    instances_by_mesh: SecondaryMap<MeshId, Vec<MeshInstanceId>>,
    sprite_instances: SlotMap<SpriteInstanceId, SpriteInstance>,
    camera: Camera,
    lights: Vec<Lighting>,
    pipeline: PipelineId,
    global_bind_group: GlobalBindGroupId,
    lighting_bind_group: LightingBindGroupId,
}

impl Scene {
    /// Construct a scene.
    pub fn new(
        camera: Camera,
        lights: Vec<Lighting>,
        pipeline: PipelineId,
        global_bind_group: GlobalBindGroupId,
        lighting_bind_group: LightingBindGroupId,
    ) -> Self {
        Self {
            mesh_instances: SlotMap::with_key(),
            instances_by_mesh: SecondaryMap::new(),
            sprite_instances: SlotMap::with_key(),
            camera,
            lights,
            pipeline,
            global_bind_group,
            lighting_bind_group,
        }
    }

    /// Convert the scene to render commands.
    ///
    /// Writes the scene's meshes' instance data into the `instance_buffer`,
    /// passing their ranges into the render command.
    pub fn to_commands<'a>(
        &'a self,
        world: &World,
        assets: &'a AssetStore,
        instance_buffer: &mut InstanceBuffer,
    ) -> Result<Vec<RenderCommand<'a>>, SceneError> {
        let mut commands = Vec::new();

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
                    let entity = world
                        .entity(instance.entity)
                        .ok_or(SceneError::EntityNotFound(instance.entity))?;
                    Ok(
                        entity.transform_raw()
                    )
                })
                .collect::<Result<_, SceneError>>()?;
            let instance_buffer_range = instance_buffer.add(instance_transforms, mesh_id);
            let mesh_commands = mesh.to_render_command(
                mesh_id,
                material,
                self.pipeline,
                instance_buffer_range,
                self.global_bind_group,
                self.lighting_bind_group,
            );
            commands.push(mesh_commands);
        }

        Ok(commands)
    }

    /// Updates and writes updateable buffers.
    ///
    /// Currently, this is for the camera and light uniforms.
    pub fn update_and_write_buffers(&mut self, world: &World, gpu: &GpuContext) {
        self.camera.update_and_write_uniform_buffer(world, gpu);
        for light in &self.lights {
            light.update_uniform_buffer(gpu);
        }
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

    /// Get the camera.
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Get the lighting.
    pub fn lights(&mut self) -> &mut Vec<Lighting> {
        &mut self.lights
    }
}

#[derive(Debug, Error)]
pub enum SceneError {
    #[error("Couldn't find mesh of ID {0:?}")]
    MeshNotFound(MeshId),
    #[error("Couldn't find material of ID {0:?}")]
    MaterialNotFound(MaterialId),
    #[error("Couldn't find mesh instance for ID {0:?}")]
    MeshInstanceNotFound(MeshInstanceId),
    #[error("Couldn't find the entity of ID {0:?}")]
    EntityNotFound(WorldEntityId)
}
