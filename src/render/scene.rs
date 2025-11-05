use crate::{
    camera::Camera,
    gpu::GpuContext,
    lighting::Lighting,
    render::{
        commands::RenderCommand,
        model::Model,
        renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId},
    },
};

pub struct Scene {
    models: Vec<Model>,
    camera: Camera,
    lights: Vec<Lighting>,
    pipeline: PipelineId,
    global_bind_group: GlobalBindGroupId,
    lighting_bind_group: LightingBindGroupId,
}

impl Scene {
    /// Construct a scene.
    pub fn new(
        models: Vec<Model>,
        camera: Camera,
        lights: Vec<Lighting>,
        pipeline: PipelineId,
        global_bind_group: GlobalBindGroupId,
        lighting_bind_group: LightingBindGroupId,
    ) -> Self {
        Self {
            models,
            camera,
            lights,
            pipeline,
            global_bind_group,
            lighting_bind_group,
        }
    }

    /// Convert the scene to render commands.
    pub fn to_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        for model in &self.models {
            for mesh in &model.meshes {
                let material = &model.materials[mesh.material];
                let command = mesh.to_render_command(
                    material,
                    self.pipeline,
                    self.global_bind_group,
                    self.lighting_bind_group,
                );
                commands.push(command);
            }
        }

        commands
    }

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
            // TODO: update buffer for light
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
