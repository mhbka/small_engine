use crate::graphics::{gpu::{GpuContext, pipeline::GpuPipeline}, render::{pipelines::hdr::HdrPipeline, renderable::model::ModelVertex, renderer::{PipelineId, Renderer, resources::RendererResources}}, scene::instance_buffer::InstanceData, textures::depth::DepthTexture};

pub struct ModelPipeline {
    pipeline: PipelineId
}

impl ModelPipeline {
    /// Instantiate the pipeline used for rendering models,
    /// 
    /// and add it into the renderer's resources.
    pub fn new(
        gpu: &GpuContext,
        resources: &mut RendererResources,
        shader: &wgpu::ShaderModule,
        model_texture_layout: &wgpu::BindGroupLayout,
        camera_layout: &wgpu::BindGroupLayout,
        point_light_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let pipeline = GpuPipeline::create_default(
            "basic_pipeline",
            &gpu,
            &[
                Some(model_texture_layout),
                Some(camera_layout),
                Some(point_light_layout),
            ],
            &[
                ModelVertex::desc(), 
                InstanceData::desc()
            ],
            &shader,
            &shader,
            Some(wgpu::DepthStencilState {
                format: DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            wgpu::PrimitiveTopology::TriangleList,
            HdrPipeline::COLOR_FORMAT
        );

        let pipeline = resources.add_pipelines(vec![pipeline])[0];

        Self {
            pipeline
        }
    }

    /// Get the handle to reference the actual pipeline.
    pub fn inner(&self) -> PipelineId {
        self.pipeline
    }
}