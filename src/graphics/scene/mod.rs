pub mod camera;
pub mod instance_buffer;
pub mod lighting;
pub mod node;
pub mod spatial_transform;

use std::collections::VecDeque;

use slotmap::{SecondaryMap, SlotMap, new_key_type};
use thiserror::Error;

use crate::graphics::{
    gpu::GpuContext,
    render::{
        assets::{AssetStore, MaterialId, MeshId},
        commands::RenderCommand,
        renderable::{model::MeshInstance, sprite::SpriteInstance},
        renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId},
    },
    scene::{
        camera::Camera,
        instance_buffer::InstanceBuffer,
        lighting::Lighting,
        node::{SceneNode, SceneNodeId},
        spatial_transform::{RawSpatialTransform, SpatialTransform},
    },
};

new_key_type! {
    /// To refer to a mesh instance.
    pub struct MeshInstanceId;
    /// To refer to a sprite instance.
    pub struct SpriteInstanceId;
}

/// The main representation of "something" in the game.
pub struct Scene {
    scene_nodes: SlotMap<SceneNodeId, SceneNode>,
    mesh_instances: SlotMap<MeshInstanceId, MeshInstance>,
    instances_by_mesh: SecondaryMap<MeshId, Vec<MeshInstanceId>>,
    sprite_instances: SlotMap<SpriteInstanceId, SpriteInstance>,
    root_scene_node: SceneNodeId,
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
        let mut scene_nodes = SlotMap::with_key();
        let root_scene_node = scene_nodes.insert(SceneNode::new(
            None,
            vec![],
            SpatialTransform::identity(),
            SpatialTransform::identity(),
        ));
        Self {
            scene_nodes,
            mesh_instances: SlotMap::with_key(),
            instances_by_mesh: SecondaryMap::new(),
            sprite_instances: SlotMap::with_key(),
            root_scene_node,
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
            let instance_transforms: Vec<RawSpatialTransform> = mesh_instances
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
                    let transform = instance_node.transform_raw();
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
        self.camera.write_uniform_buffer(gpu);
        for light in &self.lights {
            light.update_uniform_buffer(gpu);
        }
    }

    /// Add the nodes to the scene, returning their IDs.
    pub fn add_nodes(&mut self, nodes: Vec<SceneNode>) -> Vec<SceneNodeId> {
        nodes
            .into_iter()
            .map(|mut node| {
                if node.parent().is_none() {
                    node.set_parent(self.root_scene_node);
                }
                self.scene_nodes.insert(node)
            })
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

    /// Walks the scene graph and propagates each node's transforms to its children's global transforms.
    fn update_graph(&mut self) {
        let mut node_queue = VecDeque::with_capacity(self.scene_nodes.len());
        node_queue.push_front(self.root_scene_node);
        while !node_queue.is_empty() {
            let cur_node_id = node_queue.pop_back().unwrap();
            let cur_node = self.scene_nodes.get_mut(cur_node_id).unwrap();
            let children = cur_node.children().clone();

            if !cur_node.propagated_global_to_children() {
                cur_node.set_propagated(true);
                let global = cur_node.transform();
                for node in &children {
                    let node = self.scene_nodes.get_mut(*node).unwrap();
                    node.update_global_transform(|old| *old = global);
                    node.set_propagated(false);
                }
            }

            for child in children {
                node_queue.push_front(child);
            }
        }
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
