use crate::core::entity::WorldEntity;
use crate::core::world::World;
use crate::systems::camera::{OPENGL_TO_WGPU_MATRIX, CameraUniform};
use crate::graphics::{
    gpu::{GpuContext, buffer::GpuBuffer},
};
use cgmath::{Deg, EuclideanSpace, Matrix4, Point3, Vector3, perspective};
use wgpu::SurfaceConfiguration;

/// A perspective camera, ie one with depth scaling. Used for 3D scenes usually.
pub struct PerspectiveCamera {
    data: PerspectiveCameraData,
    uniform: CameraUniform,
    buffer: GpuBuffer,
}

impl PerspectiveCamera {
    /// Create a perspective camera.
    pub fn new(
        gpu: &GpuContext, 
        config: &SurfaceConfiguration, 
        camera_entity: &WorldEntity,
        label: &str
    ) -> Self {
        let data = PerspectiveCameraData::new(
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );
        let mut uniform = CameraUniform::new();
        uniform.update_perspective(&data, camera_entity);
        let buffer = GpuBuffer::create_uniform(
            label,
            gpu,
            bytemuck::cast_slice(&[uniform]),
        );
        Self {
            data,
            uniform,
            buffer,
        }
    }

    /// Update and write the camera's uniform buffer to the GPU.
    pub(super) fn update_and_write_uniform_buffer(&mut self, entity: &WorldEntity, gpu: &GpuContext) {
        self.uniform.update_perspective(&self.data, entity);
        gpu.queue().write_buffer(
            self.buffer().handle(),
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    /// Get the buffer.
    pub fn buffer(&self) -> &GpuBuffer {
        &self.buffer
    }

    /// Get the camera data mutably.
    pub fn data_mut(&mut self) -> &mut PerspectiveCameraData {
        &mut self.data
    }
}

/// Data for the camera.
pub struct PerspectiveCameraData {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl PerspectiveCameraData {
    pub fn new(
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub(super) fn build_view_matrix(&self, entity: &WorldEntity) -> Matrix4<f32> {
        let transform = entity.transform();
        let position = Point3::from_vec(transform.position);
        let forward = position + transform.forward();
        let up = transform.up();
        Matrix4::look_at_rh(position, forward, up)
    }

    pub(super) fn build_view_projection_matrix(&self, entity: &WorldEntity) -> Matrix4<f32> {
        let view = self.build_view_matrix(entity);
        let proj =
            OPENGL_TO_WGPU_MATRIX * perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return proj * view;
    }
}