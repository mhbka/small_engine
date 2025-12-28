use crate::graphics::{gpu::{GpuContext, bind_group::GpuBindGroup, pipeline::GpuPipeline, texture::GpuTexture}, textures::standard::StandardTexture};

/// Render pipeline and texture for HDR/tonemapping.
pub struct HdrPipeline {
    pipeline: GpuPipeline,
    bind_group: GpuBindGroup,
    texture: StandardTexture,
    width: u32,
    height: u32,
}

impl HdrPipeline {
    /// The color format for HDR.
    pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    const BIND_GROUP_LAYOUT_ENTRIES: [wgpu::BindGroupLayoutEntry; 2] = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture { 
                sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                view_dimension: wgpu::TextureViewDimension::D2, 
                multisampled: false 
            },
            count: None
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ];

    /// Initialize the HDR pipeline.
    pub fn new(gpu: &GpuContext, config: &wgpu::SurfaceConfiguration) -> Self {
        let width = config.width;
        let height = config.height;

        let texture = StandardTexture::new(
            gpu, 
            width, 
            height, 
            Self::COLOR_FORMAT, 
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, 
            Some("Hdr::texture")
        );

        let bind_group = GpuBindGroup::create_default(
            "Hdr::bind_group", 
            gpu, 
            &Self::BIND_GROUP_LAYOUT_ENTRIES, 
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.inner().view())
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture.inner().sampler())
                }
            ]
        );

        let shader = gpu.device().create_shader_module(wgpu::include_wgsl!("../../hdr.wgsl"));
        let pipeline = GpuPipeline::create_default(
            "Hdr::pipeline", 
            gpu, 
            &[bind_group.layout()], 
            &[], // we generate vertex data directly in the shader 
            &shader, 
            &shader, 
            None,
            wgpu::PrimitiveTopology::TriangleList,
            config.format.add_srgb_suffix()
        );

        Self {
            pipeline,
            bind_group,
            texture,
            width,
            height,
        }
    }

    /// Resize the HDR texture.
    pub fn resize(&mut self, gpu: &GpuContext, width: u32, height: u32) {
        self.texture = StandardTexture::new(
            gpu, 
            width, 
            height, 
            Self::COLOR_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, 
            Some("Hdr::texture")
        );
        self.bind_group = GpuBindGroup::create_default(
            "Hdr::bind_group", 
            gpu, 
            &Self::BIND_GROUP_LAYOUT_ENTRIES, 
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(self.texture.inner().view())
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(self.texture.inner().sampler())
                }
            ]
        );
        self.width = width;
        self.height = height;
    }

    /// Renders the HDR texture to the supplied texture view.
    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
            label: Some("Hdr::render_pass"), 
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store
                    },
                    depth_slice: None
                })
            ], 
            depth_stencil_attachment: None, 
            timestamp_writes: None, 
            occlusion_query_set: None 
        });
        pass.set_pipeline(self.pipeline.handle());
        pass.set_bind_group(0, self.bind_group.handle(), &[]);
        pass.draw(0..3, 0..1);
    }

    /// Get the inner texture.
    pub fn texture(&self) -> &GpuTexture {
        self.texture.inner()
    }
}