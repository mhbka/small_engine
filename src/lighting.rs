use crate::gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer};

pub struct Lighting {
    uniform: LightUniform,
    buffer: GpuBuffer,
}

impl Lighting {
    /// Create a lighting, including initializing the uniform buffer for it.
    pub fn create(gpu: &GpuContext, label: &str, position: [f32; 3], color: [f32; 3]) -> Self {
        let uniform = LightUniform::new(position, color);
        let buffer = GpuBuffer::create_uniform(label, gpu, bytemuck::cast_slice(&[uniform]));
        Self { uniform, buffer }
    }

    /// Update this lighting's uniform through a callback.
    pub fn update<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut LightUniform),
    {
        update(&mut self.uniform);
    }

    /// Updates the uniform buffer for this light.
    pub fn update_uniform_buffer(&self, gpu: &GpuContext) {
        gpu.queue().write_buffer(
            self.buffer.handle(),
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    pub fn uniform(&mut self) -> &mut LightUniform {
        &mut self.uniform
    }

    pub fn buffer(&self) -> &GpuBuffer {
        &self.buffer
    }
}

/// Represents a colored point in space.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    _padding: u32, // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here...
    pub color: [f32; 3],
    _padding2: u32, // ...And here
}

impl LightUniform {
    /// Create a light uniform.
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding: 0,
            color,
            _padding2: 0,
        }
    }

    pub fn position(&self) -> &[f32; 3] {
        &self.position
    }

    pub fn color(&self) -> &[f32; 3] {
        &self.color
    }
}

/// Create a bind group for lighting.
pub fn create_lighting_bind_group(gpu: &GpuContext, lighting: &Lighting) -> GpuBindGroup {
    GpuBindGroup::create_default(
        "light_bind_group",
        gpu,
        &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        &[wgpu::BindGroupEntry {
            binding: 0,
            resource: lighting.buffer().handle().as_entire_binding(),
        }],
    )
}
