use wgpu::SurfaceConfiguration;
use cgmath::{Deg, Matrix4, Quaternion, Rad, Rotation3, SquareMatrix, Vector3, ortho};
use crate::graphics::{gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer}, scene::camera::{CameraUniform, create_camera_bind_group}};
use crate::graphics::scene::camera::OPENGL_TO_WGPU_MATRIX;

/// An orthographic camera, ie one without depth scaling.
/// Usually for 2D scenes but also for certain situations in 3D. 
/// 
/// For 2D scenes, you can set the yaw/pitch to 0.
pub struct OrthographicCamera {
    data: OrthoCameraData,
    uniform: CameraUniform,
    buffer: GpuBuffer,
}

impl OrthographicCamera {
    /// Create the orthographic camera.
    pub fn new(
        gpu: &GpuContext, 
        origin_at_top_left: bool,
        invert_y: bool,
        position: Vector3<f32>,
        width: f32,
        height: f32,
        yaw: f32,
        pitch: f32,
        zoom: f32,
        near: f32,
        far: f32
    ) -> Self {
        let data = {
            OrthoCameraData::new(
                origin_at_top_left,
                invert_y,
                position,
                width,
                height,
                yaw,
                pitch,
                zoom,
                near,
                far
            )
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

    /// Get the buffer.
    pub fn buffer(&self) -> &GpuBuffer { &self.buffer }

    /// Get the camera data mutably.
    pub fn data_mut(&mut self) -> &mut OrthoCameraData { &mut self.data }
}

/// Data for the camera.
pub struct OrthoCameraData {
    pub origin_at_top_left: bool,
    pub invert_y: bool,
    pub position: Vector3<f32>,
    pub width: f32,
    pub height: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub near: f32,
    pub far: f32
}

impl OrthoCameraData {
    pub fn new(
        origin_at_top_left: bool,
        invert_y: bool,
        position: Vector3<f32>,
        width: f32,
        height: f32,
        yaw: f32,
        pitch: f32,
        zoom: f32,
        near: f32,
        far: f32
    ) -> Self {
        Self {
            origin_at_top_left,
            invert_y,
            position,
            width,
            height,
            yaw,
            pitch,
            zoom,
            near,
            far
        }
    }

    pub fn build_view_matrix(&self) -> Matrix4<f32> {
        let rotation: Matrix4<f32> = (Quaternion::from_angle_y(Deg(self.yaw)) * Quaternion::from_angle_x(Deg(self.pitch))).into();
        let translation = Matrix4::from_translation(-self.position);
        rotation * translation
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = self.build_view_matrix();

        let (mut left, mut right) = if self.origin_at_top_left {
            (0.0, self.width)
        } else {
            (-self.width / 2.0, self.width / 2.0)
        };
        let (mut bottom, mut top) = if self.origin_at_top_left {
            if self.invert_y { (0.0, self.height) } else { (self.height, 0.0) }
        } else {
            if self.invert_y { (-self.height / 2.0, self.height / 2.0) } else { (self.height / 2.0, -self.height / 2.0) }
        };

        left = left / self.zoom;
        right = right / self.zoom;
        top = top / self.zoom;
        bottom = bottom / self.zoom;

        let proj = OPENGL_TO_WGPU_MATRIX * ortho(left, right, bottom, top, self.near, self.far);
        return proj * view;
    }
}