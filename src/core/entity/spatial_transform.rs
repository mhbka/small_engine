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
    /// Get the identity transform (ie doesn't do anything and is positioned at origin).
    pub fn identity() -> Self {
        Self {
            scale: Vector3::new(1.0, 1.0, 1.0),
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        }
    }

    /// Get the uniform data for this transform.
    pub fn to_raw(&self) -> RawSpatialTransform {
        RawSpatialTransform {
            model: self.model().into(),
            normal: self.normal().into(),
        }
    }

    /// Get the model matrix.
    pub fn model(&self) -> Matrix4<f32> {
        (
            Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
        )
        .into()
    }

    /// Get the normal matrix.
    pub fn normal(&self) -> Matrix3<f32> {
        let m4 = self.model();
        // Extract the upper-left 3x3 matrix (columns) from the 4x4 model matrix
        let m3 = Matrix3::from_cols(
            m4.x.truncate(), // column 0 (x, y, z)
            m4.y.truncate(), // column 1
            m4.z.truncate(), // column 2
        );
        m3.invert().unwrap_or(Matrix3::identity()).transpose()
    }

    /// Get the forward direction of the transform.
    pub fn forward(&self) -> Vector3<f32> {
        self.rotation * Vector3::unit_z()
    }

    /// Get the up direction of the transform.
    pub fn up(&self) -> Vector3<f32> {
        self.rotation * Vector3::unit_y()
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
        let combined_model = self.model() * b.model();
        let m3 = Matrix3::from_cols(
            combined_model.x.truncate(),
            combined_model.y.truncate(),
            combined_model.z.truncate(),
        );
        RawSpatialTransform {
            model: combined_model.into(),
            normal: m3.invert().unwrap_or(Matrix3::identity()).transpose().into(),
        }
    }
}