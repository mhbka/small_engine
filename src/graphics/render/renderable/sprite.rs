use crate::graphics::render::commands::{DrawCommand, SpriteRenderCommand};
use crate::graphics::render::renderer::{BindGroupId, PipelineId};
use crate::graphics::scene::instance_buffer::InstanceBufferRange;
use crate::graphics::textures::standard::StandardTexture;
use crate::{core::world::WorldEntityId, graphics::render::assets::SpriteId};

/// An instance of a sprite.
pub struct SpriteInstance {
    pub texture: SpriteId,
    pub entity: WorldEntityId
}

/// A sprite.
pub struct Sprite {
    name: String,
    texture: StandardTexture,
    bind_group: BindGroupId
}

impl Sprite {
    /// Create a command for rendering this sprite.
    pub fn to_render_command(
        &self,
        sprite_id: SpriteId,
        pipeline: PipelineId,
        camera_bind_group: BindGroupId,
        instance_buffer_range: InstanceBufferRange,
    ) -> SpriteRenderCommand {
        SpriteRenderCommand {
            name: &self.name,
            sprite: sprite_id,
            pipeline,
            camera_bind_group,
            sprite_bind_group: self.bind_group,
            instance_buffer_range,
            draw: DrawCommand::Indexed { 
                indices: 0..5,
                base_vertex: 0,
                instances: 0..1
            },
        }
    }
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
const QUAD_VERTICES: [QuadVertex; 4] = [
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

/// Indices to form 2 triangles out of `QUAD_VERTICES`.
const QUAD_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];
