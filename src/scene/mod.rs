pub mod node;
pub mod spacial_transform;
pub mod instance_buffer;

use slotmap::{SecondaryMap, SlotMap};
use thiserror::Error;

use crate::{
    camera::Camera,
    gpu::{GpuContext, buffer::GpuBuffer},
    lighting::Lighting,
    render::{
        assets::{AssetStore, MaterialId, MeshId}, commands::RenderCommand, model::{Material, Mesh, Model, instance::{MeshInstance, MeshInstanceId}}, renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId}
    }, scene::{instance_buffer::InstanceBuffer, node::{SceneNode, SceneNodeId}, spacial_transform::RawSpacialTransform},
};

/// The main representation of "something" in the game.
pub struct Scene {
    scene_nodes: SlotMap<SceneNodeId, SceneNode>,
    mesh_instances: SlotMap<MeshInstanceId, MeshInstance>,
    instances_by_mesh: SecondaryMap<MeshId, Vec<MeshInstanceId>>,
    camera: Camera,
    lights: Vec<Lighting>,
    pipeline: PipelineId,
    global_bind_group: GlobalBindGroupId,
    lighting_bind_group: LightingBindGroupId,
}

impl Scene {
    /// Construct a scene.
    pub fn new(
        label: &str,
        gpu: GpuContext,
        camera: Camera,
        lights: Vec<Lighting>,
        pipeline: PipelineId,
        global_bind_group: GlobalBindGroupId,
        lighting_bind_group: LightingBindGroupId,
    ) -> Self {
        Self {
            scene_nodes: SlotMap::with_key(),
            mesh_instances: SlotMap::with_key(),
            instances_by_mesh: SecondaryMap::new(),
            camera,
            lights,
            pipeline,
            global_bind_group,
            lighting_bind_group,
        }
    }

    /// Convert the scene to render commands.
    pub fn to_commands(
        &mut self, 
        assets: &AssetStore,
        instance_buffer: &mut InstanceBuffer
    ) -> Result<Vec<RenderCommand>, SceneError> {
        let mut commands = Vec::new();

        for (mesh_id, mesh_instances) in &self.instances_by_mesh {
            let mesh = assets
                .mesh(mesh_id)
                .ok_or(SceneError::MeshNotFound(mesh_id))?;
            let material = assets
                .material(mesh.material)
                .ok_or(SceneError::MaterialNotFound(mesh.material))?;
            let instance_transforms: Vec<RawSpacialTransform> = mesh_instances
                .iter()
                .map(|&inst_id| {
                    let instance = self.mesh_instances
                        .get(inst_id)
                        .ok_or(SceneError::MeshInstanceNotFound(inst_id))?;
                    let instance_node = self.scene_nodes
                        .get(instance.node)
                        .ok_or(SceneError::SceneNodeNotFound(instance.node))?;
                    let transform = instance_node.get_transform();
                    Ok(transform)
                })
                .collect::<Result<_, SceneError>>()?;
            let instance_buffer_range = instance_buffer.add(instance_transforms, mesh_id);
            let mesh_commands = mesh.to_render_command(
                mesh_id,
                material, 
                self.pipeline, 
                instance_buffer_range, 
                self.global_bind_group, 
                self.lighting_bind_group
            );
            commands.push(mesh_commands);
        }

        Ok(commands)
    }

    /// Get the instance buffer for the scene.
    /// 
    /// For the renderer to use.
    pub fn instance_buffer(&self) -> &InstanceBuffer { &self.instance_buffer }

    /// Writes any stored "updateable" data to their buffers.
    ///
    /// Currently, this is for the camera, each mesh's instances, and light uniforms.
    pub fn update_buffers(&self, gpu: &GpuContext) {
        self.camera.update_uniform_buffer(gpu);
        for model in &self.models {
            for mesh in &model.meshes {
                mesh.update_instance_buffer(gpu);
            }
        }
        for light in &self.lights {
            light.update_uniform_buffer(gpu);
        }
    }

    /// Get the camera.
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Get the models.
    pub fn models(&mut self) -> &mut Vec<Model> {
        &mut self.models
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
    #[error("Couldn't find scene node of ID {0:?}")]
    SceneNodeNotFound(SceneNodeId),
}