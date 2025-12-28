use image::GenericImageView;
use crate::graphics::gpu::{GpuContext, texture::GpuTexture};

/// The bind group layout entries for a diffuse texture.
pub const DIFFUSE_BIND_GROUP_LAYOUT_ENTRIES: [wgpu::BindGroupLayoutEntry; 4] = [
    // diffuse texture = sampler
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    },
    wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // should match the filterable field of the corresponding texture entry above
        count: None,
    },
    // normal texture + sampler
    wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    },
    wgpu::BindGroupLayoutEntry {
        binding: 3,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // should match the filterable field of the corresponding texture entry above
        count: None,
    },
];

/// A normal texture from a 2D image, ie for diffuse/normal textures.
pub struct StandardTexture {
    texture: GpuTexture
}

impl StandardTexture {
    /// Create a standard 2D texture.
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
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[]
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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
    
    /// Creates a diffuse texture from an image.
    pub fn from_image(
        gpu: &GpuContext,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> anyhow::Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let device = gpu.device();
        let queue = gpu.queue();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture = GpuTexture::new(texture, view, sampler);

        Ok(Self { texture })
    }

    /// Get a handle to the texture.
    pub fn inner(&self) -> &GpuTexture {
        &self.texture
    }

    /// Get the default bind group layouts for a texture.
    pub fn bind_group_entries<'a>(
        diffuse_texture: &'a Self,
        normal_texture: &'a Self
    ) -> (
        [wgpu::BindGroupLayoutEntry; 4],
        [wgpu::BindGroupEntry<'a>; 4],
    ) {
        let entries = [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.inner().view()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.inner().sampler()),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal_texture.inner().view()),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&normal_texture.inner().sampler()),
            },
        ];
        (DIFFUSE_BIND_GROUP_LAYOUT_ENTRIES, entries)
    }
}