pub mod instance;

use crate::{
    gpu::{bind_group::GpuBindGroup, buffer::GpuBuffer, texture::GpuTexture},
    render::{
        assets::{MaterialId, MeshId},
        commands::{DrawCommand, MeshRenderCommand, RenderCommand},
        renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId},
    },
    scene::instance_buffer::InstanceBufferRange,
};

/// Represents something that can be rendered.
pub struct Model {
    pub meshes: Vec<MeshId>,
    pub materials: Vec<MaterialId>,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: GpuTexture,
    pub bind_group: GpuBindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub material: MaterialId,
    pub num_elements: u32,
}

impl Mesh {
    /// Create a render command for rendering this mesh.
    pub fn to_render_command<'buf>(
        &'buf self,
        id: MeshId,
        material: &Material,
        pipeline: PipelineId,
        instance_buffer_range: InstanceBufferRange,
        global_bind_group: GlobalBindGroupId,
        lighting_bind_group: LightingBindGroupId,
    ) -> RenderCommand<'buf> {
        let command = MeshRenderCommand {
            mesh: id,
            pipeline,
            global_bind_group,
            lighting_bind_group,
            object_bind_group: material.bind_group.clone(),
            extra_bind_groups: vec![],
            vertex_buffer: self.vertex_buffer.handle().slice(..),
            instance_buffer_range: instance_buffer_range,
            index_buffer: self.index_buffer.handle().slice(..),
            draw: DrawCommand::Indexed {
                base_vertex: 0,
                instances: 0..(instance_buffer_range.end - instance_buffer_range.start) as u32,
                indices: 0..self.num_elements,
            },
        };

        RenderCommand::Mesh(command)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl ModelVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ModelVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 5]>() as u64,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
