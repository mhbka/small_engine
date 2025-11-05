use cgmath::Rotation3;
use image::GenericImageView;
use std::sync::Arc;
use web_time::Instant;
use wgpu::Backends;
use wgpu::{
    BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, CompareFunction,
    DepthBiasState, DepthStencilState, DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits, PowerPreference, RequestAdapterOptions, SamplerBindingType,
    ShaderStages, StencilState, SurfaceConfiguration, SurfaceError, TextureSampleType, TextureUsages, TextureViewDimension,
    Trace,
};
use winit::{
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::camera::create_camera_bind_group;
use crate::camera::Camera;
use crate::gpu::GpuContext;
use crate::gpu::pipeline::GpuPipeline;
use crate::gpu::texture::GpuTexture;
use crate::lighting::{Lighting, create_lighting_bind_group};
use crate::render::model::instances::RawInstance;
use crate::render::model::ModelVertex;
use crate::render::renderer::Renderer;
use crate::render::scene::Scene;
use crate::resources;

// This will store the state of our game
pub struct State<'a> {
    pub window: Arc<Window>,
    gpu: GpuContext,
    renderer: Renderer<'a>,
    scene: Scene,
    last_frame_update: Instant,
}

impl<'a> State<'a> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<State<'a>> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = Instance::new(&InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await?;

        // textures
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let gpu = GpuContext::new(device, queue);
        let device = gpu.device();

        // camera
        let camera = Camera::new(&gpu, &config);
        let camera_bind_group = create_camera_bind_group(&gpu, camera.buffer());

        // shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // lighting
        let lighting = Lighting::create(&gpu, "lighting", [2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);
        let lighting_bind_group = create_lighting_bind_group(&gpu, &lighting);

        // render_pipeline
        let pipeline = GpuPipeline::create_default(
            "basic_pipeline",
            &gpu,
            &config,
            &[
                &texture_bind_group_layout,
                &camera_bind_group.layout(),
                &lighting_bind_group.layout(),
            ],
            &[ModelVertex::desc(), RawInstance::desc()],
            &shader,
            &shader,
            Some(DepthStencilState {
                format: GpuTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
        );

        // object
        let obj_model = resources::load_model("cube.obj", &gpu).await.unwrap();

        // renderer
        let mut renderer = Renderer::new(gpu.clone(), surface, config);
        let pipeline_id = renderer.add_pipelines(vec![pipeline])[0];
        let camera_bind_group_id = renderer.add_global_bind_groups(vec![camera_bind_group])[0];
        let lighting_bind_group_id =
            renderer.add_lighting_bind_groups(vec![lighting_bind_group])[0];

        // scene
        let scene = Scene::new(
            vec![obj_model],
            camera,
            vec![lighting],
            pipeline_id,
            camera_bind_group_id,
            lighting_bind_group_id,
        );

        Ok(Self {
            window,
            gpu,
            renderer,
            scene,
            last_frame_update: Instant::now(),
        })
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now - self.last_frame_update;
        self.last_frame_update = now;

        self.scene.camera().update();

        for model in self.scene.models() {
            for mesh in &mut model.meshes {
                for instance in mesh.instances.actual() {
                    instance.update(|instance| {
                        // what to do?
                    });
                }
            }
        }

        for light in self.scene.lights() {
            light.update(|uniform| {
                let old_position: cgmath::Vector3<_> = uniform.position.into();
                uniform.position =
                    (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                        * old_position)
                        .into();
            });
        }

        self.scene.update_buffers(&self.gpu);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        self.window.request_redraw();
        self.renderer.render_scene_for_frame(&self.scene).unwrap();
        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if (code, is_pressed) == (KeyCode::Escape, true) {
            event_loop.exit();
        } else {
            self.scene.camera().handle_key(code, is_pressed);
        }
    }
}
