use crate::renderer::Renderer;
use crate::views::context::GameplayRenderContext;
use wgpu::{CommandBuffer, CommandEncoderDescriptor, TextureView};

impl Renderer {
    /// Rendu de la couche de jeu (Gameplay, Résultats ou Fond du menu)
    pub fn render_game_layer(
        &mut self,
        view: &TextureView,
    ) -> Result<CommandBuffer, wgpu::SurfaceError> {
        let (in_menu, show_result) = if let Ok(menu_state) = self.menu_state.lock() {
            (menu_state.in_menu, menu_state.show_result)
        } else {
            (false, false)
        };

        // Modification: Si on est en ResultScreen, on veut aussi dessiner le background du menu
        // Donc on traite le cas "show_result" comme le cas "in_menu" pour le background.
        // L'UI Egui viendra se dessiner par-dessus avec un fond semi-transparent.

        if in_menu || show_result {
            // --- MENU BACKGROUND (Draw this for both Menu and Result) ---
            self.update_menu_background();

            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Menu/Result Background Encoder"),
                });

            {
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

                if let (Some(pipeline), Some(bind_group)) = (
                    self.background_pipeline.as_ref(),
                    self.background_bind_group.as_ref(),
                ) {
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(0, bind_group, &[]);
                    render_pass.draw(0..6, 0..1);
                }
            }
            return Ok(encoder.finish());
        }

        // --- GAMEPLAY (Only if !in_menu and !show_result) ---
        let mut ctx = GameplayRenderContext {
            device: &self.device,
            queue: &self.queue,
            text_brush: &mut self.text_brush,
            render_pipeline: &self.render_pipeline,
            instance_buffer: &self.instance_buffer,
            receptor_buffer: &self.receptor_buffer,
            note_bind_groups: &self.note_bind_groups,
            receptor_bind_groups: &self.receptor_bind_groups,
            receptor_pressed_bind_groups: &self.receptor_pressed_bind_groups, // PASSÉ AU CONTEXTE
            view,
            pixel_system: &self.pixel_system,
            screen_width: self.config.width as f32,
            screen_height: self.config.height as f32,
            fps: self.fps,
            master_volume: self.settings.master_volume,
        };

        self.gameplay_view.render(
            &mut ctx,
            &mut self.engine,
            &mut self.score_display,
            &mut self.accuracy_panel,
            &mut self.judgements_panel,
            &mut self.combo_display,
            &mut self.judgement_flash,
            &mut self.hit_bar,
        )
    }
}