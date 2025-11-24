use bytemuck::{Pod, Zeroable};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// The raw data for a spatial transform, to be directly used in the shader.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RawSpatialTransform {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl RawSpatialTransform {
    /// Get the vertex buffer description of this transform.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<RawSpatialTransform>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // Note that we start at location 5 to reserve 2-4 for other vertex stuff.
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
