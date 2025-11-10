use crate::gpu::GpuContext;
use wgpu::{BufferDescriptor, util::{BufferInitDescriptor, DeviceExt}};

#[derive(Clone, Debug)]
pub struct GpuBuffer {
    buffer: wgpu::Buffer,
}

impl GpuBuffer {
    /// Create a vertex buffer.
    pub fn create_vertex(label: &str, gpu: &GpuContext, contents: &[u8]) -> Self {
        let buffer = gpu.device().create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self { buffer }
    }

    /// Create a writeable vertex buffer (usually for instances).
    pub fn create_writeable_vertex(label: &str, gpu: &GpuContext, contents: &[u8]) -> Self {
        let buffer = gpu.device().create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self { buffer }
    }
 
    /// Creates a writeable vertex buffer that is uninitialized but has fixed capacity of `size`.
    pub fn create_writeable_vertex_uninit(label: &str, gpu: &GpuContext, size: u64) -> Self {
        let buffer = gpu.device().create_buffer(&BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        Self { buffer }
    }

    /// Create an index buffer.
    pub fn create_index(label: &str, gpu: &GpuContext, contents: &[u8]) -> Self {
        let buffer = gpu.device().create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: wgpu::BufferUsages::INDEX,
        });
        Self { buffer }
    }

    /// Create a (writeable) uniform buffer.
    pub fn create_uniform(label: &str, gpu: &GpuContext, contents: &[u8]) -> Self {
        let buffer = gpu.device().create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self { buffer }
    }

    /// Get the actual buffer.
    pub fn handle(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
