use crate::graphics::{
    render::{
        assets::{MeshId, SpriteId},
        renderer::{BindGroupId, PipelineId},
    },
    scene::instance_buffer::InstanceBufferRange,
};

/// The render commands.
pub struct RenderCommandBuffer<'obj> {
    pub mesh: Vec<MeshRenderCommand<'obj>>,
    pub sprite: Vec<SpriteRenderCommand<'obj>>,
    pub skybox: Option<SkyboxRenderCommand<'obj>>,
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
    pub index_buffer: wgpu::BufferSlice<'obj>,
}

/// A command describing how to render a skybox.
pub struct SkyboxRenderCommand<'obj> {
    pub name: &'obj str,
    pub sky_pipeline: PipelineId,
    pub sky_bind_group: BindGroupId,
    pub camera_bind_group: BindGroupId
}

/// A command describing how to render a sprite.
pub struct SpriteRenderCommand<'obj> {
    pub name: &'obj str,
    pub sprite: SpriteId,
    pub pipeline: PipelineId,
    pub camera_bind_group: BindGroupId,
    pub sprite_bind_group: BindGroupId,
    pub instance_buffer_range: InstanceBufferRange,
}
