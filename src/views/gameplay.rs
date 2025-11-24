use crate::models::engine::{GameEngine, InstanceRaw, NUM_COLUMNS};
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, PlayfieldDisplay,
    ScoreDisplay,
};
use crate::views::context::GameplayRenderContext;
use bytemuck;
use wgpu::SurfaceError;
use wgpu_text::glyph_brush::Section;

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
        ctx: &mut GameplayRenderContext<'_>,
        engine: &mut GameEngine,
        score_display: &mut ScoreDisplay,
        accuracy_panel: &mut AccuracyDisplay,
        judgements_panel: &mut JudgementPanel,
        combo_display: &mut ComboDisplay,
        judgement_flash: &mut JudgementFlash,
        hit_bar: &mut HitBarDisplay,
    ) -> Result<(), SurfaceError> {
        // Mises à jour logiques
        engine.update_active_notes();
        engine.detect_misses();
        engine.start_audio_if_needed(ctx.master_volume);
        engine.set_volume(ctx.master_volume);

        // --- CORRECTION AUDIO MASTER ---
        let song_time = engine.get_audio_time();
        
        let rate = engine.rate; 

        // On calcule la vitesse de défilement effective.
        // ScrollSpeed (500ms) * Rate (1.5) = 750ms de distance affichée.
        // Cela compense l'accélération du temps pour garder une vitesse visuelle constante.
        let effective_scroll_speed = engine.scroll_speed_ms * rate;

        let max_future_time = song_time + effective_scroll_speed;
        
        // On recule aussi le temps de disparition pour les notes ratées (proportionnel au rate)
        let min_past_time = song_time - (200.0 * rate);

        // Optimisation du Head Index (inchangée, mais utilise min_past_time corrigé)
        while engine.head_index < engine.chart.len() {
            if engine.chart[engine.head_index].timestamp_ms < min_past_time {
                engine.head_index += 1;
                engine.notes_passed += 1;
            } else {
                break;
            }
        }

        // Récupération des notes visibles
        let visible_notes: Vec<_> = engine
            .chart
            .iter()
            .skip(engine.head_index)
            .take_while(|note| note.timestamp_ms <= max_future_time)
            .cloned()
            .collect();

        // --- RENDU DU PLAYFIELD ---
        // On passe effective_scroll_speed ici !
        let instances_with_columns = self.playfield_component.render_notes(
            &visible_notes,
            song_time,
            effective_scroll_speed, // <--- C'est ici que la magie opère pour le visuel
            ctx.pixel_system,
        );

        // Le reste du code WGPU (Buffers, Textes, RenderPass) est PARFAIT.
        // Je remets juste la suite pour la forme, mais tu n'as rien à changer en bas.

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
            ctx.queue
                .write_buffer(ctx.instance_buffer, 0, bytemuck::cast_slice(&all_instances));
        }

        let mut text_sections = Vec::new();
        let fps_text = format!("FPS: {:.0}", ctx.fps);
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

        score_display.set_score(engine.notes_passed);
        text_sections.extend(score_display.render(ctx.screen_width, ctx.screen_height));
        text_sections.extend(accuracy_panel.render(
            engine.hit_stats.calculate_accuracy(),
            ctx.screen_width,
            ctx.screen_height,
        ));
        
        // Pour le panel de judgement, on affiche la scrollspeed de base (comme si on était en rate 1.0)
        // mais l'engine utilise effective_scroll_speed pour le rendu
        text_sections.extend(judgements_panel.render(
            &engine.hit_stats,
            engine.get_remaining_notes(),
            engine.scroll_speed_ms, // <--- Affiche la valeur de base, pas la valeur effective
            ctx.screen_width,
            ctx.screen_height,
        ));
        
        text_sections.extend(combo_display.render(
            engine.combo,
            ctx.screen_width,
            ctx.screen_height,
        ));
        text_sections.extend(judgement_flash.render(
            engine.last_hit_judgement,
            ctx.screen_width,
            ctx.screen_height,
        ));
        text_sections.extend(hit_bar.render(
            engine.last_hit_timing.zip(engine.last_hit_judgement),
            ctx.screen_width,
            ctx.screen_height,
        ));

        ctx.text_brush
            .queue(ctx.device, ctx.queue, text_sections)
            .map_err(|_| SurfaceError::Lost)?;

        let receptor_instances = self.playfield_component.render_receptors(ctx.pixel_system);
        if !receptor_instances.is_empty() {
            ctx.queue.write_buffer(
                ctx.receptor_buffer,
                0,
                bytemuck::cast_slice(&receptor_instances),
            );
        }

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
                        render_pass.set_bind_group(0, &ctx.receptor_bind_groups[col], &[]);
                        let offset = (col * std::mem::size_of::<InstanceRaw>()) as u64;
                        let size = std::mem::size_of::<InstanceRaw>() as u64;
                        render_pass
                            .set_vertex_buffer(0, ctx.receptor_buffer.slice(offset..offset + size));
                        render_pass.draw(0..6, 0..1);
                    }
                }
            }

            render_pass.set_pipeline(ctx.render_pipeline);
            for (col, col_instances) in instances_by_column.iter().enumerate() {
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

        ctx.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
