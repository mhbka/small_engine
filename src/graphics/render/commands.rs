use crate::graphics::{
    gpu::bind_group::GpuBindGroup,
    render::{
        assets::MeshId,
        renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId},
    },
    scene::instance_buffer::InstanceBufferRange,
};
use std::ops::Range;

/// The types of render commands.
pub enum RenderCommand<'obj> {
    Mesh(MeshRenderCommand<'obj>),
}

/// A command describing how to render a mesh.
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
    pub draw: DrawCommand,
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
