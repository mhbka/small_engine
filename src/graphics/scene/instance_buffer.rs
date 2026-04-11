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
/// This allows us to use one vertex buffer for all instance data, as an optimization.
/// 
/// The flow is like so:
/// - Set/modify instance data however you like for your entities (mesh/sprite)
/// - Call `write` to write them to the vertex buffer as a contiguous piece of data
/// - Call `get_mesh/sprite_slice` to get the range of the buffer for that entity (note that these are invalid once you call `write` again)
pub struct InstanceBuffer {
    gpu: GpuContext,
    buffer: GpuBuffer,
    buffer_size: u64,
    buffer_label: String,
    internal_buffer: Vec<MeshInstanceData>,
    mesh_data: SecondaryMap<MeshId, Vec<MeshInstanceData>>,
    sprite_data: SecondaryMap<SpriteId, Vec<MeshInstanceData>>,
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
            internal_buffer: Vec::with_capacity(Self::INITIAL_BUF_SIZE as usize),
            buffer_size: Self::INITIAL_BUF_SIZE,
            mesh_data: SecondaryMap::new(),
            sprite_data: SecondaryMap::new(),
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
        self.internal_buffer.clear();
    }

    /// Returns the instance data for this mesh for modification.
    /// 
    /// If there isn't any, instantiates it first.
    pub fn get_mesh_data(&mut self, mesh: MeshId) -> &mut Vec<MeshInstanceData> {
        if !self.mesh_data.contains_key(mesh) {
        self.mesh_data.insert(mesh, vec![]);
        }
        self.mesh_data.get_mut(mesh).unwrap()
    }

    /// Returns the instance data for this sprite for modification.
    /// 
    /// If there isn't any, instantiates it first.
    pub fn get_sprite_data(&mut self, sprite: SpriteId) -> &mut Vec<MeshInstanceData> {
        if !self.sprite_data.contains_key(sprite) {
            self.sprite_data.insert(sprite, vec![]);
        }
        self.sprite_data.get_mut(sprite).unwrap()
    }

    /// Writes the internal buffered instance data to the actual GPU buffer.
    ///
    /// You should do this once all your instance data has been written, and you're ready to render.
    pub fn write(&self) {
        if self.buffer.handle().size()
            < ((self.internal_buffer.len() * size_of::<MeshInstanceData>())).try_into().unwrap()
        {
            panic!("Instance buffer data is larger than buffer's capacity! (this shouldn't happen)");
        }

        self.gpu.queue().write_buffer(
            self.buffer.handle(),
            0,
            &bytemuck::cast_slice(&self.internal_buffer),
        );
        self.gpu.queue().submit([]);
    }

    /// Get the buffer slice for the given mesh, if it exists.
    ///
    /// ## Note
    /// This becomes invalid after your next `write` call.
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
    /// This becomes invalid after your next `write` call.
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
        let required_size = (self.internal_buffer.len() + new_data_len) as u64;
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
