//! Gameplay rendering view.



use bytemuck;
use wgpu::{
    CommandEncoder, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
};
use wgpu_text::glyph_brush::Section; // Import bytemuck

use crate::models::engine::{InstanceRaw, NUM_COLUMNS};
use crate::models::skin::JudgementLabels;
use crate::models::stats::JudgementColors;
use crate::shared::snapshot::GameplaySnapshot;
use crate::views::components::gameplay::playfield::NoteVisual;
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel,
    NotesRemainingDisplay, NpsDisplay, PlayfieldDisplay, ScoreDisplay, ScrollSpeedDisplay,
    TimeLeftDisplay,
};
use crate::views::context::GameplayRenderContext; // Import

pub struct GameplayView {
    playfield_component: PlayfieldDisplay,
    instance_cache: Vec<InstanceRaw>,
    column_instances_cache: Vec<Vec<InstanceRaw>>,
    mine_instances: Vec<InstanceRaw>,
    hold_body_instances: Vec<InstanceRaw>,
    hold_end_instances: Vec<InstanceRaw>,
    burst_body_instances: Vec<InstanceRaw>,
    burst_end_instances: Vec<InstanceRaw>,
}

impl GameplayView {
    pub fn new(playfield_component: PlayfieldDisplay) -> Self {
        let mut column_instances_cache = Vec::with_capacity(NUM_COLUMNS);
        for _ in 0..NUM_COLUMNS {
            column_instances_cache.push(Vec::with_capacity(100));
        }

        Self {
            playfield_component,
            instance_cache: Vec::with_capacity(2000),
            column_instances_cache,
            mine_instances: Vec::with_capacity(50),
            hold_body_instances: Vec::with_capacity(50),
            hold_end_instances: Vec::with_capacity(50),
            burst_body_instances: Vec::with_capacity(50),
            burst_end_instances: Vec::with_capacity(50),
        }
    }

    pub fn playfield_component(&self) -> &PlayfieldDisplay {
        &self.playfield_component
    }

    pub fn playfield_component_mut(&mut self) -> &mut PlayfieldDisplay {
        &mut self.playfield_component
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        ctx: &mut GameplayRenderContext<'_>,
        encoder: &mut CommandEncoder,
        snapshot: &GameplaySnapshot,
        score_display: &mut ScoreDisplay,
        accuracy_panel: &mut AccuracyDisplay,
        judgements_panel: &mut JudgementPanel,
        combo_display: &mut ComboDisplay,
        judgement_flash: &mut JudgementFlash,
        hit_bar: &mut HitBarDisplay,
        nps_display: &mut NpsDisplay,
        notes_remaining_display: &mut NotesRemainingDisplay,
        scroll_speed_display: &mut ScrollSpeedDisplay,
        time_left_display: &mut TimeLeftDisplay,
        colors: &JudgementColors,
        labels: &JudgementLabels,
    ) -> Result<(), wgpu::SurfaceError> {
        let effective_scroll_speed = snapshot.scroll_speed * snapshot.rate;

        let now = std::time::Instant::now();
        let delta_time_ms = now.duration_since(snapshot.timestamp).as_secs_f64() * 1000.0;
        let clamped_delta = delta_time_ms.min(50.0);
        let interpolated_time = snapshot.audio_time + (clamped_delta * snapshot.rate);

        let typed_instances = self.playfield_component.render_notes_typed(
            &snapshot.visible_notes,
            interpolated_time,
            effective_scroll_speed,
            ctx.pixel_system,
        );

        self.instance_cache.clear();
        for col_vec in &mut self.column_instances_cache {
            col_vec.clear();
        }
        self.mine_instances.clear();
        self.hold_body_instances.clear();
        self.hold_end_instances.clear();
        self.burst_body_instances.clear();
        self.burst_end_instances.clear();

        for note_instance in typed_instances {
            match note_instance.visual {
                NoteVisual::Tap => {
                    if note_instance.column < self.column_instances_cache.len() {
                        self.column_instances_cache[note_instance.column]
                            .push(note_instance.instance);
                    }
                }
                NoteVisual::Mine => self.mine_instances.push(note_instance.instance),
                NoteVisual::HoldBody => self.hold_body_instances.push(note_instance.instance),
                NoteVisual::HoldEnd => self.hold_end_instances.push(note_instance.instance),
                NoteVisual::BurstBody => self.burst_body_instances.push(note_instance.instance),
                NoteVisual::BurstEnd => self.burst_end_instances.push(note_instance.instance),
            }
        }

        let mut column_offsets: Vec<u64> = Vec::with_capacity(NUM_COLUMNS);
        let mut total_instances = 0u64;

        for col_instances in &self.column_instances_cache {
            column_offsets.push(total_instances);
            self.instance_cache.extend(col_instances.iter().copied());
            total_instances += col_instances.len() as u64;
        }

        let mine_offset = total_instances;
        self.instance_cache
            .extend(self.mine_instances.iter().copied());
        total_instances += self.mine_instances.len() as u64;

        let hold_body_offset = total_instances;
        self.instance_cache
            .extend(self.hold_body_instances.iter().copied());
        total_instances += self.hold_body_instances.len() as u64;

        let hold_end_offset = total_instances;
        self.instance_cache
            .extend(self.hold_end_instances.iter().copied());
        total_instances += self.hold_end_instances.len() as u64;

        let burst_body_offset = total_instances;
        self.instance_cache
            .extend(self.burst_body_instances.iter().copied());
        total_instances += self.burst_body_instances.len() as u64;

        let burst_end_offset = total_instances;
        self.instance_cache
            .extend(self.burst_end_instances.iter().copied());

        if !self.instance_cache.is_empty() {
            ctx.queue.write_buffer(
                ctx.instance_buffer,
                0,
                bytemuck::cast_slice(&self.instance_cache),
            );
        }

        let mut text_sections = Vec::new();
        let fps_text = format!("{:.0}", ctx.fps);
        text_sections.push(Section {
            screen_position: (ctx.screen_width - 60.0, 20.0),
            bounds: (ctx.screen_width, ctx.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(&fps_text)
                    .with_scale(24.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        score_display.set_score(snapshot.score);
        text_sections.extend(score_display.render(ctx.screen_width, ctx.screen_height));

        text_sections.extend(accuracy_panel.render(
            snapshot.accuracy,
            ctx.screen_width,
            ctx.screen_height,
        ));

        // PASSAGE DES LABELS AU PANEL (no more notes/speed - they're separate now)
        text_sections.extend(judgements_panel.render(
            &snapshot.hit_stats,
            ctx.screen_width,
            ctx.screen_height,
            labels,
        ));

        text_sections.extend(combo_display.render(
            snapshot.combo,
            ctx.screen_width,
            ctx.screen_height,
        ));

        // PASSAGE DES COULEURS ET LABELS AU FLASH avec timing pour +/-
        text_sections.extend(judgement_flash.render(
            snapshot.last_hit_judgement,
            snapshot.last_hit_timing, // timing in ms for +/- indicator
            ctx.screen_width,
            ctx.screen_height,
            colors,
            labels,
        ));

        text_sections.extend(hit_bar.render(
            snapshot.last_hit_timing.zip(snapshot.last_hit_judgement),
            ctx.screen_width,
            ctx.screen_height,
        ));
        text_sections.extend(nps_display.render(snapshot.nps, ctx.screen_width, ctx.screen_height));

        // NEW: Separate display components
        text_sections.extend(notes_remaining_display.render(
            snapshot.remaining_notes,
            ctx.screen_width,
            ctx.screen_height,
        ));
        text_sections.extend(scroll_speed_display.render(
            snapshot.scroll_speed,
            ctx.screen_width,
            ctx.screen_height,
        ));
        text_sections.extend(time_left_display.render(
            snapshot.audio_time,   // elapsed
            snapshot.map_duration, // total
            ctx.screen_width,
            ctx.screen_height,
        ));

        ctx.text_brush
            .queue(ctx.device, ctx.queue, text_sections)
            .map_err(|_| wgpu::SurfaceError::Lost)?;

        let receptor_instances = self.playfield_component.render_receptors(ctx.pixel_system);
        if !receptor_instances.is_empty() {
            ctx.queue.write_buffer(
                ctx.receptor_buffer,
                0,
                bytemuck::cast_slice(&receptor_instances),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Gameplay Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: ctx.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(ctx.render_pipeline);

            if !receptor_instances.is_empty() {
                for (col, _) in receptor_instances.iter().enumerate() {
                    if col < ctx.receptor_bind_groups.len() {
                        let is_pressed = snapshot.keys_held.get(col).copied().unwrap_or(false);
                        let bind_group =
                            if is_pressed && col < ctx.receptor_pressed_bind_groups.len() {
                                &ctx.receptor_pressed_bind_groups[col]
                            } else {
                                &ctx.receptor_bind_groups[col]
                            };
                        render_pass.set_bind_group(0, bind_group, &[]);
                        let offset = (col * std::mem::size_of::<InstanceRaw>()) as u64;
                        let size = std::mem::size_of::<InstanceRaw>() as u64;
                        render_pass
                            .set_vertex_buffer(0, ctx.receptor_buffer.slice(offset..offset + size));
                        render_pass.draw(0..6, 0..1);
                    }
                }
            }

            let draw_special_instances =
                |render_pass: &mut wgpu::RenderPass,
                 bind_group: Option<&wgpu::BindGroup>,
                 fallback_bind_group: &wgpu::BindGroup,
                 offset: u64,
                 count: usize,
                 buffer: &wgpu::Buffer| {
                    if count == 0 {
                        return;
                    }
                    render_pass.set_bind_group(0, bind_group.unwrap_or(fallback_bind_group), &[]);
                    let offset_bytes = offset * std::mem::size_of::<InstanceRaw>() as u64;
                    let size_bytes = count as u64 * std::mem::size_of::<InstanceRaw>() as u64;
                    render_pass.set_vertex_buffer(
                        0,
                        buffer.slice(offset_bytes..offset_bytes + size_bytes),
                    );
                    render_pass.draw(0..6, 0..count as u32);
                };

            let fallback = &ctx.note_bind_groups[0];

            draw_special_instances(
                &mut render_pass,
                ctx.hold_body_bind_group,
                fallback,
                hold_body_offset,
                self.hold_body_instances.len(),
                ctx.instance_buffer,
            );
            draw_special_instances(
                &mut render_pass,
                ctx.burst_body_bind_group,
                fallback,
                burst_body_offset,
                self.burst_body_instances.len(),
                ctx.instance_buffer,
            );

            for (col, col_instances) in self.column_instances_cache.iter().enumerate() {
                if col_instances.is_empty() || col >= ctx.note_bind_groups.len() {
                    continue;
                }
                render_pass.set_bind_group(0, &ctx.note_bind_groups[col], &[]);
                let offset_bytes = column_offsets[col] * std::mem::size_of::<InstanceRaw>() as u64;
                let size_bytes =
                    col_instances.len() as u64 * std::mem::size_of::<InstanceRaw>() as u64;
                render_pass.set_vertex_buffer(
                    0,
                    ctx.instance_buffer
                        .slice(offset_bytes..offset_bytes + size_bytes),
                );
                render_pass.draw(0..6, 0..col_instances.len() as u32);
            }

            draw_special_instances(
                &mut render_pass,
                ctx.hold_end_bind_group,
                fallback,
                hold_end_offset,
                self.hold_end_instances.len(),
                ctx.instance_buffer,
            );
            draw_special_instances(
                &mut render_pass,
                ctx.burst_end_bind_group,
                fallback,
                burst_end_offset,
                self.burst_end_instances.len(),
                ctx.instance_buffer,
            );

            draw_special_instances(
                &mut render_pass,
                ctx.mine_bind_group,
                fallback,
                mine_offset,
                self.mine_instances.len(),
                ctx.instance_buffer,
            );

            // Render TimeLeft progress (Bar/Circle)
            if let Some(instance) = time_left_display.get_progress_instance(
                snapshot.audio_time,
                snapshot.map_duration,
                ctx.screen_width,
                ctx.screen_height,
            ) {
                // Write to buffer
                ctx.queue
                    .write_buffer(ctx.progress_buffer, 0, bytemuck::bytes_of(&instance));

                // Draw
                render_pass.set_pipeline(ctx.progress_pipeline);
                render_pass.set_vertex_buffer(0, ctx.progress_buffer.slice(..));
                render_pass.draw(0..4, 0..1); // 4 vertices for triangle strip, 1 instance
            }

            ctx.text_brush.draw(&mut render_pass);
        }

        Ok(())
    }
}

