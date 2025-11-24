use bytemuck::NoUninit;
use cgmath::{Matrix4, SquareMatrix};
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages};
use crate::{core::{entity::WorldEntity, world::{World, WorldEntityId}}, systems::camera::{
        ortho::{OrthoCameraData, OrthographicCamera},
        perspective::{PerspectiveCamera, PerspectiveCameraData},
    }};
use crate::graphics::gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer};

pub mod ortho;
pub mod perspective;

/// Converts OpenGL to wgpu matrix conventions.
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

/// A camera for a scene.
/// 
/// Spatial data is denoted by its referenced entity.
pub struct Camera {
    entity: WorldEntityId,
    cam_type: CameraType
}

/// The type of camera.
pub enum CameraType {
    Perspective(PerspectiveCamera),
    Ortho(OrthographicCamera),
}

impl Camera {
    /// Create the camera.
    pub fn new(entity: WorldEntityId, cam_type: CameraType) -> Self {
        Self {
            entity,
            cam_type
        }
    }

    /// Update the camera's data and write it to the uniform data.
    pub fn update_and_write_uniform_buffer(&mut self, world: &World, gpu: &GpuContext) {
        let entity = world
            .entity(self.entity)
            .expect("Camera's entity must exist");
        match &mut self.cam_type {
            CameraType::Perspective(camera) => camera.update_and_write_uniform_buffer(entity, gpu),
            CameraType::Ortho(camera) => camera.update_and_write_uniform_buffer(entity, gpu),
        }
    }

    /// Get the camera's buffer.
    pub fn buffer(&self) -> &GpuBuffer {
        match &self.cam_type {
            CameraType::Perspective(c) => c.buffer(),
            CameraType::Ortho(c) => c.buffer(),
        }
    }
}

/// The camera uniform, ie the actual matrix representing the camera.
#[repr(C)]
#[derive(Debug, Copy, Clone, NoUninit)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
}

impl CameraUniform {
    /// Create a new uniform.
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            view: Matrix4::identity().into(),
        }
    }

    /// Update the uniform for a perspective camera.
    pub fn update_perspective(&mut self, data: &PerspectiveCameraData, entity: &WorldEntity) {
        self.view = data.build_view_matrix(entity).into();
        self.view_proj = data.build_view_projection_matrix(entity).into();
    }

    /// Update the uniform for an ortho camera.
    pub fn update_ortho(&mut self, data: &OrthoCameraData, entity: &WorldEntity) {
        self.view = data.build_view_matrix(entity).into();
        self.view_proj = data.build_view_projection_matrix(entity).into();
    }
}

/// Create the bind group for a camera.
pub fn create_camera_bind_group(gpu: &GpuContext, camera_buffer: &GpuBuffer) -> GpuBindGroup {
    let layout_entries = [BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }];
    let entries = [BindGroupEntry {
        binding: 0,
        resource: camera_buffer.handle().as_entire_binding(),
    }];
    GpuBindGroup::create_default(
        "perspective_camera_bind_group",
        gpu,
        &layout_entries,
        &entries,
    )
}
