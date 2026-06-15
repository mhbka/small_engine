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
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let texture = GpuTexture::new(texture, view, sampler);
        Self { texture }
    }

    /// Get a handle to the texture.
    pub fn inner(&self) -> &GpuTexture {
        &self.texture
    }

    /// Get the bind group entries used for a `CubeMapTexture`.
    pub fn bind_group_entries<'a>(
        cube_texture: &'a Self
    ) -> (
        [wgpu::BindGroupLayoutEntry; 2],
        [wgpu::BindGroupEntry<'a>; 2]
    ) {
        let layout_entries = [
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];
        let entries = [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(cube_texture.inner().view()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(cube_texture.inner().sampler()),
            },
        ];
        return (layout_entries, entries);
    }
}