use cgmath::Rotation3;
use std::sync::Arc;
use web_time::Instant;
use wgpu::Backends;
use wgpu::{
    BindGroupLayoutDescriptor, CompareFunction, DepthBiasState, DepthStencilState,
    DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, RequestAdapterOptions, StencilState, SurfaceConfiguration, SurfaceError,
    TextureUsages, Trace,
};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::core::world::World;
use crate::example::generated_spaced_entities;
use crate::graphics::gpu::GpuContext;
use crate::graphics::gpu::pipeline::GpuPipeline;
use crate::graphics::gpu::texture::GpuTexture;
use crate::graphics::render::assets::AssetStore;
use crate::graphics::render::renderable::model::MeshInstance;
use crate::graphics::render::renderable::model::ModelVertex;
use crate::graphics::render::renderer::Renderer;
use crate::graphics::scene::Scene;
use crate::graphics::scene::camera::perspective::PerspectiveCamera;
use crate::graphics::scene::camera::{Camera, create_camera_bind_group};
use crate::graphics::scene::instance_buffer::MeshInstanceData;
use crate::graphics::scene::lighting::{Lighting, create_lighting_bind_group};
use crate::resources;

// This will store the state of our game
pub struct State<'a> {
    pub window: Arc<Window>,
    gpu: GpuContext,
    world: World, 
    renderer: Renderer<'a>,
    scene: Scene,
    last_frame_update: Instant,
}

impl<'a> State<'a> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<State<'a>> {
        let size = window.inner_size();

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
                required_limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await?;

        // texture stuff
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
                entries: &GpuTexture::DIFFUSE_BIND_GROUP_LAYOUT_ENTRIES,
                label: Some("texture_bind_group_layout"),
            });

        let gpu = GpuContext::new(device, queue);
        let device = gpu.device();

        // camera
        let perspective_camera = PerspectiveCamera::new(&gpu, &config);
        let camera = Camera::Perspective(perspective_camera);
        let camera_bind_group = create_camera_bind_group(&gpu, camera.buffer());

        // shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // lighting
        let lighting = Lighting::create(&gpu, "lighting", [2.0, 10.0, 2.0], [1.0, 0.0, 0.0]);
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
            &[ModelVertex::desc(), MeshInstanceData::desc()],
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

        // asset store
        let mut assets = AssetStore::new();

        // object
        let obj_model = resources::load_model("cube.obj", &gpu, &mut assets)
            .await
            .unwrap();
        
        // world
        let mut world = World::new();

        // scene nodes
        let entities = generated_spaced_entities(&mut world);

        // renderer
        let mut renderer = Renderer::new(gpu.clone(), surface, config, assets);
        let pipeline_id = renderer.add_pipelines(vec![pipeline])[0];
        let camera_bind_group_id = renderer.add_global_bind_groups(vec![camera_bind_group])[0];
        let lighting_bind_group_id =
            renderer.add_lighting_bind_groups(vec![lighting_bind_group])[0];

        // scene
        let mut scene = Scene::new(
            camera,
            vec![lighting],
            pipeline_id,
            camera_bind_group_id,
            lighting_bind_group_id,
        );

        // scene nodes + mesh instances
        let mesh_instances = obj_model
            .meshes
            .iter()
            .map(|&mesh| {
                let instances = entities
                    .iter()
                    .map(|&entity| MeshInstance { mesh, entity })
                    .collect::<Vec<_>>();
                let instance_ids = scene.add_mesh_instances(mesh, instances);
                (mesh, instance_ids)
            })
            .collect::<Vec<_>>();

        Ok(Self {
            window,
            gpu,
            renderer,
            scene,
            world,
            last_frame_update: Instant::now(),
        })
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now - self.last_frame_update;
        self.last_frame_update = now;

        self.scene.camera().update();

        for light in self.scene.lights() {
            light.update(|uniform| {
                let old_position: cgmath::Vector3<_> = uniform.position.into();
                uniform.position =
                    (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                        * old_position)
                        .into();
            });
        }

        self.scene.write_buffers(&self.gpu);
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
