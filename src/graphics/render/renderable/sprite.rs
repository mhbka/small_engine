use crate::graphics::{render::assets::SpriteTextureId, scene::node::SceneNodeId};

/// An instance of a sprite.
pub struct SpriteInstance {
    node: SceneNodeId,
    texture: SpriteTextureId,
}

/// The data for a quad vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

impl QuadVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
            ],
        }
    }
}

/// A 1x1, origin-centred square with standard interpolated texture.
///
/// Any other rectangular quad can be transformed from this.
const QUAD: [QuadVertex; 4] = [
    QuadVertex {
        position: [-0.5, -0.5, 0.0],
        uv: [0.0, 1.0],
    },
    QuadVertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
    },
    QuadVertex {
        position: [0.5, 0.5, 0.0],
        uv: [1.0, 0.0],
    },
    QuadVertex {
        position: [-0.5, 0.5, 0.0],
        uv: [0.0, 0.0],
    },
];
