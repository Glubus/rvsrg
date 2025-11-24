use crate::models::menu::MenuState;
use crate::views::components::song_selection_menu::SongSelectionDisplay;
use std::sync::{Arc, Mutex};
use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline, SurfaceError, TextureView};
use wgpu_text::TextBrush;

pub struct MenuView {
    song_menu: SongSelectionDisplay,
}

impl MenuView {
    pub fn new() -> Self {
        Self {
            song_menu: SongSelectionDisplay::new(1280.0, 720.0),
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        text_brush: &mut TextBrush,
        menu_state: &Arc<Mutex<MenuState>>,
        screen_width: f32,
        screen_height: f32,
        fps: f64,
        view: &TextureView,
        background_pipeline: Option<&RenderPipeline>,
        background_bind_group: Option<&BindGroup>,
        quad_pipeline: &RenderPipeline,
        quad_buffer: &Buffer,
    ) -> Result<(), SurfaceError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        if let (Some(pipeline), Some(bind_group)) = (background_pipeline, background_bind_group) {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        } else {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Menu Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        queue.submit(std::iter::once(encoder.finish()));

        self.song_menu.update_size(screen_width, screen_height);
        self.song_menu.update(menu_state);

        self.song_menu.render(
            device,
            queue,
            text_brush,
            view,
            quad_pipeline,
            quad_buffer,
            fps,
            menu_state,
        )?;

        Ok(())
    }
}
