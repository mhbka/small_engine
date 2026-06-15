pub mod resources;use std::cmp::{max, min};

use crate::{core::world::World, graphics::{
    constants::{
        INDEX_BUFFER_FORMAT, INSTANCE_BUFFER_SLOT, MESH_CAMERA_BIND_GROUP_SLOT, MESH_LIGHTING_BIND_GROUP_SLOT, MESH_MATERIAL_BIND_GROUP_SLOT, SKYBOX_CAMERA_BIND_GROUP_SLOT, SKYBOX_CUBEMAP_BIND_GROUP_SLOT, SPRITE_CAMERA_BIND_GROUP_SLOT, SPRITE_SPRITE_BIND_GROUP_SLOT, VERTEX_BUFFER_SLOT
    },
    gpu::{GpuContext, bind_group::GpuBindGroup, pipeline::GpuPipeline, texture::GpuTexture},
    render::{
        assets::{AssetStore, MeshId, SpriteId}, commands::{MeshRenderCommand, SkyboxRenderCommand, SpriteRenderCommand}, pipelines::hdr::HdrPipeline, renderer::resources::RendererResources
    },
    scene::{Scene, SceneError, instance_buffer::{InstanceBuffer, InstanceData, WrittenInstanceBuffer}}, textures::depth::DepthTexture,
}};
use log::debug;
use slotmap::{SlotMap, new_key_type};
use thiserror::Error;
use wgpu::{BufferSlice, CommandEncoder, CurrentSurfaceTexture, RenderPass, SurfaceTexture, TextureView};

new_key_type! {
    /// For referencing pipelines in the renderer.
    pub struct PipelineId;
    /// For referencing bind groups in the renderer.
    pub struct BindGroupId;
}

/// Data for a currently rendering frame.
struct CurrentFrameData {
    output: SurfaceTexture,
    view: TextureView
}

/// Handles rendering of the engine.
pub struct Renderer<'a> {
    gpu: GpuContext,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_is_configured: bool,
    depth_texture: DepthTexture,
    instance_buffer: InstanceBuffer,
    resources: RendererResources,
    hdr: HdrPipeline,
    current_frame: Option<CurrentFrameData>
}

impl<'a> Renderer<'a> {
    /// Initialize the renderer.
    pub fn new(
        gpu: GpuContext,
        surface: wgpu::Surface<'a>,
        surface_config: wgpu::SurfaceConfiguration,
        assets: AssetStore,
    ) -> Self {
        let depth_texture = DepthTexture::new(&gpu, "depth_texture", &surface_config);
        let instance_buffer = InstanceBuffer::new(gpu.clone(), "instance_buffer".into());
        let hdr = HdrPipeline::new(&gpu, &surface_config);
        let resources = RendererResources::new(
            SlotMap::with_key(),
            SlotMap::with_key(),
            assets
        );
        Self {
            gpu,
            surface,
            surface_config,
            surface_is_configured: false,
            depth_texture,
            instance_buffer,
            hdr,
            resources,
            current_frame: None
        }
    }

    /// Get the renderer resources.
    pub fn resources(&mut self) -> &mut RendererResources {
        &mut self.resources
    }

    /// Handle resizing of the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // NOTE: WebGL has max 2048px, so we cap this here
            // If we don't wanna support WebGL in the future, this can be removed
            self.surface_config.width = min(2048, width);
            self.surface_config.height = min(2048, height);
            self.surface
                .configure(&self.gpu.device(), &self.surface_config);
            self.surface_is_configured = true;
            self.depth_texture = DepthTexture::new(&self.gpu, "depth_texture", &self.surface_config);
            self.hdr.resize(&self.gpu, width, height);
        }
    }

    /// Begin a frame for rendering.
    pub fn begin_frame(&mut self) -> RenderResult<()> {
        match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(output) => {
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.current_frame = Some(CurrentFrameData { output, view });
                Ok(())
            },
            CurrentSurfaceTexture::Suboptimal(_) | CurrentSurfaceTexture::Outdated => {
                debug!("Surface suboptimal or outdated; configure and try again");
                self.surface.configure(self.gpu.device(), &self.surface_config);
                self.begin_frame()
            }
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                debug!("Surface timed out or occluded; skipping frame");
                Ok(())
            },
            CurrentSurfaceTexture::Lost => {
                panic!("The surface was lost; crashing (WIP: can recover from this in the future");
            }
            CurrentSurfaceTexture::Validation => {
                panic!("A validation error was brought up by the surface texture; crashing");
            }
        }
        
    }

    /// End a frame for rendering by displaying it.
    pub fn end_frame(&mut self) -> RenderResult<()> {
        if let Some(frame) = self.current_frame.take() {
            frame.output.present();
            return Ok(());
        }
        Err(RenderError::NoFrameInProgress)
    }

    /// Render the given scene only for the frame.
    ///
    /// If any command fails, rendering stops there and this returns a `RenderError`.
    pub fn render_scene_for_frame(&mut self, scene: &Scene, world: &World) -> RenderResult<()> {
        if !self.surface_is_configured {
            return Err(RenderError::UnconfiguredSurface);
        }

        // get the surface, encoder, render pass
        let frame = match &self.current_frame {
            Some(frame) => frame,
            None => return Err(RenderError::NoFrameInProgress)
        };
        let mut encoder = self.gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.hdr.texture().view(),
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.inner().view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None
        });

        
        let Renderer { instance_buffer, resources, .. } = self;

        // get the render commands
        let commands = scene.to_commands(&world, resources.get_assets_store(), instance_buffer)?;

        // write the instance buffer (we *must* write this first, so that instance buffer ranges in the scene's commands are correct)
        let written_instance_buffer = instance_buffer.write();

        // write the render commands
        if let Some(command) = &commands.skybox { 
            Self::write_skybox_command(resources, &command, &mut render_pass)?
        }
        for command in commands.mesh {
            Self::write_mesh_command(resources, &command, &written_instance_buffer, &mut render_pass)?
        }
        for command in commands.sprite {
            Self::write_sprite_command(resources, &command, &written_instance_buffer, &mut render_pass)?
        }
        drop(render_pass);

        // process the HDR view into the final surface view and submit the queue
        self.hdr.process(&mut encoder, &frame.view);
        self.gpu.queue().submit([encoder.finish()]);

        Ok(())
    }

    /// Submit some commands to the command encoder.
    pub fn encode_commands<G>(&mut self, mut encode: G) -> RenderResult<()> 
    where 
        G: FnMut(&mut CommandEncoder)
    {
        if !self.surface_is_configured {
            return Err(RenderError::UnconfiguredSurface);
        }

        let frame = match &self.current_frame {
            Some(frame) => frame,
            None => return Err(RenderError::NoFrameInProgress)
        };

        let mut encoder = self.gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        encode(&mut encoder);

        self.hdr.process(&mut encoder, &frame.view);
        self.gpu
            .queue()
            .submit([encoder.finish()]);
        Ok(())
    }

    /// Render with a render pass.
    pub fn render_with_render_pass<F>(&mut self, mut render: F, use_depth: bool) -> RenderResult<()> 
    where 
        F: FnMut(RenderPass<'_>)
    {
        if !self.surface_is_configured {
            return Err(RenderError::UnconfiguredSurface);
        }

        // get the surface, encoder, render pass
        let frame = match &self.current_frame {
            Some(frame) => frame,
            None => return Err(RenderError::NoFrameInProgress)
        };
        let mut encoder =
            self.gpu
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });

        let depth_stencil_attachment = if use_depth {
            Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.inner().view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            })
        } else {
            None
        };
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.hdr.texture().view(),
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None
        });
        
        render(render_pass);

        self.hdr.process(&mut encoder, &frame.view);
        self.gpu.queue().submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    /// Write the mesh command.
    ///
    /// Additionally requires the mesh ID + the instance buffer that the mesh's instance data is in.
    fn write_mesh_command(
        resources: &RendererResources,
        command: &MeshRenderCommand,
        written_instance_buffer: &WrittenInstanceBuffer<'_>,
        render_pass: &mut wgpu::RenderPass,
    ) -> RenderResult<()>
    {
        let pipeline = resources
            .get_pipeline(command.pipeline, command.name)?
            .handle();
        render_pass.set_pipeline(pipeline);

        // bind groups
        let camera_bind_group = resources
            .get_bind_group(command.camera_bind_group, command.name)?
            .handle();
        let lighting_bind_group = resources
            .get_bind_group(command.lighting_bind_group, command.name)?
            .handle();
        let material_bind_group = resources
            .get_bind_group(command.material_bind_group, command.name)?
            .handle();
        render_pass.set_bind_group(MESH_CAMERA_BIND_GROUP_SLOT, camera_bind_group, &[]);
        render_pass.set_bind_group(MESH_LIGHTING_BIND_GROUP_SLOT, lighting_bind_group, &[]);
        render_pass.set_bind_group(MESH_MATERIAL_BIND_GROUP_SLOT, material_bind_group, &[]);

        // normal vertex buffer
        render_pass.set_vertex_buffer(VERTEX_BUFFER_SLOT, command.vertex_buffer);

        // instance vertex buffer
        let instance_buffer_slice = written_instance_buffer
            .get_mesh_slice(command.mesh)
            .ok_or(RenderError::MeshHasNoInstanceData(command.mesh))?;
        render_pass.set_vertex_buffer(INSTANCE_BUFFER_SLOT, instance_buffer_slice);

        // index buffer
        render_pass.set_index_buffer(command.index_buffer, INDEX_BUFFER_FORMAT);

        // draw
        let indices = 0..(command.index_buffer.size().get() as u32 / size_of::<u32>() as u32);
        let instances = 0..(instance_buffer_slice.size().get() as u32 / size_of::<InstanceData>() as u32);
        render_pass.draw_indexed(indices, 0, instances);

        Ok(())
    }

    /// Write a sprite render command to the render pass.
    fn write_sprite_command(
        resources: &RendererResources,
        command: &SpriteRenderCommand,
        written_instance_buffer: &WrittenInstanceBuffer<'_>,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> RenderResult<()> {
        let pipeline = resources
            .get_pipeline(command.pipeline, command.name)?
            .handle();
        render_pass.set_pipeline(pipeline);

        let camera_bind_group = resources
            .get_bind_group(command.camera_bind_group, command.name)?
            .handle();
        let sprite_bind_group = resources
            .get_bind_group(command.sprite_bind_group, command.name)?
            .handle();
        render_pass.set_bind_group(SPRITE_CAMERA_BIND_GROUP_SLOT, camera_bind_group, &[]);
        render_pass.set_bind_group(SPRITE_SPRITE_BIND_GROUP_SLOT, sprite_bind_group, &[]);

        let instance_buffer_slice = written_instance_buffer
            .get_sprite_slice(command.sprite)
            .ok_or(RenderError::SpriteHasNoInstanceData(command.sprite))?;
        render_pass.set_vertex_buffer(INSTANCE_BUFFER_SLOT, instance_buffer_slice);

        let instances = 0..(instance_buffer_slice.size().get() as u32);
        let vertices = 0..5;
        render_pass.draw(vertices, instances);

        Ok(())
    }

    /// Write a skybox render command to the render pass.
    fn write_skybox_command(
        resources: &RendererResources, 
        command: &SkyboxRenderCommand,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> RenderResult<()> 
    {
        let pipeline = resources
            .get_pipeline(command.sky_pipeline, command.name)?
            .handle();
        render_pass.set_pipeline(pipeline);

        let camera_bind_group = resources
            .get_bind_group(command.camera_bind_group, command.name)?
            .handle();
        let sky_bind_group = resources
            .get_bind_group(command.sky_bind_group, command.name)?
            .handle();
        render_pass.set_bind_group(SKYBOX_CAMERA_BIND_GROUP_SLOT, camera_bind_group, &[]);
        render_pass.set_bind_group(SKYBOX_CUBEMAP_BIND_GROUP_SLOT, sky_bind_group, &[]);

        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

/// An error from rendering.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("No frame in progress (tried to end frame when there's no current frame)")]
    NoFrameInProgress,
    #[error("Pipeline referenced by command {label} not found")]
    PipelineNotFound { label: String },
    #[error("Global bind group referenced by command {label} not found")]
    GlobalBindGroupNotFound { label: String },
    #[error("Global bind group referenced by command with label {label} not found")]
    LightingBindGroupNotFound { label: String },
    #[error("The surface is not configured yet")]
    UnconfiguredSurface,
    #[error("The mesh {0:?} didn't have a corresponding instance buffer slice")]
    MeshHasNoInstanceData(MeshId),
    #[error("The sprite {0:?} didn't have a corresponding instance buffer slice")]
    SpriteHasNoInstanceData(SpriteId),
    #[error("{0}")]
    Scene(#[from] SceneError),
    #[error("An error came from the surface texture")]
    Surface
}

/// A result from the renderer.
pub type RenderResult<T> = Result<T, RenderError>;
