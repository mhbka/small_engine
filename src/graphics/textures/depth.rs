use crate::graphics::gpu::{GpuContext, texture::GpuTexture};

/// A depth texture.
pub struct DepthTexture {
    texture: GpuTexture
}

impl DepthTexture {
    /// The format for depth textures.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a depth texture.
    pub fn new(
        gpu: &GpuContext,
        label: &str,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let device = gpu.device();

        let size = wgpu::Extent3d {
            width: surface_config.width.max(1),
            height: surface_config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let texture = GpuTexture::new(texture, view, sampler);

        Self { texture }
    }

    /// Get a handle to the texture.
    pub fn inner(&self) -> &GpuTexture {
        &self.texture
    }
}