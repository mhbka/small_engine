use crate::graphics::{
    gpu::bind_group::GpuBindGroup,
    render::{
        assets::MeshId,
        renderer::{BindGroupId, PipelineId},
    },
    scene::instance_buffer::InstanceBufferRange,
};
use std::ops::Range;

/// The render commands.
pub struct RenderCommandBuffer<'obj> {
    pub mesh: Vec<MeshRenderCommand<'obj>>,
    pub skybox: Option<SkyboxRenderCommand<'obj>>
}

/// A command describing how to render a mesh.
pub struct MeshRenderCommand<'obj> {
    pub name: &'obj str,
    pub mesh: MeshId,
    pub pipeline: PipelineId,
    pub camera_bind_group: BindGroupId,
    pub lighting_bind_group: BindGroupId,
    pub material_bind_group: BindGroupId,
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

/// A command describing how to render a skybox.
pub struct SkyboxRenderCommand<'obj> {
    pub name: &'obj str,
    pub sky_pipeline: PipelineId,
    pub sky_bind_group: BindGroupId,
    pub camera_bind_group: BindGroupId
}
