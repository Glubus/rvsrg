use crate::models::engine::{InstanceRaw, NUM_COLUMNS};
use crate::shared::snapshot::GameplaySnapshot; // NOUVEAU
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, PlayfieldDisplay,
    ScoreDisplay,
};
use crate::views::context::GameplayRenderContext;
use bytemuck;
use wgpu::{CommandBuffer, SurfaceError};
use wgpu_text::glyph_brush::Section;

pub struct GameplayView {
    playfield_component: PlayfieldDisplay,
    instance_cache: Vec<InstanceRaw>,
    column_instances_cache: Vec<Vec<InstanceRaw>>,
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
        ctx: &mut GameplayRenderContext<'_>,
        snapshot: &GameplaySnapshot, // CHANGEMENT : On reçoit un snapshot, plus le moteur
        score_display: &mut ScoreDisplay,
        accuracy_panel: &mut AccuracyDisplay,
        judgements_panel: &mut JudgementPanel,
        combo_display: &mut ComboDisplay,
        judgement_flash: &mut JudgementFlash,
        hit_bar: &mut HitBarDisplay,
    ) -> Result<CommandBuffer, SurfaceError> {
        
        // --- LOGIQUE SUPPRIMÉE ICI (Déplacée dans Engine::update) ---
        // Le renderer est maintenant "stupide" et ne fait qu'afficher ce qu'on lui donne

        // On calcule la vitesse de défilement effective pour l'affichage
        let effective_scroll_speed = snapshot.scroll_speed * snapshot.rate;

        // --- RENDU DU PLAYFIELD ---
        // On utilise les notes visibles fournies par le snapshot
        let instances_with_columns = self.playfield_component.render_notes(
            &snapshot.visible_notes,
            snapshot.audio_time,
            effective_scroll_speed,
            ctx.pixel_system,
        );

        // Nettoyage des caches (sans désallouer la mémoire interne)
        self.instance_cache.clear();
        for col_vec in &mut self.column_instances_cache {
            col_vec.clear();
        }

        // Tri dans les caches
        for (column, instance) in instances_with_columns {
            if column < self.column_instances_cache.len() {
                self.column_instances_cache[column].push(instance);
            }
        }

        // Construction du buffer d'instances global avec offsets
        let mut column_offsets: Vec<u64> = Vec::with_capacity(NUM_COLUMNS);
        let mut total_instances = 0u64;

        for col_instances in &self.column_instances_cache {
            column_offsets.push(total_instances);
            self.instance_cache.extend(col_instances.iter().cloned());
            total_instances += col_instances.len() as u64;
        }

        if !self.instance_cache.is_empty() {
            ctx.queue.write_buffer(
                ctx.instance_buffer,
                0,
                bytemuck::cast_slice(&self.instance_cache),
            );
        }

        // --- TEXTES ---
        let mut text_sections = Vec::new();
        let fps_text = format!("{:.0}", ctx.fps);
        text_sections.push(Section {
            screen_position: (ctx.screen_width - 100.0, 20.0),
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

        text_sections.extend(judgements_panel.render(
            &snapshot.hit_stats,
            snapshot.remaining_notes,
            snapshot.scroll_speed,
            ctx.screen_width,
            ctx.screen_height,
        ));

        text_sections.extend(combo_display.render(
            snapshot.combo,
            ctx.screen_width,
            ctx.screen_height,
        ));
        
        text_sections.extend(judgement_flash.render(
            snapshot.last_hit_judgement,
            ctx.screen_width,
            ctx.screen_height,
        ));
        
        text_sections.extend(hit_bar.render(
            snapshot.last_hit_timing.zip(snapshot.last_hit_judgement),
            ctx.screen_width,
            ctx.screen_height,
        ));

        ctx.text_brush
            .queue(ctx.device, ctx.queue, text_sections)
            .map_err(|_| SurfaceError::Lost)?;

        // --- RECEPTORS ---
        let receptor_instances = self.playfield_component.render_receptors(ctx.pixel_system);
        if !receptor_instances.is_empty() {
            ctx.queue.write_buffer(
                ctx.receptor_buffer,
                0,
                bytemuck::cast_slice(&receptor_instances),
            );
        }

        // --- ENCODING ---
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx.view,
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
                render_pass.set_pipeline(ctx.render_pipeline);
                for (col, _) in receptor_instances.iter().enumerate() {
                    if col < ctx.receptor_bind_groups.len() {
                        // Utilisation du snapshot pour l'état des touches
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

            render_pass.set_pipeline(ctx.render_pipeline);
            for (col, col_instances) in self.column_instances_cache.iter().enumerate() {
                if col_instances.is_empty() || col >= ctx.note_bind_groups.len() {
                    continue;
                }

                let offset_bytes = column_offsets[col] * std::mem::size_of::<InstanceRaw>() as u64;
                let size_bytes =
                    col_instances.len() as u64 * std::mem::size_of::<InstanceRaw>() as u64;

                render_pass.set_bind_group(0, &ctx.note_bind_groups[col], &[]);
                render_pass.set_vertex_buffer(
                    0,
                    ctx.instance_buffer
                        .slice(offset_bytes..offset_bytes + size_bytes),
                );
                render_pass.draw(0..6, 0..col_instances.len() as u32);
            }

            ctx.text_brush.draw(&mut render_pass);
        }

        Ok(encoder.finish())
    }
}