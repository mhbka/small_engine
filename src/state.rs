use cgmath::{Quaternion, Rotation3, Vector3, Zero};
use egui::ViewportId;
use egui_wgpu::winit::Painter;
use egui_wgpu::{RenderState, RendererOptions, WgpuConfiguration, WgpuSetup, WgpuSetupExisting};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use std::num::NonZero;
use std::sync::Arc;
use web_time::Instant;
use wgpu::{Backends, PresentMode, TextureFormat};
use wgpu::{
    BindGroupLayoutDescriptor, CompareFunction, DepthBiasState, DepthStencilState,
    DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, RequestAdapterOptions, StencilState, SurfaceConfiguration, SurfaceError,
    TextureUsages, Trace,
};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::core::entity::spatial_transform::SpatialTransform;
use crate::core::world::World;
use crate::debug_menu::DebugMenu;
use crate::example::{generate_one_big_entity, generated_spaced_entities};
use crate::graphics::gpu::GpuContext;
use crate::graphics::gpu::bind_group::GpuBindGroup;
use crate::graphics::gpu::pipeline::GpuPipeline;
use crate::graphics::gpu::texture::GpuTexture;
use crate::graphics::render::assets::AssetStore;
use crate::graphics::render::hdr::HdrPipeline;
use crate::graphics::render::renderable::model::MeshInstance;
use crate::graphics::render::renderable::model::ModelVertex;
use crate::graphics::render::renderable::skybox::SkyBox;
use crate::graphics::render::renderer::Renderer;
use crate::graphics::scene::Scene;
use crate::graphics::scene::instance_buffer::MeshInstanceData;
use crate::graphics::scene::light::point::{PointLight, PointLightCollection};
use crate::graphics::textures::depth::DepthTexture;
use crate::graphics::textures::standard::DIFFUSE_BIND_GROUP_LAYOUT_ENTRIES;
use crate::input::state::InputState;
use crate::resources;
use crate::resources::hdr::HdrLoader;
use crate::systems::camera::{Camera, CameraType, create_camera_bind_group};
use crate::systems::camera::perspective::PerspectiveCamera;
use crate::systems::controller::freecam::FreecamController;
use crate::debug_state::DebugState;

// The state of the game.
pub struct State<'a> {
    pub window: Arc<Window>,
    input_state: InputState,
    gpu: GpuContext,
    world: World, 
    renderer: Renderer<'a>,
    scene: Scene,
    last_frame_update: Instant,
    freecam: FreecamController,
    debug_menu: DebugMenu,
    debug_state: DebugState,
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
                // required_features: Features::none(), TODO: changed to below for compute shaders - figure out a feature gate for this
                required_features: Features::all_webgpu_mask(),
                experimental_features: ExperimentalFeatures::disabled(),
                /* TODO: changed to below for compute shaders - figure out a feature gate for this
                required_limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::downlevel_defaults()
                },
                */
                required_limits: Limits::downlevel_defaults(),
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
                entries: &DIFFUSE_BIND_GROUP_LAYOUT_ENTRIES,
                label: Some("texture_bind_group_layout"),
            });

        let gpu = GpuContext::new(device, queue);
        let device = gpu.device();

        // world
        let mut world = World::new();

        // scene nodes
        let entities = generated_spaced_entities(&mut world);
        //let entities = generate_one_big_entity(&mut world);

        // camera
        let cam_entity_id = world.add_entity(
            None, 
            vec![], 
            SpatialTransform::identity()
        );
        let cam_entity = world.entity(cam_entity_id).unwrap();
        let perspective_camera = PerspectiveCamera::new(&gpu, &config, cam_entity, "perspective_camera");
        let cam_type = CameraType::Perspective(perspective_camera);
        let camera = Camera::new(cam_entity_id, cam_type);
        let camera_bind_group = create_camera_bind_group(&gpu, camera.buffer());

        // shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // lighting
        let cam_light = PointLight::new(cam_entity_id, Vector3::new(1.0, 1.0, 1.0));
        let point_light_collection = PointLightCollection::new("point_light_collection", vec![cam_light], &gpu);
        let point_light_bind_group = point_light_collection.create_bind_group("point_light_collection_bind_group", &gpu);

        // render pipeline
        let pipeline = GpuPipeline::create_default(
            "basic_pipeline",
            &gpu,
            &[
                &texture_bind_group_layout,
                &camera_bind_group.layout(),
                &point_light_bind_group.layout(),
            ],
            &[ModelVertex::desc(), MeshInstanceData::desc()],
            &shader,
            &shader,
            Some(DepthStencilState {
                format: DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            wgpu::PrimitiveTopology::TriangleList,
            HdrPipeline::COLOR_FORMAT
        );

        // renderer
        let mut renderer = Renderer::new(gpu.clone(), surface, config, AssetStore::new());
        let pipeline_id = renderer.add_pipelines(vec![pipeline])[0];

        // object
        let obj_model = resources::general::load_model("cube.obj", &gpu, &mut renderer)
            .await
            .unwrap();

        // skybox
        let hdr_loader = HdrLoader::new(&gpu);
        let sky_texture_bytes = resources::general::load_binary("pure-sky.hdr").await?;
        let sky_texture = hdr_loader.from_equirect_bytes(&gpu, &sky_texture_bytes, 1080, "Sky Texture")?;
        let sky_bind_group = GpuBindGroup::create_default(
            "sky_bind_group", 
            &gpu, 
            &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ], 
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(sky_texture.inner().view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sky_texture.inner().sampler()),
                },
            ]
        );
        let shader = device.create_shader_module(wgpu::include_wgsl!("sky.wgsl")); 
        let depth_stencil = wgpu::DepthStencilState {
            format: DepthTexture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        };
        let sky_pipeline = GpuPipeline::create_default(
            "skybox_pipeline",
            &gpu,
            &[camera_bind_group.layout(), sky_bind_group.layout()],
            &[],
            &shader,
            &shader,
            Some(depth_stencil),
            wgpu::PrimitiveTopology::TriangleList,
            HdrPipeline::COLOR_FORMAT,
        );
        let sky_pipeline_id = renderer.add_pipelines(vec![sky_pipeline])[0];
        let skybox = SkyBox::new("skybox".into(), sky_texture);
  

        // scene
        let bind_group_ids = renderer.add_bind_groups(vec![camera_bind_group, point_light_bind_group, sky_bind_group]);
        let camera_bind_group_id = bind_group_ids[0];
        let lighting_bind_group_id = bind_group_ids[1];
        let sky_bind_group_id = bind_group_ids[2]; 
        let mut scene = Scene::new(
            camera,
            point_light_collection,
            pipeline_id,
            camera_bind_group_id,
            lighting_bind_group_id,
            skybox,
            sky_pipeline_id,
            sky_bind_group_id
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

        // input state
        let input_state = InputState::new(true);

        // freecam
        let freecam = FreecamController::new(cam_entity_id);

        // debug menu
        let debug_menu = DebugMenu::new(
            &gpu, 
            &window.display_handle().unwrap(), 
            window.inner_size()
        );
        let debug_state = DebugState::new();

        Ok(Self {
            window,
            input_state,
            gpu,
            renderer,
            scene,
            world,
            last_frame_update: Instant::now(),
            freecam,
            debug_menu,
            debug_state,
        })
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now - self.last_frame_update;
        self.last_frame_update = now;
        self.scene.update_and_write_buffers(&self.world, &self.gpu);
        self.freecam.update(&self.input_state, &mut self.world, delta_time.as_secs_f32()).unwrap();
        
        let cam_pos = self.freecam.pos(&self.world);
        self.debug_state.update(cam_pos);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.debug_menu.resize(width, height);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        self.window.request_redraw();

        self.renderer
            .begin_frame()
            .unwrap();
        
        self.renderer
            .render_scene_for_frame(&self.scene, &self.world)
            .unwrap();

        let mut primitives = vec![];
        self.renderer
            .encode_commands(|encoder| primitives = self.debug_menu.setup_render(&self.window, encoder, &mut self.debug_state, &self.gpu))
            .unwrap();
        self.renderer
            .render_with_render_pass(|pass| self.debug_menu.render(&primitives, pass), false)
            .unwrap();

        self.renderer
            .end_frame()
            .unwrap();

        Ok(())
    }

    pub fn reset_for_frame(&mut self) {
        self.input_state.begin_frame();
    }

    pub fn handle_window_event_for_debug_menu(&mut self, event: &WindowEvent) -> bool {
        self.debug_menu.handle_input(&self.window, event)
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key_code: KeyCode, key_state: ElementState) {
        if (key_code, key_state.is_pressed()) == (KeyCode::Escape, true) {
            event_loop.exit();
        } else {
            self.input_state.process_key_event(key_code, key_state);
        }
    }

    pub fn handle_cursor_delta(&mut self, delta_x: f64, delta_y: f64) {
        self.input_state.process_cursor_delta(delta_x as f32, delta_y as f32);
    }

    pub fn handle_cursor_movement(&mut self, x: f64, y: f64) {
        self.input_state.process_cursor_movement(x as f32, y as f32);
    }

    pub fn handle_mouse_wheel(&mut self, change: MouseScrollDelta) {
        self.input_state.process_mouse_scroll(change)
    }
}
