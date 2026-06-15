use crate::graphics::{gpu::{GpuContext, pipeline::GpuPipeline}, render::{pipelines::hdr::HdrPipeline, renderer::{PipelineId, Renderer, resources::RendererResources}}, textures::depth::DepthTexture};

pub struct SkyboxPipeline {
    pipeline: PipelineId
}

impl SkyboxPipeline {
    /// Instantiate the pipeline used for rendering a skybox,
    /// 
    /// and add it into the renderer's resources.
    pub fn new(
        gpu: &GpuContext,
        resources: &mut RendererResources,
        skybox_shader: &wgpu::ShaderModule,
        camera_layout: &wgpu::BindGroupLayout,
        skybox_layout: &wgpu::BindGroupLayout
    ) -> Self {
        let depth_stencil = wgpu::DepthStencilState {
            format: DepthTexture::DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(),
            bias: Default::default(),
        };
        let sky_pipeline = GpuPipeline::create_default(
            "skybox_pipeline",
            &gpu,
            &[
                Some(camera_layout), 
                Some(skybox_layout)
            ],
            &[],
            &skybox_shader,
            &skybox_shader,
            Some(depth_stencil),
            wgpu::PrimitiveTopology::TriangleList,
            HdrPipeline::COLOR_FORMAT,
        );

        let pipeline = resources.add_pipelines(vec![sky_pipeline])[0];

        Self {
            pipeline
        }
    }

    /// Get the handle to reference the actual pipeline.
    pub fn inner(&self) -> PipelineId {
        self.pipeline
    }
}