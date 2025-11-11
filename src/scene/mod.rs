pub mod instance_buffer;
pub mod node;
pub mod spacial_transform;

use slotmap::{SecondaryMap, SlotMap};
use thiserror::Error;

use crate::{
    camera::Camera,
    gpu::GpuContext,
    lighting::Lighting,
    render::{
        assets::{AssetStore, MaterialId, MeshId},
        commands::RenderCommand,
        model::instance::{MeshInstance, MeshInstanceId},
        renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId},
    },
    scene::{
        instance_buffer::InstanceBuffer,
        node::{SceneNode, SceneNodeId},
        spacial_transform::RawSpacialTransform,
    },
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
    ///
    /// Writes the scene's meshes' instance data into the `instance_buffer`,
    /// passing their ranges into the render command.
    pub fn to_commands<'a>(
        &'a self,
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
            let instance_transforms: Vec<RawSpacialTransform> = mesh_instances
                .iter()
                .map(|&inst_id| {
                    let instance = self
                        .mesh_instances
                        .get(inst_id)
                        .ok_or(SceneError::MeshInstanceNotFound(inst_id))?;
                    let instance_node = self
                        .scene_nodes
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
                self.lighting_bind_group,
            );
            commands.push(mesh_commands);
        }

        Ok(commands)
    }

    /// Writes any stored "updateable" data to their buffers.
    ///
    /// Currently, this is for the camera and light uniforms.
    pub fn write_buffers(&self, gpu: &GpuContext) {
        self.camera.update_uniform_buffer(gpu);
        for light in &self.lights {
            light.update_uniform_buffer(gpu);
        }
    }

    /// Add the nodes to the scene, returning their IDs.
    pub fn add_nodes(&mut self, nodes: Vec<SceneNode>) -> Vec<SceneNodeId> {
        nodes
            .into_iter()
            .map(|node| self.scene_nodes.insert(node))
            .collect()
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
    #[error("Couldn't find scene node of ID {0:?}")]
    SceneNodeNotFound(SceneNodeId),
}
