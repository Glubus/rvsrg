use super::Renderer;
use crate::views::context::{GameplayRenderContext, ResultRenderContext};
use std::{collections::BTreeMap, sync::Arc, time::Instant};
use wgpu::CommandEncoderDescriptor;

impl Renderer {
    // Note: On prend maintenant &Window en paramètre pour egui
    pub fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // --- 1. EGUI UPDATE ---
        // On récupère les inputs et on construit l'UI
        let raw_input = self.egui_state.take_egui_input(window);
        
        // Extraction des données nécessaires pour l'UI avant la closure
        let mut settings_is_open = self.settings.is_open;
        let mut settings_show_keybindings = self.settings.show_keybindings;
        let mut master_volume = self.settings.master_volume;
        let mut hit_window_mode = self.settings.hit_window_mode;
        let mut hit_window_value = self.settings.hit_window_value;
        let keybinding_rows = {
            let mut grouped: BTreeMap<usize, Vec<String>> = BTreeMap::new();
            for (key, column) in &self.skin.key_to_column {
                grouped.entry(*column).or_default().push(key.clone());
            }
            grouped
                .into_iter()
                .map(|(column, mut keys)| {
                    keys.sort();
                    (column, keys)
                })
                .collect::<Vec<_>>()
        };
        
        // Initialize song select screen if needed
        let (in_menu, show_result) = if let Ok(menu_state) = self.menu_state.lock() {
            (menu_state.in_menu, menu_state.show_result)
        } else {
            (false, false)
        };

        if in_menu && self.song_select_screen.is_none() {
            self.song_select_screen = Some(crate::views::components::menu::song_select::SongSelectScreen::new(
                Arc::clone(&self.menu_state)
            ));
        }

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Construction de l'UI directement dans la closure pour éviter les problèmes de borrow
            
            // Render song select if in menu and settings not open
            if in_menu && !settings_is_open {
                if let Some(ref mut song_select) = self.song_select_screen {
                    song_select.render(
                        ctx,
                        &view,
                        self.config.width as f32,
                        self.config.height as f32,
                    );
                }
                return;
            }

            if !settings_is_open {
                return;
            }

            // 1. Panneau Latéral Gauche
            egui::SidePanel::left("settings_panel")
                .resizable(false)
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("Settings");
                    ui.separator();

                    ui.label("Audio");
                    // Slider Volume
                    if ui.add(egui::Slider::new(&mut master_volume, 0.0..=1.0).text("Volume")).changed() {
                        // Appliquer le volume immédiatement
                        self.engine.set_volume(master_volume);
                    }

                    ui.separator();
                    ui.label("Gameplay");
                    
                    // Rate control avec boutons
                    ui.horizontal(|ui| {
                        ui.label("Rate:");
                        let current_rate = if let Ok(menu_state) = self.menu_state.lock() {
                            menu_state.rate
                        } else {
                            1.0
                        };
                        ui.label(format!("{:.1}x", current_rate));
                        
                        if ui.button("−").clicked() {
                            if let Ok(mut menu_state) = self.menu_state.lock() {
                                menu_state.decrease_rate();
                            }
                        }
                        if ui.button("+").clicked() {
                            if let Ok(mut menu_state) = self.menu_state.lock() {
                                menu_state.increase_rate();
                            }
                        }
                    });

                    ui.add_space(10.0);
                    ui.label("Hit Window");
                    
                    // Sélection du mode (OD ou Judge)
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut hit_window_mode, crate::models::settings::HitWindowMode::OsuOD, "OD");
                        ui.radio_value(&mut hit_window_mode, crate::models::settings::HitWindowMode::EtternaJudge, "Judge");
                    });

                    // Slider pour la valeur
                    let (min_val, max_val, label) = match hit_window_mode {
                        crate::models::settings::HitWindowMode::OsuOD => (0.0, 10.0, "OD"),
                        crate::models::settings::HitWindowMode::EtternaJudge => (1.0, 9.0, "Judge Level"),
                    };
                    
                    if ui.add(egui::Slider::new(&mut hit_window_value, min_val..=max_val).text(label)).changed() {
                        // Appliquer immédiatement la hit window
                        self.engine.update_hit_window(hit_window_mode, hit_window_value);
                    }

                    ui.separator();
                    ui.label("Controls");
                    
                    // Bouton pour ouvrir le remapping
                    if ui.button("Remap Keys").clicked() {
                        settings_show_keybindings = true;
                    }

                    ui.add_space(20.0);
                    if ui.button("Close (Ctrl+O)").clicked() {
                        settings_is_open = false;
                    }
                });

            // 2. Fenêtre Centrale (Modal) pour le Keybinding
            if settings_show_keybindings {
                egui::Window::new("Key Bindings")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        if keybinding_rows.is_empty() {
                            ui.label("No key bindings declared in the current skin.");
                        } else {
                            egui::Grid::new("keybinds_grid")
                                .striped(true)
                                .show(ui, |ui| {
                                    for (column, keys) in keybinding_rows.iter() {
                                        ui.label(format!("Column {}", column + 1));
                                        let display = keys.join(", ");
                                        if ui.button(&display).clicked() {
                                            // TODO : logiques de remappage à implémenter
                                        }
                                        ui.end_row();
                                    }
                                });
                        }

                        ui.add_space(10.0);
                        if ui.button("Done").clicked() {
                            settings_show_keybindings = false;
                        }
                    });
            }
        });
        
        // Mise à jour des settings après la closure
        self.settings.is_open = settings_is_open;
        self.settings.show_keybindings = settings_show_keybindings;
        self.settings.master_volume = master_volume;
        self.settings.hit_window_mode = hit_window_mode;
        self.settings.hit_window_value = hit_window_value;

        // --- 2. LOGIQUE DE JEU & FPS ---
        // (in_menu and show_result already extracted above)

        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_update).as_secs_f64();
        if elapsed >= 0.5 {
            self.fps = self.frame_count as f64 / elapsed;
            self.frame_count = 0;
            self.last_fps_update = now;
        }

        // Préparation des triangles Egui
        let tris = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image) in full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, id, &image);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        // --- 3. RENDER PASS DU JEU (Clear) ---
        if show_result {
            // Rendu de l'écran de résultats
            let ctx = ResultRenderContext {
                device: &self.device,
                queue: &self.queue,
                text_brush: &mut self.text_brush,
                view: &view,
                quad_pipeline: &self.quad_pipeline,
                quad_buffer: &self.quad_buffer,
                screen_width: self.config.width as f32,
                screen_height: self.config.height as f32,
                fps: self.fps,
            };

            self.result_view.update_size(ctx.screen_width, ctx.screen_height);
            self.result_view.render(
                ctx.device,
                ctx.queue,
                ctx.text_brush,
                ctx.view,
                ctx.quad_pipeline,
                ctx.quad_buffer,
                &self.engine.hit_stats,
                &self.engine.replay_data,
                self.engine.notes_passed,
                self.engine.hit_stats.calculate_accuracy(),
                self.engine.max_combo,
                &self.engine.hit_window,
            )?;
        } else if in_menu {
            // Charger les scores du leaderboard si nécessaire
            if !self.leaderboard_scores_loaded {
                self.load_leaderboard_scores();
            }
            
            self.update_menu_background();

            // Render background with wgpu
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            if let (Some(pipeline), Some(bind_group)) = (
                self.background_pipeline.as_ref(),
                self.background_bind_group.as_ref()
            ) {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Background Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
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
                        view: &view,
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

            self.queue.submit(std::iter::once(encoder.finish()));
            
        } else {
            // Rendu Gameplay
            // Ici, gameplay_view.render crée aussi son propre encoder/renderpass.
            let mut ctx = GameplayRenderContext {
                device: &self.device,
                queue: &self.queue,
                text_brush: &mut self.text_brush,
                render_pipeline: &self.render_pipeline,
                instance_buffer: &self.instance_buffer,
                receptor_buffer: &self.receptor_buffer,
                note_bind_groups: &self.note_bind_groups,
                receptor_bind_groups: &self.receptor_bind_groups,
                view: &view,
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
            )?;
        }

        // --- 4. RENDER PASS EGUI (Load) ---
        // On crée un encoder séparé pour egui pour éviter les problèmes de lifetime
        let mut egui_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Egui Render Encoder"),
        });

        // Mise à jour des buffers Egui
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut egui_encoder,
            &tris,
            &screen_descriptor,
        );

        // On fait une passe dédiée pour l'UI qui se dessine PAR DESSUS ce qui a déjà été fait
        {
            let mut rpass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // IMPORTANT : On garde l'image précédente (Jeu/Menu)
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // --- THE FIX ---
            // We transmute the lifetime of rpass to 'static to satisfy the overly strict 
            // bounds of the bleeding-edge egui-wgpu render function.
            // SAFETY: We know rpass is valid for this block, and we drop it immediately after.
            let rpass_static = unsafe { 
                std::mem::transmute::<&mut wgpu::RenderPass<'_>, &mut wgpu::RenderPass<'static>>(&mut rpass) 
            };

            self.egui_renderer.render(rpass_static, &tris, &screen_descriptor);
        } // rpass is dropped here

        let egui_command_buffer = egui_encoder.finish();

        // Nettoyage textures egui
        for id in full_output.textures_delta.free {
            self.egui_renderer.free_texture(&id);
        }

        // Soumission de la commande egui
        self.queue.submit(std::iter::once(egui_command_buffer));
        output.present();
        Ok(())
    }
}