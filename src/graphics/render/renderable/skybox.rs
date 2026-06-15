use crate::graphics::{gpu::{GpuContext, pipeline::GpuPipeline}, render::{commands::SkyboxRenderCommand, renderer::{BindGroupId, PipelineId}}, textures::{cube::CubeMapTexture, depth::DepthTexture}};

/// A skybox.
pub struct Skybox {
    name: String,
    texture: CubeMapTexture,
    bind_group: BindGroupId,
}

impl Skybox {
    /// Initialize a skybox.
    pub fn new(name: String, texture: CubeMapTexture, bind_group: BindGroupId) -> Self {
        Self { name, texture, bind_group }
    }

    /// Create a command for rendering this skybox.
    pub fn to_render_command(
        &self,
        sky_pipeline: PipelineId,
        camera_bind_group: BindGroupId
    ) -> SkyboxRenderCommand<'_> {
        SkyboxRenderCommand {
            name: &self.name,
            sky_pipeline,
            sky_bind_group: self.bind_group,
            camera_bind_group
        }
    }

    /// Get the bind group.
    pub fn bind_group(&self) -> BindGroupId {
        self.bind_group
    }
}