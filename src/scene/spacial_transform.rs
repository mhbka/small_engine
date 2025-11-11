use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix, Matrix3, Matrix4, Quaternion, SquareMatrix, Vector3, Zero};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// Represents the spacial data for anything.
pub struct SpacialTransform {
    pub scale: Vector3<f32>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl SpacialTransform {
    /// Get the identity transform (ie doesn't do anything).
    pub fn identity() -> Self {
        Self {
            scale: Vector3::new(1.0, 1.0, 1.0),
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::zero(),
        }
    }

    /// Get the uniform data for this transform.
    pub fn to_raw(&self) -> RawSpacialTransform {
        let matrices = self.to_matrices();
        RawSpacialTransform {
            model: matrices.0.into(),
            normal: matrices.1.into(),
        }
    }

    /// Get the model and normal matrices.
    pub fn to_matrices(&self) -> (Matrix4<f32>, Matrix3<f32>) {
        (
            (Matrix4::from_translation(self.position)
                * Matrix4::from(self.rotation)
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z))
            .into(),
            Matrix3::from(self.rotation)
                .invert()
                .unwrap_or(Matrix3::identity())
                .transpose()
                .into(),
        )
    }

    /// Combine this transform with another, outputting the raw transform.
    ///
    /// (Assuming this is the global transform) used to combine with local transform for the instance's overall transform.
    pub fn combine(&self, b: &SpacialTransform) -> RawSpacialTransform {
        let (self_model, self_norm) = self.to_matrices();
        let (b_model, b_norm) = b.to_matrices();
        RawSpacialTransform {
            model: (self_model * b_model).into(),
            normal: (self_norm * b_norm).into(),
        }
    }
}

/// The raw data for a spacial transform, to be directly used in the shader.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RawSpacialTransform {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl RawSpacialTransform {
    /// Get the vertex buffer description of this transform.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<RawSpacialTransform>() as wgpu::BufferAddress,
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
