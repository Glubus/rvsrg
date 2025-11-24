use crate::models::menu::MenuState;
use crate::views::components::map_list::MapListDisplay;
use std::sync::{Arc, Mutex};
use wgpu::{Buffer, Device, Queue, RenderPipeline, SurfaceError, TextureView};
use wgpu_text::TextBrush;
use bytemuck::cast_slice;

pub struct SongSelectionDisplay {
    map_list: MapListDisplay,
    screen_width: f32,
    screen_height: f32,
}

impl SongSelectionDisplay {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            map_list: MapListDisplay::new(screen_width, screen_height),
            screen_width,
            screen_height,
        }
    }

    pub fn update_size(&mut self, screen_width: f32, screen_height: f32) {
        self.screen_width = screen_width;
        self.screen_height = screen_height;
        self.map_list.update_size(screen_width, screen_height);
    }

    pub fn update(&mut self, menu_state: &Arc<Mutex<MenuState>>) {
        let (visible_items, selected_index, selected_difficulty_index) = {
            let menu_state_guard = menu_state.lock().unwrap();
            let visible_items = menu_state_guard.get_visible_items();
            (
                visible_items
                    .iter()
                    .map(|(bs, bms)| (bs.clone(), bms.clone()))
                    .collect::<Vec<_>>(),
                menu_state_guard.get_relative_selected_index(),
                menu_state_guard.selected_difficulty_index,
            )
        };

        self.map_list
            .update_cards(&visible_items, selected_index, selected_difficulty_index);
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        text_brush: &mut TextBrush,
        view: &TextureView,
        quad_pipeline: &RenderPipeline,
        quad_buffer: &Buffer,
        fps: f64,
        menu_state: &Arc<Mutex<MenuState>>,
    ) -> Result<(), SurfaceError> {
        let quad_instances = self.map_list.create_quads();

        if !quad_instances.is_empty() {
            queue.write_buffer(quad_buffer, 0, cast_slice(&quad_instances));

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Song Selection Menu Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(quad_pipeline);
                render_pass.set_vertex_buffer(0, quad_buffer.slice(..));
                render_pass.draw(0..4, 0..quad_instances.len() as u32);
            }
            queue.submit(std::iter::once(encoder.finish()));
        }

        let map_list_x = self.map_list.x;
        let map_list_width = self.map_list.width;
        let cards_empty = self.map_list.cards.is_empty();

        let mut text_sections = self.map_list.create_text_sections();

        let fps_text = format!("FPS: {:.0}", fps);
        text_sections.push(wgpu_text::glyph_brush::Section {
            screen_position: (self.screen_width - 100.0, 20.0),
            bounds: (self.screen_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(&fps_text)
                    .with_scale(24.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        text_sections.push(wgpu_text::glyph_brush::Section {
            screen_position: (map_list_x + 20.0, 50.0),
            bounds: (map_list_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Map Selection")
                    .with_scale(36.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        let instructions = "Up/Down: Mapset | Left/Right: Difficulty | Enter: Play | F8: Rescan | ESC: Quit | PageUp/Down: Rate";
        text_sections.push(wgpu_text::glyph_brush::Section {
            screen_position: (20.0, self.screen_height - 50.0),
            bounds: (self.screen_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(instructions)
                    .with_scale(18.0)
                    .with_color([0.5, 0.5, 0.5, 1.0]),
            ],
            ..Default::default()
        });

        let rate_text = {
            if let Ok(menu_state) = menu_state.lock() {
                format!("Rate: {:.1}x", menu_state.rate)
            } else {
                "Rate: 1.0x".to_string()
            }
        };
        text_sections.push(wgpu_text::glyph_brush::Section {
            screen_position: (20.0, self.screen_height - 80.0),
            bounds: (self.screen_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(&rate_text)
                    .with_scale(20.0)
                    .with_color([1.0, 1.0, 0.5, 1.0]),
            ],
            ..Default::default()
        });

        if cards_empty {
            text_sections.push(wgpu_text::glyph_brush::Section {
                screen_position: (map_list_x + 20.0, self.screen_height / 2.0),
                bounds: (self.screen_width, self.screen_height),
                text: vec![
                    wgpu_text::glyph_brush::Text::new("No map found")
                        .with_scale(36.0)
                        .with_color([1.0, 0.5, 0.5, 1.0]),
                ],
                ..Default::default()
            });

            text_sections.push(wgpu_text::glyph_brush::Section {
                screen_position: (map_list_x + 20.0, self.screen_height / 2.0 + 50.0),
                bounds: (self.screen_width, self.screen_height),
                text: vec![
                    wgpu_text::glyph_brush::Text::new("Press F8 to scan maps")
                        .with_scale(24.0)
                        .with_color([0.7, 0.7, 0.7, 1.0]),
                ],
                ..Default::default()
            });
        }

        text_brush
            .queue(device, queue, text_sections)
            .map_err(|_| SurfaceError::Lost)?;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Song Selection Menu Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            text_brush.draw(&mut render_pass);
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}

