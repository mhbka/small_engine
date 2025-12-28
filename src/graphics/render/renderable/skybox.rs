use crate::graphics::{gpu::{GpuContext, pipeline::GpuPipeline}, render::{commands::SkyboxRenderCommand, hdr::HdrPipeline, renderer::{BindGroupId, PipelineId}}, textures::{cube::CubeMapTexture, depth::DepthTexture}};

/// A skybox.
pub struct SkyBox {
    name: String,
    texture: CubeMapTexture
}

impl SkyBox {
    /// Initialize a skybox.
    pub fn new(name: String, texture: CubeMapTexture) -> Self {
        Self { name, texture }
    }

    /// Create a command for rendering this skybox.
    pub fn to_render_command(
        &self,
        sky_pipeline: PipelineId,
        sky_bind_group: BindGroupId,
        camera_bind_group: BindGroupId
    ) -> SkyboxRenderCommand<'_> {
        SkyboxRenderCommand {
            name: &self.name,
            sky_pipeline,
            sky_bind_group,
            camera_bind_group
        }
    }
}