use std::ops::Range;

use slotmap::SecondaryMap;
use wgpu::BufferSlice;

use crate::graphics::gpu::{GpuContext, buffer::GpuBuffer};
use crate::graphics::render::assets::SpriteId;
use crate::graphics::scene::MeshId;
use crate::graphics::scene::raw_spatial_transform::RawSpatialTransform;

/// The data per instance. Currently just the spacial transform for it.
pub type MeshInstanceData = RawSpatialTransform;

/// Describes the range for a mesh's instance data within the entire buffer.
/// 
/// This allows us to allocate one big buffer for all instance data, 
/// and pass out ranges for each mesh/sprite to use in rendering.  
///
/// ## Note
/// This is in terms of `MeshInstanceData`, not bytes. Thus the total number of instances
/// can be calculated from `end - start`.
#[derive(Clone, Copy)]
pub struct InstanceBufferRange {
    pub start: u32,
    pub end: u32,
}

impl InstanceBufferRange {
    /// Get a `Range` of this range.
    pub fn range(&self) -> Range<u32> {
        self.start..self.end
    }
}

/// This is a special big vertex buffer, functioning as a single instance buffer for many meshes.
/// 
/// This is so that we can just use slices out of this 1 buffer for many meshes, apparently an optimization.
pub struct InstanceBuffer {
    gpu: GpuContext,
    buffer: GpuBuffer,
    buffer_label: String,
    buffer_data: Vec<MeshInstanceData>,
    buffer_size: u64,
    mesh_ranges: SecondaryMap<MeshId, InstanceBufferRange>,
    sprite_ranges: SecondaryMap<SpriteId, InstanceBufferRange>,
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
            sprite_ranges: SecondaryMap::new()
        }
    }

    /// Get the actual buffer.
    pub fn handle(&self) -> &GpuBuffer {
        &self.buffer
    }

    /// Clear the buffer data and mappings.
    pub fn clear(&mut self) {
        self.mesh_ranges.clear();
        self.buffer_data.clear();
    }

    /// Add the given data to internal buffer + create a mapping for it.
    pub fn add_mesh(&mut self, data: Vec<MeshInstanceData>, mesh: MeshId) -> InstanceBufferRange {
        self.check_and_extend_buf_size(data.len());

        let range = InstanceBufferRange {
            start: self.buffer_data.len() as u32,
            end: (self.buffer_data.len() + data.len()) as u32,
        };
        self.mesh_ranges.insert(mesh, range.clone());
        self.buffer_data.extend_from_slice(&data);

        range
    }

    /// Add the given data to internal buffer + create a mapping for it as a sprite.
    pub fn add_sprite(&mut self, data: Vec<MeshInstanceData>, sprite: SpriteId) -> InstanceBufferRange {
        self.check_and_extend_buf_size(data.len());

        let range = InstanceBufferRange {
            start: self.buffer_data.len() as u32,
            end: (self.buffer_data.len() + data.len()) as u32,
        };
        self.sprite_ranges.insert(sprite, range.clone());
        self.buffer_data.extend_from_slice(&data);

        range
    }

    /// Writes the internal buffered instance data to the actual GPU buffer.
    ///
    /// You should do this once all your instance data has been written, and you're ready to render.
    pub fn write(&self) {
        if self.buffer.handle().size()
            < ((self.buffer_data.len() * size_of::<MeshInstanceData>())).try_into().unwrap()
        {
            panic!("Instance buffer data is larger than buffer's capacity! (this shouldn't happen)");
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
    pub fn get_mesh_slice(&self, mesh: MeshId) -> Option<BufferSlice<'_>> {
        if let Some(range) = self.mesh_ranges.get(mesh) {
            let slice = self.buffer.handle().slice(
                (range.start * size_of::<MeshInstanceData>() as u32) as u64
                    ..(range.end * size_of::<MeshInstanceData>() as u32) as u64,
            );
            Some(slice)
        } else {
            None
        }
    }

    /// Get the buffer slice for the given sprite, if it exists.
    ///
    /// ## Note
    /// This becomes invalid when the instance buffer is cleared.
    pub fn get_sprite_slice(&self, sprite: SpriteId) -> Option<BufferSlice<'_>> {
        if let Some(range) = self.sprite_ranges.get(sprite) {
            let slice = self.buffer.handle().slice(
                (range.start * size_of::<MeshInstanceData>() as u32) as u64
                    ..(range.end * size_of::<MeshInstanceData>() as u32) as u64,
            );
            Some(slice)
        } else {
            None
        }
    }

    /// Create a new gpu buffer with double the size if we've maxed it out.
    fn check_and_extend_buf_size(&mut self, new_data_len: usize) {
        let required_size = (self.buffer_data.len() + new_data_len) as u64;
        if required_size > self.buffer_size {
            self.buffer.handle().destroy();
            self.buffer = GpuBuffer::create_writeable_vertex_uninit(
                &self.buffer_label,
                &self.gpu,
                self.buffer_size * 2,
            );
            self.buffer_size *= 2;
        }
    }
}
