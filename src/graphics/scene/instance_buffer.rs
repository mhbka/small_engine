use std::ops::Range;
use slotmap::SecondaryMap;
use wgpu::BufferSlice;
use crate::graphics::gpu::{GpuContext, buffer::GpuBuffer};
use crate::graphics::render::assets::SpriteId;
use crate::graphics::scene::MeshId;
use crate::graphics::scene::raw_spatial_transform::RawSpatialTransform;

/// The data per instance. Currently just the spacial transform for it.
pub type InstanceData = RawSpatialTransform;

/// This is a special big vertex buffer, functioning as a single instance buffer for many meshes.
/// 
/// This allows us to use one vertex buffer for all instance data, as an optimization.
/// 
/// The flow is like so:
/// - Set/modify instance data however you like for your entities (mesh/sprite)
/// - Call `write` to write them to the vertex buffer, obtaining a `WrittenInstanceBuffer`
/// - Call `get_mesh/sprite_slice` to get the range of the buffer for that entity (note that these are invalid once you call `write` again)
pub struct InstanceBuffer {
    gpu: GpuContext,
    buffer: GpuBuffer,
    buffer_label: String,
    internal_buffer: Vec<InstanceData>,
    mesh_data: SecondaryMap<MeshId, Vec<InstanceData>>,
    sprite_data: SecondaryMap<SpriteId, Vec<InstanceData>>,
    mesh_ranges: SecondaryMap<MeshId, Range<u32>>,
    sprite_ranges: SecondaryMap<SpriteId, Range<u32>>,
    dirty: bool
}

impl InstanceBuffer {
    /// The initial size of the buffer (in items, not bytes).
    const INITIAL_BUF_SIZE: u64 = 10_000;

    /// Instantiate the buffer.
    pub fn new(gpu: GpuContext, label: String) -> Self {
        let initial_buffer_size = Self::INITIAL_BUF_SIZE * size_of::<InstanceData>() as u64;
        let buffer = GpuBuffer::create_writeable_vertex_uninit(&label, &gpu, initial_buffer_size);
        Self {
            gpu,
            buffer,
            buffer_label: label,
            internal_buffer: Vec::with_capacity(Self::INITIAL_BUF_SIZE as usize),
            mesh_data: SecondaryMap::new(),
            sprite_data: SecondaryMap::new(),
            mesh_ranges: SecondaryMap::new(),
            sprite_ranges: SecondaryMap::new(),
            dirty: false
        }
    }

    /// Get the actual buffer.
    pub fn handle(&self) -> &GpuBuffer {
        &self.buffer
    }

    /// Returns the instance data for this mesh for modification.
    /// 
    /// If there isn't any, instantiates it first.
    pub fn get_mesh_data(&mut self, mesh: MeshId) -> &mut Vec<InstanceData> {
        self.dirty = true;

        if !self.mesh_data.contains_key(mesh) {
        self.mesh_data.insert(mesh, vec![]);
        }
        self.mesh_data.get_mut(mesh).unwrap()
    }

    /// Returns the instance data for this sprite for modification.
    /// 
    /// If there isn't any, instantiates it first.
    pub fn get_sprite_data(&mut self, sprite: SpriteId) -> &mut Vec<InstanceData> {
        self.dirty = true;

        if !self.sprite_data.contains_key(sprite) {
            self.sprite_data.insert(sprite, vec![]);
        }
        self.sprite_data.get_mut(sprite).unwrap()
    }

    /// Writes the internal buffered instance data to the actual GPU buffer.
    ///
    /// You should do this once all your instance data has been written, and you're ready to render.
    pub fn write(&mut self) -> WrittenInstanceBuffer {
        WrittenInstanceBuffer::instantiate(self)
    }
}

/// An instance buffer that has been written to the GPU buffer, obtained by calling `write()` on an `InstanceBuffer`.
/// 
/// Once this is obtained, you cannot modify instance data, as it has already been written.
pub struct WrittenInstanceBuffer<'buffer> {
    instance_buffer: &'buffer mut InstanceBuffer,
}

impl<'buffer> WrittenInstanceBuffer<'buffer> {
    /// Instantiate the written instance buffer.
    /// 
    /// Includes initializing the internal buffer out of all the instance data
    /// and tracking each entity's ranges in the buffer,
    /// and writing the data to the vertex buffer.
    pub fn instantiate(instance_buffer: &'buffer mut InstanceBuffer) -> Self {
        // (re)construct the internal buffer and entity ranges, if dirty
        if instance_buffer.dirty {
            let buf = &mut instance_buffer.internal_buffer;
            buf.clear();

            instance_buffer.mesh_ranges.clear();
            instance_buffer.sprite_ranges.clear();

            for (mesh, data) in &instance_buffer.mesh_data {
                let range = buf.len() as u32 .. (buf.len() + data.len()) as u32;
                instance_buffer.mesh_ranges.insert(mesh, range);
                buf.extend_from_slice(&data);
            }
            for (sprite, data) in &instance_buffer.sprite_data {
                let range = buf.len() as u32 .. (buf.len() + data.len()) as u32;
                instance_buffer.sprite_ranges.insert(sprite, range.into());
                buf.extend_from_slice(&data);
            }

            instance_buffer.dirty = false;
        }

        // if we've overrun the vertex buffer size, destroy and create a new one with double our current size
        let cur_buf_byte_len = instance_buffer.buffer.handle().size();
        let internal_buf_byte_len = (instance_buffer.internal_buffer.len() * size_of::<InstanceData>()) as u64;
        if cur_buf_byte_len < internal_buf_byte_len {
            instance_buffer.buffer.handle().destroy();
            instance_buffer.buffer = GpuBuffer::create_writeable_vertex_uninit(
                &instance_buffer.buffer_label, 
                &instance_buffer.gpu, 
                internal_buf_byte_len * 2
            );
        }

        // write to the vertex buffer
        instance_buffer.gpu.queue().write_buffer(
            instance_buffer.buffer.handle(),
            0,
            &bytemuck::cast_slice(&instance_buffer.internal_buffer),
        );
        instance_buffer.gpu.queue().submit([]);

        Self {
            instance_buffer
        }
    }
    
    /// Get the buffer slice for the given mesh, if it exists.
    pub fn get_mesh_slice(&'buffer self, mesh: MeshId) -> Option<BufferSlice<'buffer>> {
        if let Some(range) = self.instance_buffer.mesh_ranges.get(mesh) {
            let slice = self.instance_buffer.buffer.handle().slice(
                (range.start * size_of::<InstanceData>() as u32) as u64
                    ..(range.end * size_of::<InstanceData>() as u32) as u64,
            );
            Some(slice)
        } else {
            None
        }
    }

    /// Get the buffer slice for the given sprite, if it exists.
    pub fn get_sprite_slice(&'buffer self, sprite: SpriteId) -> Option<BufferSlice<'buffer>> {
        if let Some(range) = self.instance_buffer.sprite_ranges.get(sprite) {
            let slice = self.instance_buffer.buffer.handle().slice(
                (range.start * size_of::<InstanceData>() as u32) as u64
                    ..(range.end * size_of::<InstanceData>() as u32) as u64,
            );
            Some(slice)
        } else {
            None
        }
    }
}
