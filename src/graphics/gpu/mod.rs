pub mod bind_group;
pub mod buffer;
pub mod pipeline;
pub mod shader;
pub mod texture;

/// Abstraction over GPU-related data.
#[derive(Clone, Debug)]
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GpuContext {
    /// Instantiate.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self { device, queue }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
