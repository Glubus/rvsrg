use crate::renderer::Renderer;
use crate::shared::snapshot::RenderState;
use crate::views::context::GameplayRenderContext;
use wgpu::{CommandBuffer, CommandEncoderDescriptor, TextureView};

impl Renderer {
    pub fn render_game_layer(
        &mut self,
        view: &TextureView,
    ) -> Result<CommandBuffer, wgpu::SurfaceError> {
        
        // DÉCISION BASÉE SUR L'ÉTAT COURANT
        match &self.current_state {
            RenderState::Empty => {
                // Rien à afficher (écran noir par défaut via le clear color)
                let encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Empty Encoder"),
                });
                Ok(encoder.finish())
            }

            RenderState::Menu(_) | RenderState::Result(_) => {
                // Mode MENU : On affiche le Background (l'UI Egui viendra par-dessus)
                self.update_menu_background();

                let mut encoder = self
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("Menu Background Encoder"),
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
                Ok(encoder.finish())
            }

            RenderState::InGame(snapshot) => {
                // Mode JEU : On utilise le snapshot pour tout dessiner
                let mut ctx = GameplayRenderContext {
                    device: &self.device,
                    queue: &self.queue,
                    text_brush: &mut self.text_brush,
                    render_pipeline: &self.render_pipeline,
                    instance_buffer: &self.instance_buffer,
                    receptor_buffer: &self.receptor_buffer,
                    note_bind_groups: &self.note_bind_groups,
                    receptor_bind_groups: &self.receptor_bind_groups,
                    receptor_pressed_bind_groups: &self.receptor_pressed_bind_groups,
                    view,
                    pixel_system: &self.pixel_system,
                    screen_width: self.config.width as f32,
                    screen_height: self.config.height as f32,
                    fps: self.fps,
                    master_volume: self.settings.master_volume,
                };

                self.gameplay_view.render(
                    &mut ctx,
                    snapshot, // On passe le snapshot readonly
                    &mut self.score_display,
                    &mut self.accuracy_panel,
                    &mut self.judgements_panel,
                    &mut self.combo_display,
                    &mut self.judgement_flash,
                    &mut self.hit_bar,
                )
            }
        }
    }
}