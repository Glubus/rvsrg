use crate::models::engine::{GameEngine, InstanceRaw, NUM_COLUMNS, PixelSystem};
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, PlayfieldDisplay,
    ScoreDisplay,
};
use bytemuck;
use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline, SurfaceError, TextureView};
use wgpu_text::{TextBrush, glyph_brush::Section};

pub struct GameplayView {
    playfield_component: PlayfieldDisplay,
}

impl GameplayView {
    pub fn new(playfield_component: PlayfieldDisplay) -> Self {
        Self {
            playfield_component,
        }
    }

    pub fn playfield_component(&self) -> &PlayfieldDisplay {
        &self.playfield_component
    }

    pub fn playfield_component_mut(&mut self) -> &mut PlayfieldDisplay {
        &mut self.playfield_component
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        text_brush: &mut TextBrush,
        render_pipeline: &RenderPipeline,
        instance_buffer: &Buffer,
        receptor_buffer: &Buffer,
        note_bind_groups: &[BindGroup],
        receptor_bind_groups: &[BindGroup],
        engine: &mut GameEngine,
        pixel_system: &PixelSystem,
        score_display: &mut ScoreDisplay,
        accuracy_panel: &mut AccuracyDisplay,
        judgements_panel: &mut JudgementPanel,
        combo_display: &mut ComboDisplay,
        judgement_flash: &mut JudgementFlash,
        hit_bar: &mut HitBarDisplay,
        screen_width: f32,
        screen_height: f32,
        fps: f64,
        view: &TextureView,
    ) -> Result<(), SurfaceError> {
        engine.update_active_notes();
        engine.detect_misses();
        engine.start_audio_if_needed();

        let song_time = engine.get_game_time();
        let max_future_time = song_time + engine.scroll_speed_ms;
        let min_past_time = song_time - 200.0;

        while engine.head_index < engine.chart.len() {
            if engine.chart[engine.head_index].timestamp_ms < min_past_time {
                engine.head_index += 1;
                engine.notes_passed += 1;
            } else {
                break;
            }
        }

        let visible_notes: Vec<_> = engine
            .chart
            .iter()
            .skip(engine.head_index)
            .take_while(|note| note.timestamp_ms <= max_future_time)
            .cloned()
            .collect();

        let instances_with_columns = self.playfield_component.render_notes(
            &visible_notes,
            song_time,
            engine.scroll_speed_ms,
            pixel_system,
        );

        let mut instances_by_column: Vec<Vec<InstanceRaw>> = vec![Vec::new(); NUM_COLUMNS];
        for (column, instance) in instances_with_columns {
            if column < instances_by_column.len() {
                instances_by_column[column].push(instance);
            }
        }

        let mut column_offsets: Vec<u64> = Vec::new();
        let mut total_instances = 0u64;
        for col_instances in &instances_by_column {
            column_offsets.push(total_instances);
            total_instances += col_instances.len() as u64;
        }

        let mut all_instances: Vec<InstanceRaw> = Vec::new();
        for col_instances in &instances_by_column {
            all_instances.extend(col_instances.iter().cloned());
        }

        if !all_instances.is_empty() {
            queue.write_buffer(instance_buffer, 0, bytemuck::cast_slice(&all_instances));
        }

        let mut text_sections = Vec::new();
        let fps_text = format!("FPS: {:.0}", fps);
        text_sections.push(Section {
            screen_position: (screen_width - 100.0, 20.0),
            bounds: (screen_width, screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(&fps_text)
                    .with_scale(24.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        score_display.set_score(engine.notes_passed);
        text_sections.extend(score_display.render(screen_width, screen_height));
        text_sections.extend(accuracy_panel.render(
            engine.hit_stats.calculate_accuracy(),
            screen_width,
            screen_height,
        ));
        text_sections.extend(judgements_panel.render(
            &engine.hit_stats,
            engine.get_remaining_notes(),
            engine.scroll_speed_ms,
            screen_width,
            screen_height,
        ));
        text_sections.extend(combo_display.render(engine.combo, screen_width, screen_height));
        text_sections.extend(judgement_flash.render(
            engine.last_hit_judgement,
            screen_width,
            screen_height,
        ));
        text_sections.extend(hit_bar.render(
            engine.last_hit_timing.zip(engine.last_hit_judgement),
            screen_width,
            screen_height,
        ));

        text_brush
            .queue(device, queue, text_sections)
            .map_err(|_| SurfaceError::Lost)?;

        let receptor_instances = self.playfield_component.render_receptors(pixel_system);
        if !receptor_instances.is_empty() {
            queue.write_buffer(
                receptor_buffer,
                0,
                bytemuck::cast_slice(&receptor_instances),
            );
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
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

            if !receptor_instances.is_empty() {
                render_pass.set_pipeline(render_pipeline);
                for (col, _) in receptor_instances.iter().enumerate() {
                    if col < receptor_bind_groups.len() {
                        render_pass.set_bind_group(0, &receptor_bind_groups[col], &[]);
                        let offset = (col * std::mem::size_of::<InstanceRaw>()) as u64;
                        let size = std::mem::size_of::<InstanceRaw>() as u64;
                        render_pass
                            .set_vertex_buffer(0, receptor_buffer.slice(offset..offset + size));
                        render_pass.draw(0..6, 0..1);
                    }
                }
            }

            render_pass.set_pipeline(render_pipeline);
            for (col, col_instances) in instances_by_column.iter().enumerate() {
                if col_instances.is_empty() || col >= note_bind_groups.len() {
                    continue;
                }

                let offset_bytes = column_offsets[col] * std::mem::size_of::<InstanceRaw>() as u64;
                let size_bytes =
                    col_instances.len() as u64 * std::mem::size_of::<InstanceRaw>() as u64;

                render_pass.set_bind_group(0, &note_bind_groups[col], &[]);
                render_pass.set_vertex_buffer(
                    0,
                    instance_buffer.slice(offset_bytes..offset_bytes + size_bytes),
                );
                render_pass.draw(0..6, 0..col_instances.len() as u32);
            }

            text_brush.draw(&mut render_pass);
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
