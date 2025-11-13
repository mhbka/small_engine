use crate::graphics::gpu::GpuContext;

/// Abstraction of the render pipeline.
#[derive(Clone, Debug)]
pub struct GpuPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl GpuPipeline {
    /// Creates a render pipeline with mostly default configs.
    pub fn create_default(
        label: &str,
        gpu: &GpuContext,
        surface_config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
        vertex_shader: &wgpu::ShaderModule,
        fragment_shader: &wgpu::ShaderModule,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Self {
        let device = gpu.device();

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{label}_layout")),
            bind_group_layouts,
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: vertex_shader,
                entry_point: None, // if we have >1 vertex shader, I think we must specify this?
                buffers: vertex_buffer_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: fragment_shader,
                entry_point: None, // if we have >1 fragment shader, I think we must specify this?
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self { pipeline }
    }

    /// Get the actual pipeline.
    pub fn handle(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
