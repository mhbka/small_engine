use crate::{
    gpu::bind_group::GpuBindGroup,
    render::{assets::MeshId, renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId}}, scene::instance_buffer::InstanceBufferRange,
};
use std::ops::Range;

/// The types of render commands.
pub enum RenderCommand<'obj> {
    Raw(RawRenderCommand<'obj>),
    Basic(BasicRenderCommand<'obj>),
    Mesh(MeshRenderCommand<'obj>)
}

/// An escape hatch render command where you can describe the bind groups and buffers to use, without the global bind group.
pub struct RawRenderCommand<'obj> {
    pub pipeline: PipelineId,
    pub bind_group_commands: Vec<RawBindGroupCommand>,
    pub vertex_buffers: Vec<VertexBufferCommand<'obj>>,
    pub index_buffer: Option<(wgpu::BufferSlice<'obj>, wgpu::IndexFormat)>,
    pub draw: DrawCommand,
}

/// A render command where the structure of the object's bind groups and buffers can be freely described.
pub struct BasicRenderCommand<'obj> {
    pub pipeline: PipelineId,
    pub global_bind_group: GlobalBindGroupId,
    pub lighting_bind_group: LightingBindGroupId,
    pub object_bind_group: GpuBindGroup,
    pub extra_bind_groups: Vec<GpuBindGroup>,
    pub vertex_buffers: Vec<VertexBufferCommand<'obj>>,
    pub index_buffer: Option<(wgpu::BufferSlice<'obj>, wgpu::IndexFormat)>,
    pub draw: DrawCommand,
}

/// A more restrictive render command that properly describes rendering of a mesh.
pub struct MeshRenderCommand<'obj> {
    pub mesh: MeshId,
    pub pipeline: PipelineId,
    pub global_bind_group: GlobalBindGroupId,
    pub lighting_bind_group: LightingBindGroupId,
    pub object_bind_group: GpuBindGroup,
    pub extra_bind_groups: Vec<GpuBindGroup>,
    pub vertex_buffer: wgpu::BufferSlice<'obj>,
    pub instance_buffer_range: InstanceBufferRange,
    pub index_buffer: wgpu::BufferSlice<'obj>,
    pub draw: DrawCommand
}

/// Represents a command for setting a bind group manually.
pub struct RawBindGroupCommand {
    pub slot: u32,
    pub group: GpuBindGroup,
    pub offsets: Vec<wgpu::DynamicOffset>,
}

/// Represents a command for setting a vertex buffer manually.
pub struct VertexBufferCommand<'obj> {
    pub slot: u32,
    pub buffer: wgpu::BufferSlice<'obj>,
}

/// What kind of drawing the render should do.
#[derive(Clone)]
pub enum DrawCommand {
    NonIndexed {
        vertices: Range<u32>,
        instances: Range<u32>,
    },
    Indexed {
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    },
}
