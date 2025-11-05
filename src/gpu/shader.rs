use crate::gpu::GpuContext;

#[derive(Clone, Debug)]
pub struct GpuShader {
    shader: wgpu::ShaderModule,
}

impl GpuShader {
    /// Initialize from file at compile time.
    pub fn create(gpu: &GpuContext, desc: wgpu::ShaderModuleDescriptor<'_>) -> Self {
        let shader = gpu.device().create_shader_module(desc);
        Self { shader }
    }

    /// Get the actual shader.
    pub fn handle(&self) -> &wgpu::ShaderModule {
        &self.shader
    }
}
