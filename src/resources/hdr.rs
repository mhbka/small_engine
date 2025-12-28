use std::io::Cursor;
use image::codecs::hdr::HdrDecoder;
use crate::graphics::{gpu::{GpuContext, texture::GpuTexture}, textures::{cube::CubeMapTexture, standard::StandardTexture}};

/// Handles loading a 2D HDR image into a cube map.
pub struct HdrLoader {
    format: wgpu::TextureFormat,
    equirect_layout: wgpu::BindGroupLayout,
    equirect_to_cubemap: wgpu::ComputePipeline
}

impl HdrLoader {
    /// Initialize the loader.
    pub fn new(gpu: &GpuContext) -> Self {
        let device = gpu.device();

        let shader = device.create_shader_module(wgpu::include_wgsl!("../equirectangular.wgsl"));
        let format = wgpu::TextureFormat::Rgba32Float;
        let equirect_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("HdrLoader::equirect_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&equirect_layout],
            push_constant_ranges: &[],
        });

        let equirect_to_cubemap = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("equirect_to_cubemap"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("compute_equirect_to_cubemap"),
            cache: None,
            compilation_options: Default::default()
        });

        Self {
            format,
            equirect_layout,
            equirect_to_cubemap
        }
    }

    /// Initialize a cube map from a HDR image.
    pub fn from_equirect_bytes(
        &self,
        gpu: &GpuContext,
        data: &[u8],
        dst_size: u32,
        label: &str
    ) -> anyhow::Result<CubeMapTexture> {
        let hdr_decoder = HdrDecoder::new(Cursor::new(data))?;
        let meta = hdr_decoder.metadata();

        #[cfg(not(target_arch="wasm32"))]
        let pixels = {
            let mut pixels = vec![[0.0, 0.0, 0.0, 0.0]; meta.width as usize * meta.height as usize];
            hdr_decoder.read_image_transform(
                |pix| {
                    let rgb = pix.to_hdr();
                    [rgb.0[0], rgb.0[1], rgb.0[2], 1.0f32]
                },
                &mut pixels[..],
            )?;
            pixels
        };
        #[cfg(target_arch="wasm32")]
        let pixels = hdr_decoder.read_image_native()?
            .into_iter()
            .map(|pix| {
                let rgb = pix.to_hdr();
                [rgb.0[0], rgb.0[1], rgb.0[2], 1.0f32]
            })
            .collect::<Vec<_>>();

        let src = StandardTexture::new(
            gpu, 
            meta.width, 
            meta.height, 
            self.format, 
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            Some(label)
        );
        let src_texture = src.inner().handle();

        gpu.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src.inner().handle(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytemuck::cast_slice(&pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(src_texture.size().width * std::mem::size_of::<[f32; 4]>() as u32),
                rows_per_image: Some(src_texture.size().height),
            },
            src_texture.size(),
        );
        let dst = CubeMapTexture::new(
            gpu, 
            dst_size, 
            dst_size, 
            self.format, 
            // We are going to write to `dst` texture, so we need to use a `STORAGE_BINDING`.
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING, 
            Some(label)
        );
        let dst_view = dst
            .inner()
            .handle()
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some(label),
                // Normally, you'd use `TextureViewDimension::Cube`
                // for a cube texture, but we can't use that
                // view dimension with a `STORAGE_BINDING`.
                // We need to access the cube texture layers
                // directly.
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });

        let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &self.equirect_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src.inner().view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst_view),
                },
            ],
        });

        let mut encoder = gpu.device().create_command_encoder(&Default::default());
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
            label: Some(label), 
            timestamp_writes: None 
        });
        let num_workgroups = (dst_size + 15) / 16;

        pass.set_pipeline(&self.equirect_to_cubemap);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);
        drop(pass);
        gpu.queue().submit([encoder.finish()]);

        Ok(dst)
    }
}