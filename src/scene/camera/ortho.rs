use wgpu::SurfaceConfiguration;
use cgmath::{Matrix4, SquareMatrix, ortho};
use crate::{gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer}, scene::camera::{CameraUniform, create_camera_bind_group}};
use crate::scene::camera::OPENGL_TO_WGPU_MATRIX;

/// An orthographic camera, ie one without depth scaling. Used for 2D scenes usually.
pub struct OrthographicCamera {
    data: OrthoCameraData,
    uniform: CameraUniform,
    buffer: GpuBuffer,
}

impl OrthographicCamera {
    /// Create the orthographic camera.
    pub fn new(gpu: &GpuContext, config: &SurfaceConfiguration, zero_is_centre: bool) -> Self {
        let data = {
            if zero_is_centre {
                OrthoCameraData::new(
                    0.0, 
                    config.width as f32, 
                    0.0, 
                    config.height as f32, 
                    -1.0, 
                    1.0
                )
            } else {
                OrthoCameraData::new(
                    -(config.width as f32 / 2.0), 
                    config.width as f32 / 2.0, 
                    -(config.height as f32 / 2.0), 
                    config.height as f32 / 2.0, 
                    -1.0, 
                    1.0
                )
            }
        };
        let mut uniform = CameraUniform::new();
        uniform.update_ortho(&data);
        let buffer =
            GpuBuffer::create_uniform("ortho_camera_buffer", gpu, bytemuck::cast_slice(&[uniform]));
        Self {
            data,
            uniform,
            buffer,
        }
    }

    /// Write the camera's uniform buffer to the GPU.
    pub fn write_uniform_buffer(&self, gpu: &GpuContext) {
        gpu.queue().write_buffer(
            self.buffer.handle(),
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    pub fn buffer(&self) -> &GpuBuffer { &self.buffer }
}

/// Data for the camera.
pub struct OrthoCameraData {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32
}

impl OrthoCameraData {
    pub fn new(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32
    ) -> Self {
        Self {
            left,
            right,
            bottom,
            top,
            near,
            far
        }
    }

    pub fn build_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = self.build_view_matrix();
        let proj = OPENGL_TO_WGPU_MATRIX * ortho(self.left, self.right, self.bottom, self.top, self.near, self.far);
        return proj * view;
    }
}