use crate::core::world::WorldEntityId;
use crate::graphics::textures::standard::StandardTexture;
use crate::graphics::{
    gpu::{bind_group::GpuBindGroup, buffer::GpuBuffer, texture::GpuTexture},
    render::{
        assets::{MaterialId, MeshId},
        commands::MeshRenderCommand,
        renderer::{BindGroupId, PipelineId},
    },
};

/// Represents an instance of a mesh.
///
/// The instance points to the actual mesh it is an instance of,
/// the entity containing its spatial data,
/// and the material for it.
#[derive(Clone)]
pub struct MeshInstance {
    pub mesh: MeshId,
    pub entity: WorldEntityId,
}

/// A model, essentially a collection of materials (textures) and meshes (vertices).
pub struct Model {
    pub meshes: Vec<MeshId>,
    pub materials: Vec<MaterialId>,
}

/// A material; the texture(s) for meshes.
pub struct Material {
    pub name: String,
    pub diffuse_texture: StandardTexture,
    pub normal_texture: StandardTexture,
    pub bind_group: BindGroupId,
}

/// A mesh; the actual thing rendered.
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub material: MaterialId,
    pub num_elements: u32,
}

impl Mesh {
    /// Create a command for rendering this mesh.
    pub fn to_render_command<'buf>(
        &'buf self,
        id: MeshId,
        material: &Material,
        pipeline: PipelineId,
        camera_bind_group: BindGroupId,
        lighting_bind_group: BindGroupId,
    ) -> MeshRenderCommand<'buf> {
        MeshRenderCommand {
            name: &self.name,
            mesh: id,
            pipeline,
            camera_bind_group,
            lighting_bind_group,
            material_bind_group: material.bind_group,
            vertex_buffer: self.vertex_buffer.handle().slice(..),
            index_buffer: self.index_buffer.handle().slice(..)
        }
    }
}

/// The data provided for each vertex for a model/mesh.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3]
}

impl ModelVertex {
    /// Get the vertex buffer layout.
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
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
