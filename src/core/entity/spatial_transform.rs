use cgmath::{ElementWise, Matrix, Matrix3, Matrix4, Quaternion, SquareMatrix, Vector3, Zero};

use crate::graphics::scene::raw_spatial_transform::RawSpatialTransform;

/// Represents the spacial data for anything.
#[derive(Clone, Copy)]
pub struct SpatialTransform {
    pub scale: Vector3<f32>,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl SpatialTransform {
    /// Get the identity transform (ie doesn't do anything).
    pub fn identity() -> Self {
        Self {
            scale: Vector3::new(1.0, 1.0, 1.0),
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::zero(),
        }
    }

    /// Get the uniform data for this transform.
    pub fn to_raw(&self) -> RawSpatialTransform {
        let matrices = self.to_matrices();
        RawSpatialTransform {
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

    /// Combines this transform with a child transform.
    /// Returns the resulting overall transform of the child.
    pub fn combine(&self, child: &SpatialTransform) -> SpatialTransform {
        let scaled_position = self.scale.mul_element_wise(child.position);
        let combined_scale = self.scale.mul_element_wise(child.scale);

        let rotated_position = self.rotation * scaled_position;
        let final_position = self.position + rotated_position;
        let combined_rotation = self.rotation * child.rotation;

        SpatialTransform {
            scale: combined_scale,
            position: final_position,
            rotation: combined_rotation,
        }
    }

    /// Combine this transform with a child transform.
    /// Returns the resulting raw overall transform of the child.
    pub fn combine_raw(&self, b: &SpatialTransform) -> RawSpatialTransform {
        let (self_model, self_norm) = self.to_matrices();
        let (b_model, b_norm) = b.to_matrices();
        RawSpatialTransform {
            model: (self_model * b_model).into(),
            normal: (self_norm * b_norm).into(),
        }
    }
}