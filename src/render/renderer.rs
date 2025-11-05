use crate::{
    constants::{GLOBAL_BIND_GROUP_SLOT, LIGHTING_BIND_GROUP_SLOT, OBJECT_BIND_GROUP_SLOT},
    gpu::{GpuContext, bind_group::GpuBindGroup, pipeline::GpuPipeline, texture::GpuTexture},
    render::{
        commands::{BasicRenderCommand, DrawCommand, RawRenderCommand, RenderCommand},
        scene::Scene,
    },
};
use slotmap::{SlotMap, new_key_type};
use thiserror::Error;

new_key_type! {
    /// For referencing pipelines in the renderer.
    pub struct PipelineId;
    /// For referencing global bind groups in the renderer.
    pub struct GlobalBindGroupId;
    /// For referencing local lighting bind groups in the renderer.
    pub struct LightingBindGroupId;
}

/// Handles rendering for the entire program.
pub struct Renderer<'a> {
    gpu: GpuContext,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    surface_is_configured: bool,
    depth_texture: GpuTexture,
    pipelines: SlotMap<PipelineId, GpuPipeline>,
    global_bind_groups: SlotMap<GlobalBindGroupId, GpuBindGroup>,
    lighting_bind_groups: SlotMap<LightingBindGroupId, GpuBindGroup>,
}

impl<'a> Renderer<'a> {
    /// Initialize the renderer.
    pub fn new(
        gpu: GpuContext,
        surface: wgpu::Surface<'a>,
        surface_config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let depth_texture =
            GpuTexture::create_depth_texture(&gpu, "depth_texture", &surface_config);
        Self {
            gpu,
            surface,
            surface_config,
            surface_is_configured: false,
            depth_texture,
            pipelines: SlotMap::with_key(),
            global_bind_groups: SlotMap::with_key(),
            lighting_bind_groups: SlotMap::with_key(),
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

    /// Render the given scene only for the frame.
    ///
    /// If any command fails, rendering stops there and this returns a `RenderError`.
    pub fn render_scene_for_frame(&mut self, scene: &Scene) -> Result<(), RenderError> {
        if !self.surface_is_configured {
            return Err(RenderError::UnconfiguredSurface);
        }

        // get the render commands
        let commands = scene.to_commands();

        // get the surface, encoder, render pass
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.gpu
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
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
                RenderCommand::Raw(command) => {
                    self.write_raw_command(command, &mut render_pass, i)?
                }
                RenderCommand::Basic(command) => {
                    self.write_basic_command(command, &mut render_pass, i)?
                }
            }
        }

        // submit the commands and present the output
        drop(render_pass);
        self.gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Write the render calls a raw render command.
    fn write_raw_command(
        &mut self,
        command: &RawRenderCommand<'_>,
        render_pass: &mut wgpu::RenderPass<'_>,
        index: usize,
    ) -> Result<(), RenderError> {
        let pipeline = self
            .pipelines
            .get(command.pipeline)
            .ok_or(RenderError::PipelineNotFound { index })?
            .handle();
        render_pass.set_pipeline(pipeline);

        for group_command in &command.bind_group_commands {
            render_pass.set_bind_group(
                group_command.slot,
                group_command.group.handle(),
                &group_command.offsets,
            );
        }
        for buffer_command in &command.vertex_buffers {
            render_pass.set_vertex_buffer(buffer_command.slot, buffer_command.buffer);
        }
        if let Some((buffer, format)) = command.index_buffer {
            render_pass.set_index_buffer(buffer, format);
        }
        self.draw(command.draw.clone(), render_pass);

        Ok(())
    }

    /// Write the render calls for a basic render command.
    fn write_basic_command(
        &mut self,
        command: &BasicRenderCommand<'_>,
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

        // rest is standard
        for buffer_command in &command.vertex_buffers {
            render_pass.set_vertex_buffer(buffer_command.slot, buffer_command.buffer);
        }
        if let Some((buffer, format)) = command.index_buffer {
            render_pass.set_index_buffer(buffer, format);
        }
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
    #[error("Pipeline referenced by command {index} not found")]
    PipelineNotFound { index: usize },
    #[error("Global bind group referenced by command {index} not found")]
    GlobalBindGroupNotFound { index: usize },
    #[error("Global bind group referenced by command {index} not found")]
    LightingBindGroupNotFound { index: usize },
    #[error("The surface is not configured yet")]
    UnconfiguredSurface,
    #[error("{0}")]
    Surface(#[from] wgpu::SurfaceError),
}
