//! Thin layer wrapping egui input collection and draw submission.

use egui_wgpu::{Renderer as EguiRenderer, RendererOptions};
use egui_winit::State as EguiState;
use std::sync::Arc;
use wgpu::{Device, Queue, TextureFormat};
use winit::event::WindowEvent;
use winit::window::Window;

use crate::render::context::RenderContext;

pub struct UiOverlay {
    pub ctx: egui::Context,
    state: EguiState,
    renderer: EguiRenderer,
}

impl UiOverlay {
    pub fn new(window: Arc<Window>, device: &Device, output_format: TextureFormat) -> Self {
        let ctx = egui::Context::default();
        let state = EguiState::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        // Use RendererOptions + depth_stencil_format (instead of the legacy depth_format).
        let renderer = EguiRenderer::new(
            device,
            output_format,
            RendererOptions {
                depth_stencil_format: None,
                // MSAA samples and dithering rely on defaults for now.
                ..Default::default()
            },
        );

        Self {
            ctx,
            state,
            renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.ctx.begin_pass(raw_input);
    }

    pub fn end_frame_and_draw(
        &mut self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let full_output = self.ctx.end_pass();

        self.state
            .handle_platform_output(&ctx.window, full_output.platform_output);

        let tris = self
            .ctx
            .tessellate(full_output.shapes, ctx.window.scale_factor() as f32);

        for (id, image) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&ctx.device, &ctx.queue, *id, image);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [ctx.config.width, ctx.config.height],
            pixels_per_point: ctx.window.scale_factor() as f32,
        };

        self.renderer
            .update_buffers(&ctx.device, &ctx.queue, encoder, &tris, &screen_descriptor);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Egui Main Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                // Add the missing depth_slice field to avoid wgpu warnings.
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Lifetime hack required to bridge egui and WGPU.
        let rpass_static = unsafe {
            std::mem::transmute::<&mut wgpu::RenderPass<'_>, &mut wgpu::RenderPass<'static>>(
                &mut render_pass,
            )
        };

        self.renderer
            .render(rpass_static, &tris, &screen_descriptor);

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
