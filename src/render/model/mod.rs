pub mod instances;

use std::ops::Range;
use wgpu::{BindGroup, Buffer};
use crate::{gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer, texture::GpuTexture}, render::{commands::{BasicRenderCommand, DrawCommand, RenderCommand, VertexBufferCommand}, model::instances::Instances, renderer::{GlobalBindGroupId, LightingBindGroupId, PipelineId}}};

/// Represents something that can be rendered.
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: GpuTexture,
    pub bind_group: GpuBindGroup
}
 
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub instances: Instances,
    pub num_elements: u32,
    pub material: usize,
}

impl Mesh {
    /// Convert the mesh + its material (or any material) to a render command.
    pub fn to_render_command(
        &self, 
        material: &Material,
        pipeline: PipelineId,
        global_bind_group: GlobalBindGroupId,
        lighting_bind_group: LightingBindGroupId
    ) -> RenderCommand {
        let command = BasicRenderCommand {
            pipeline,
            global_bind_group,
            lighting_bind_group,
            object_bind_group: material.bind_group.clone(),
            extra_bind_groups: vec![],
            vertex_buffers: vec![
                VertexBufferCommand {
                    buffer: self.vertex_buffer.handle().slice(..),
                    slot: 0,
                },
                VertexBufferCommand {
                    buffer: self.instances.buffer().handle().slice(..),
                    slot: 1
                }
            ],
            index_buffer: Some((self.index_buffer.handle().slice(..), wgpu::IndexFormat::Uint32)),
            draw: DrawCommand::Indexed { base_vertex: 0, instances: self.instances.range(), indices: 0..self.num_elements },
        };
        RenderCommand::Basic(command)
    }

    /// Update this mesh's instance vertex buffer.
    pub fn update_instance_buffer(&self, gpu: &GpuContext) {
        gpu.queue().write_buffer(self.instances.buffer().handle(), 0, &self.instances.content());
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
