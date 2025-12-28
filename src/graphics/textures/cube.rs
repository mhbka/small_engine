use crate::graphics::gpu::{GpuContext, texture::GpuTexture};

/// A cube map texture.
pub struct CubeMapTexture {
    texture: GpuTexture 
}

impl CubeMapTexture {
    /// Create a texture for a cube map.
    pub fn new(
        gpu: &GpuContext,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        label: Option<&str>
    ) -> Self {
        let device = gpu.device();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 6, // A cube has 6 sides, so we need 6 layers
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6), // again
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
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