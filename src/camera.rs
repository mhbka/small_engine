use crate::gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer};
use bytemuck::NoUninit;
use cgmath::{Deg, Matrix4, Point3, SquareMatrix, Vector3, perspective};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, ShaderStages, SurfaceConfiguration,
};
use winit::keyboard::KeyCode;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

/// Contains all the data and functionality for a camera.
pub struct Camera {
    data: CameraData,
    uniform: CameraUniform,
    controller: CameraController,
    buffer: GpuBuffer,
}

impl Camera {
    /// Create a camera.
    pub fn new(gpu: &GpuContext, config: &SurfaceConfiguration) -> Self {
        let data = CameraData::new(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );
        let mut uniform = CameraUniform::new();
        uniform.update(&data);
        let buffer =
            GpuBuffer::create_uniform("camera_buffer", gpu, bytemuck::cast_slice(&[uniform]));
        let controller = CameraController::new(0.2);

        Self {
            data,
            uniform,
            controller,
            buffer,
        }
    }

    /// Update the camera's values based on its controller's state.
    pub fn update(&mut self) {
        self.controller.update_camera(&mut self.data);
        self.uniform.update(&self.data);
    }

    pub fn update_uniform_buffer(&self, gpu: &GpuContext) {
        gpu.queue().write_buffer(
            self.buffer().handle(),
            0,
            bytemuck::cast_slice(&[*self.uniform()]),
        );
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        self.controller.handle_key(code, is_pressed)
    }

    pub fn data(&self) -> &CameraData {
        &self.data
    }

    pub fn uniform(&self) -> &CameraUniform {
        &self.uniform
    }

    pub fn controller(&self) -> &CameraController {
        &self.controller
    }

    pub fn buffer(&self) -> &GpuBuffer {
        &self.buffer
    }
}

/// Create the bind group for the camera.
pub fn create_camera_bind_group(gpu: &GpuContext, camera_buffer: &GpuBuffer) -> GpuBindGroup {
    let layout = gpu
        .device()
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let group = gpu.device().create_bind_group(&BindGroupDescriptor {
        label: Some("camera_bind_group"),
        layout: &layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: camera_buffer.handle().as_entire_binding(),
        }],
    });

    GpuBindGroup::new(group, layout)
}

/// Data for the camera.
pub struct CameraData {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl CameraData {
    pub fn new(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn build_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = self.build_view_matrix();
        let proj =
            OPENGL_TO_WGPU_MATRIX * perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return proj * view;
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
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            view: Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &CameraData) {
        self.view = camera.build_view_matrix().into();
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

/// The camera controller, used for mapping inputs to camera movement.
pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut CameraData) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
