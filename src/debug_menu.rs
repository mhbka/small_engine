use egui::{Area, ClippedPrimitive, Grid, Id, RawInput, Ui, UiBuilder, ViewportId};
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use wgpu::{Adapter, CommandEncoder, Instance, PresentMode, RenderPass, Surface, TextureFormat, TextureView, rwh::{DisplayHandle, WindowHandle}};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};
use crate::graphics::{gpu::GpuContext, render::hdr::HdrPipeline};

/// Represents the data of a debug menu.
pub trait DebugMenuData {
    /// Render the data, and possibly mutate the data based on interactions.
    fn ui(&mut self, ui: &mut Ui);
}

/// Represents the debug menu.
pub struct DebugMenu {
    renderer: egui_wgpu::Renderer,
    state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
}

impl DebugMenu {
    /// Instantiate the debug menu.
    pub fn new(
        gpu: &GpuContext, 
        surface: &DisplayHandle, 
        window_size: PhysicalSize<u32>
    ) -> Self {
        let renderer_options = RendererOptions {
            msaa_samples: 1,
            depth_stencil_format: None,
            dithering: true,
            predictable_texture_filtering: false,
        };
        let renderer = egui_wgpu::Renderer::new(
            gpu.device(), 
            HdrPipeline::COLOR_FORMAT, 
            renderer_options
        );
        let state = egui_winit::State::new(
            egui::Context::default(), 
            ViewportId::ROOT, 
            surface, 
            None, 
            None, 
            None
        );
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [window_size.width, window_size.height],
            pixels_per_point: 1.0,
        };
        Self {
            renderer,
            state,
            screen_descriptor
        }
    }

    /// Handles a window input.
    /// 
    /// Returns whether the input was consumed; if it was, don't use it for other things (like the game itself).
    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    /// Handle resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_descriptor.size_in_pixels = [width, height];
    }

    /// Setup for the render, returning the primitives required for rendering.
    pub fn setup_render(
        &mut self, 
        window: &Window, 
        encoder: &mut CommandEncoder, 
        data: &mut impl DebugMenuData,
        gpu: &GpuContext
    ) -> Vec<ClippedPrimitive> {
        let input = self.state.take_egui_input(window);
        let output = self.state.egui_ctx().run(input, |ctx| {
            let ui = egui::Window::new("Debug Menu")
                .current_pos([0.0, 0.0])
                .default_size([100.0, 100.0])
                .show(ctx, |ui| {
                    self.ui(ui, data);
                });
        });
        let primitives = self.state
            .egui_ctx()
            .tessellate(output.shapes, output.pixels_per_point);

        for (id, texture) in output.textures_delta.set {
            self.renderer.update_texture(gpu.device(), gpu.queue(), id, &texture);
        }
        for id in output.textures_delta.free {
            self.renderer.free_texture(&id);
        }
        self.renderer.update_buffers(
            gpu.device(), 
            gpu.queue(), 
            encoder, 
            &primitives, 
            &self.screen_descriptor
        );
        primitives
    }

    /// Render the menu.
    /// 
    /// Make sure you call `setup_render` beforehand.
    pub fn render(&mut self, primitives: &Vec<ClippedPrimitive>, render_pass: RenderPass<'_>) {
        let mut render_pass = render_pass.forget_lifetime();
        self.renderer.render(
            &mut render_pass, 
            &primitives,
            &self.screen_descriptor 
        );
    }

    /// Build the menu UI.
    fn ui(&self, ui: &mut Ui, data: &mut impl DebugMenuData) {
        ui.scope_builder(UiBuilder::new(), |ui| {
            Grid::new("debug_menu_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    data.ui(ui)
                })
        });
    }
}