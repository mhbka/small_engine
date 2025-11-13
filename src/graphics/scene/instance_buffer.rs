use slotmap::SecondaryMap;
use wgpu::BufferSlice;

use crate::graphics::gpu::{GpuContext, buffer::GpuBuffer};
use crate::graphics::scene::MeshId;
use crate::graphics::scene::spacial_transform::RawSpatialTransform;

/// The data per instance. Currently just the spacial transform for it.
pub type MeshInstanceData = RawSpatialTransform;

/// Describes the range for a mesh's instance data within the entire buffer.
///
/// ## Note
/// This is in terms of `MeshInstanceData`, not bytes. Thus the total number of instances
/// can be calculated from `end - start`.
#[derive(Clone, Copy)]
pub struct InstanceBufferRange {
    pub start: u64,
    pub end: u64,
}

/// This is a special big vertex buffer, functioning as a single instance buffer for many meshes.
pub struct InstanceBuffer {
    gpu: GpuContext,
    buffer: GpuBuffer,
    buffer_label: String,
    buffer_data: Vec<MeshInstanceData>,
    buffer_size: u64,
    mesh_ranges: SecondaryMap<MeshId, InstanceBufferRange>,
}

impl InstanceBuffer {
    /// The initial size of the buffer (in items, not bytes).
    const INITIAL_BUF_SIZE: u64 = 10_000;

    /// Instantiate the buffer.
    pub fn new(gpu: GpuContext, label: String) -> Self {
        let initial_buffer_size = Self::INITIAL_BUF_SIZE * size_of::<MeshInstanceData>() as u64;
        let buffer = GpuBuffer::create_writeable_vertex_uninit(&label, &gpu, initial_buffer_size);
        Self {
            gpu,
            buffer,
            buffer_label: label,
            buffer_data: Vec::with_capacity(Self::INITIAL_BUF_SIZE as usize),
            buffer_size: Self::INITIAL_BUF_SIZE,
            mesh_ranges: SecondaryMap::new(),
        }
    }

    /// Get the actual buffer.
    pub fn handle(&self) -> &GpuBuffer {
        &self.buffer
    }

    /// Clear the mappings (ie for a new frame).
    pub fn clear(&mut self) {
        self.mesh_ranges.clear();
        self.buffer_data.clear();
    }

    /// Add the given data to the internal Vec + create a mapping for it.
    pub fn add(&mut self, data: Vec<MeshInstanceData>, mesh: MeshId) -> InstanceBufferRange {
        // create new gpu buffer with double the size when we've maxed it out
        let required_size = (self.buffer_data.len() + data.len()) as u64;
        if required_size > self.buffer_size {
            self.buffer.handle().destroy();
            self.buffer = GpuBuffer::create_writeable_vertex_uninit(
                &self.buffer_label,
                &self.gpu,
                self.buffer_size * 2,
            );
            self.buffer_size *= 2;
        }

        let range = InstanceBufferRange {
            start: self.buffer_data.len() as u64,
            end: (self.buffer_data.len() + data.len()) as u64,
        };
        self.mesh_ranges.insert(mesh, range.clone());
        self.buffer_data.extend_from_slice(&data);

        range
    }

    /// Writes the internal buffered instance data to the actual GPU buffer.
    ///
    /// You should do this once all your instance data has been written,
    /// and you're ready to render.
    /// 
    /// ## Panic
    /// Panics if the buffer data is somehow larger than the buffer size.
    pub fn write(&self) {
        if self.buffer.handle().size() < (self.buffer_data.len() * size_of::<MeshInstanceData>()) as u64 {
            panic!("Instance buffer data is larger than buffer's capacity!");
        }

        self.gpu.queue().write_buffer(
            self.buffer.handle(),
            0,
            &bytemuck::cast_slice(&self.buffer_data),
        );
        self.gpu.queue().submit([]);
    }

    /// Get the buffer slice for the given mesh, if it exists.
    ///
    /// ## Note
    /// This becomes invalid when the instance buffer is cleared.
    pub fn get_slice(&self, mesh: MeshId) -> Option<BufferSlice<'_>> {
        if let Some(range) = self.mesh_ranges.get(mesh) {
            let slice = self.buffer.handle().slice(
                range.start * size_of::<MeshInstanceData>() as u64
                    ..range.end * size_of::<MeshInstanceData>() as u64,
            );
            Some(slice)
        } else {
            None
        }
    }
}
