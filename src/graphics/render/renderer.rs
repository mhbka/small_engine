use crate::{core::world::World, graphics::{
    constants::{
        GLOBAL_BIND_GROUP_SLOT, INDEX_BUFFER_FORMAT, INSTANCE_BUFFER_SLOT,
        LIGHTING_BIND_GROUP_SLOT, OBJECT_BIND_GROUP_SLOT, VERTEX_BUFFER_SLOT,
    },
    gpu::{GpuContext, bind_group::GpuBindGroup, pipeline::GpuPipeline, texture::GpuTexture},
    render::{
        assets::{AssetStore, MeshId},
        commands::{DrawCommand, MeshRenderCommand, RenderCommand},
    },
    scene::{Scene, SceneError, instance_buffer::InstanceBuffer},
}, hdr::HdrPipeline};
use slotmap::{SlotMap, new_key_type};
use thiserror::Error;
use wgpu::{CommandEncoder, RenderPass, SurfaceTexture, TextureView};

new_key_type! {
    /// For referencing pipelines in the renderer.
    pub struct PipelineId;
    /// For referencing global bind groups in the renderer.
    pub struct GlobalBindGroupId;
    /// For referencing local lighting bind groups in the renderer.
    pub struct LightingBindGroupId;
}

/// Data for a currently rendering frame.
struct CurrentFrameData {
    output: SurfaceTexture,
    view: TextureView
}

/// Handles rendering for the entire program.
pub struct Renderer<'a> {
    gpu: GpuContext,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_is_configured: bool,
    depth_texture: GpuTexture,
    instance_buffer: InstanceBuffer,
    assets: AssetStore,
    hdr: HdrPipeline,
    pipelines: SlotMap<PipelineId, GpuPipeline>,
    global_bind_groups: SlotMap<GlobalBindGroupId, GpuBindGroup>,
    lighting_bind_groups: SlotMap<LightingBindGroupId, GpuBindGroup>,
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
        let depth_texture =
            GpuTexture::create_depth_texture(&gpu, "depth_texture", &surface_config);
        let instance_buffer = InstanceBuffer::new(gpu.clone(), "instance_buffer".into());
        let hdr = HdrPipeline::new(&gpu, &surface_config);
        Self {
            gpu,
            surface,
            surface_config,
            surface_is_configured: false,
            depth_texture,
            instance_buffer,
            assets,
            hdr,
            pipelines: SlotMap::with_key(),
            global_bind_groups: SlotMap::with_key(),
            lighting_bind_groups: SlotMap::with_key(),
            current_frame: None
        }
    }

    /// Handle resizing of the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface
                .configure(&self.gpu.device(), &self.surface_config);
            self.surface_is_configured = true;
            self.depth_texture =
                GpuTexture::create_depth_texture(&self.gpu, "depth_texture", &self.surface_config);
            self.hdr.resize(&self.gpu, width, height);
        }
    }

    /// Add the pipelines to the renderer and get back their IDs for referencing.
    pub fn add_pipelines(&mut self, pipelines: Vec<GpuPipeline>) -> Vec<PipelineId> {
        pipelines
            .into_iter()
            .map(|p| self.pipelines.insert(p))
            .collect()
    }

    /// Add the global bind groups to the renderer and get back their IDs for referencing.
    pub fn add_global_bind_groups(&mut self, groups: Vec<GpuBindGroup>) -> Vec<GlobalBindGroupId> {
        groups
            .into_iter()
            .map(|g| self.global_bind_groups.insert(g))
            .collect()
    }

    /// Add the lighting bind groups to the renderer and return their IDs for referencing.
    pub fn add_lighting_bind_groups(
        &mut self,
        groups: Vec<GpuBindGroup>,
    ) -> Vec<LightingBindGroupId> {
        groups
            .into_iter()
            .map(|g| self.lighting_bind_groups.insert(g))
            .collect()
    }

    /// Get the referenced pipeline.
    pub fn get_pipeline(&self, id: PipelineId) -> Option<&GpuPipeline> {
        self.pipelines.get(id)
    }

    /// Get the referenced global bind group.
    pub fn get_global_bind_group(&self, id: GlobalBindGroupId) -> Option<&GpuBindGroup> {
        self.global_bind_groups.get(id)
    }

    /// Get the referenced lighting bind group.
    pub fn get_lighting_bind_group(&self, id: LightingBindGroupId) -> Option<&GpuBindGroup> {
        self.lighting_bind_groups.get(id)
    }

    /// Begin a frame for rendering.
    pub fn begin_frame(&mut self) -> Result<(), RenderError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.current_frame = Some(CurrentFrameData { output, view });
        Ok(())
    }

    /// End a frame for rendering by displaying it.
    pub fn end_frame(&mut self) -> Result<(), RenderError> {
        if let Some(frame) = self.current_frame.take() {
            frame.output.present();
            return Ok(());
        }
        Err(RenderError::NoFrameInProgress)
    }

    /// Render the given scene only for the frame.
    ///
    /// If any command fails, rendering stops there and this returns a `RenderError`.
    pub fn render_scene_for_frame(&mut self, scene: &Scene, world: &World) -> Result<(), RenderError> {
        if !self.surface_is_configured {
            return Err(RenderError::UnconfiguredSurface);
        }

        // get the render commands
        let commands = scene.to_commands(&world, &self.assets, &mut self.instance_buffer)?;
        self.instance_buffer.write();

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
                view: &self.hdr.view(),
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
                view: &self.depth_texture.view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // write the render commands
        for (i, command) in commands.iter().enumerate() {
            match command {
                RenderCommand::Mesh(command) => {
                    self.write_mesh_command(&command, &mut render_pass, i)?
                }
            }
        }

        drop(render_pass);
        self.hdr.process(&mut encoder, &frame.view);
        self.gpu.queue().submit(std::iter::once(encoder.finish()));
        self.instance_buffer.clear();
        Ok(())
    }

    /// Submit some commands to the command encoder.
    pub fn encode_commands<G>(&mut self, mut encode: G) -> Result<(), RenderError> 
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
            .submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    /// Render with a render pass.
    pub fn render_with_render_pass<F>(&mut self, mut render: F, use_depth: bool) -> Result<(), RenderError> 
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
                view: &self.depth_texture.view(),
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
                view: self.hdr.view(),
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
        &self,
        command: &MeshRenderCommand<'_>,
        render_pass: &mut wgpu::RenderPass<'_>,
        index: usize,
    ) -> Result<(), RenderError> {
        let pipeline = self
            .get_pipeline(command.pipeline)
            .ok_or(RenderError::PipelineNotFound { index })?
            .handle();
        render_pass.set_pipeline(pipeline);

        // get and set the bind groups
        let global_bind_group = self
            .get_global_bind_group(command.global_bind_group)
            .ok_or(RenderError::GlobalBindGroupNotFound { index })?
            .handle();
        let lighting_bind_group = self
            .get_lighting_bind_group(command.lighting_bind_group)
            .ok_or(RenderError::LightingBindGroupNotFound { index })?
            .handle();
        render_pass.set_bind_group(GLOBAL_BIND_GROUP_SLOT, global_bind_group, &[]);
        render_pass.set_bind_group(LIGHTING_BIND_GROUP_SLOT, lighting_bind_group, &[]);
        render_pass.set_bind_group(
            OBJECT_BIND_GROUP_SLOT,
            command.object_bind_group.handle(),
            &[],
        );

        // extra bind groups are slots 3 and above
        for (i, group) in command.extra_bind_groups.iter().enumerate() {
            let i = i + 3;
            render_pass.set_bind_group(i as u32, group.handle(), &[]);
        }

        // normal vertex buffer
        render_pass.set_vertex_buffer(VERTEX_BUFFER_SLOT, command.vertex_buffer);

        // instance vertex buffer - write the buffer data, then get our buffer slices
        let instance_buffer_slice = self
            .instance_buffer
            .get_slice(command.mesh)
            .ok_or(RenderError::MeshHasNoInstanceData(command.mesh))?;
        render_pass.set_vertex_buffer(INSTANCE_BUFFER_SLOT, instance_buffer_slice);

        // index buffer
        render_pass.set_index_buffer(command.index_buffer, INDEX_BUFFER_FORMAT);

        // draw
        self.draw(command.draw.clone(), render_pass);

        Ok(())
    }

    /// Handle the draw command.
    fn draw(&self, draw_command: DrawCommand, render_pass: &mut wgpu::RenderPass<'_>) {
        match draw_command {
            DrawCommand::NonIndexed {
                vertices,
                instances,
            } => render_pass.draw(vertices, instances),
            DrawCommand::Indexed {
                indices,
                base_vertex,
                instances,
            } => render_pass.draw_indexed(indices, base_vertex, instances),
        }
    }
}

/// An error from rendering.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("No frame in progress (tried to end frame when there's no current frame)")]
    NoFrameInProgress,
    #[error("Pipeline referenced by command {index} not found")]
    PipelineNotFound { index: usize },
    #[error("Global bind group referenced by command {index} not found")]
    GlobalBindGroupNotFound { index: usize },
    #[error("Global bind group referenced by command {index} not found")]
    LightingBindGroupNotFound { index: usize },
    #[error("The surface is not configured yet")]
    UnconfiguredSurface,
    #[error("The mesh {0:?} didn't have a corresponding instance buffer slice")]
    MeshHasNoInstanceData(MeshId),
    #[error("{0}")]
    Scene(#[from] SceneError),
    #[error("{0}")]
    Surface(#[from] wgpu::SurfaceError),
}
