use bytemuck::NoUninit;
use cgmath::{Matrix4, SquareMatrix};
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages};

use crate::{gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer}, scene::camera::{ortho::{OrthoCameraData, OrthographicCamera}, perspective::{PerspectiveCamera, PerspectiveCameraData}}};

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
pub enum Camera {
    Perspective(PerspectiveCamera),
    Ortho(OrthographicCamera)
}

impl Camera {
    /// Write the camera's uniform buffer to the GPU.
    pub fn write_uniform_buffer(&self, gpu: &GpuContext) {
        match self {
            Self::Perspective(camera) => camera.write_uniform_buffer(gpu),
            Self::Ortho(camera) => camera.write_uniform_buffer(gpu)
        }
    }

    /// Get the camera's buffer.
    pub fn buffer(&self) -> &GpuBuffer {
        match self {
            Self::Perspective(c) => c.buffer(),
            Self::Ortho(c) => c.buffer()
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
    pub fn update_perspective(&mut self, data: &PerspectiveCameraData) {
        self.view = data.build_view_matrix().into();
        self.view_proj = data.build_view_projection_matrix().into();
    }

    /// Update the uniform for an ortho camera.
    pub fn update_ortho(&mut self, data: &OrthoCameraData) {
        self.view = data.build_view_matrix().into();
        self.view_proj = data.build_view_projection_matrix().into();
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
        &entries
    )
}