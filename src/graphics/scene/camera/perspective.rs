use crate::graphics::{gpu::{GpuContext, bind_group::GpuBindGroup, buffer::GpuBuffer}, scene::camera::{OPENGL_TO_WGPU_MATRIX, create_camera_bind_group}};
use cgmath::{Deg, Matrix4, Point3, Vector3, perspective};
use wgpu::SurfaceConfiguration;
use crate::graphics::scene::camera::CameraUniform;
use winit::keyboard::KeyCode;

/// A perspective camera, ie one with depth scaling. Used for 3D scenes usually.
pub struct PerspectiveCamera {
    data: PerspectiveCameraData,
    uniform: CameraUniform,
    controller: PerspectiveCameraController,
    buffer: GpuBuffer,
}

impl PerspectiveCamera {
    /// Create a perspective camera.
    pub fn new(gpu: &GpuContext, config: &SurfaceConfiguration) -> Self {
        let data = PerspectiveCameraData::new(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );
        let mut uniform = CameraUniform::new();
        uniform.update_perspective(&data);
        let buffer =
            GpuBuffer::create_uniform("perspective_camera_buffer", gpu, bytemuck::cast_slice(&[uniform]));
        let controller = PerspectiveCameraController::new(0.2);

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
        self.uniform.update_perspective(&self.data);
    }

    /// Write the camera's uniform buffer to the GPU.
    pub fn write_uniform_buffer(&self, gpu: &GpuContext) {
        gpu.queue().write_buffer(
            self.buffer().handle(),
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    /// Update the camera controller state based on the key input.
    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        self.controller.handle_key(code, is_pressed)
    }

    /// Get the buffer.
    pub fn buffer(&self) -> &GpuBuffer {
        &self.buffer
    }

    /// Get the camera data mutably.
    pub fn data_mut(&mut self) -> &mut PerspectiveCameraData { &mut self.data }
}

/// Data for the camera.
pub struct PerspectiveCameraData {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl PerspectiveCameraData {
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

/// The camera controller, used for mapping inputs to camera movement.
pub struct PerspectiveCameraController {
    pub speed: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
}

impl PerspectiveCameraController {
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

    pub fn update_camera(&self, camera: &mut PerspectiveCameraData) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the Perspectivecamera gets too close to the
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
