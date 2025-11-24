use super::Renderer;
use crate::models::engine::GameEngine;
use crate::renderer::pipeline::create_bind_group_layout;
use crate::renderer::texture::load_texture_from_path;
use std::path::PathBuf;
use winit::event::WindowEvent;

impl Renderer {
    pub fn update_menu_background(&mut self) {
        let selected_beatmapset = {
            if let Ok(menu_state) = self.menu_state.lock() {
                menu_state
                    .get_selected_beatmapset()
                    .and_then(|(bs, _)| bs.image_path.as_ref().cloned())
            } else {
                None
            }
        };

        if let Some(image_path) = selected_beatmapset {
            if self.current_background_path.as_ref() != Some(&image_path) {
                self.current_background_path = Some(image_path.clone());
                let path = std::path::Path::new(&image_path);

                if path.exists() {
                    if let Some((texture, _, _)) =
                        load_texture_from_path(&self.device, &self.queue, path)
                    {
                        let texture_view =
                            texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let bind_group_layout = create_bind_group_layout(&self.device);
                        let bind_group =
                            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Background Bind Group"),
                                layout: &bind_group_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::TextureView(&texture_view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::Sampler(
                                            &self.background_sampler,
                                        ),
                                    },
                                ],
                            });

                        self.background_texture = Some(texture);
                        self.background_bind_group = Some(bind_group);
                    }
                } else {
                    self.background_texture = None;
                    self.background_bind_group = None;
                }
            }
        } else if self.current_background_path.is_some() {
            self.current_background_path = None;
            self.background_texture = None;
            self.background_bind_group = None;
        }
    }

    pub fn load_map(&mut self, map_path: PathBuf) {
        let rate = if let Ok(menu_state) = self.menu_state.lock() {
            menu_state.rate
        } else {
            1.0
        };
        self.engine = GameEngine::from_map(map_path, rate);
        
        // Appliquer la hit window depuis les settings
        self.engine.update_hit_window(self.settings.hit_window_mode, self.settings.hit_window_value);
        
        // Recréer le buffer quad avec la bonne taille en fonction du nombre de notes
        // On a besoin de : nombre de notes (pour le graphe) + 11 quads (panneaux, background, ligne centrale)
        let num_notes = self.engine.chart.len();
        let required_quads = num_notes + 11;
        self.resize_quad_buffer(required_quads);
    }
    
    /// Recrée le buffer quad avec une nouvelle taille
    fn resize_quad_buffer(&mut self, num_quads: usize) {
        use crate::views::components::common::QuadInstance;
        
        let buffer_size = (num_quads * std::mem::size_of::<QuadInstance>()) as u64;
        self.quad_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }

    pub fn stop_audio(&mut self) {
        if let Ok(sink) = self.engine.audio_sink.lock() {
            sink.stop();
            sink.clear();
        }
    }

    pub fn decrease_note_size(&mut self) {
        self.gameplay_view
            .playfield_component_mut()
            .config
            .decrease_note_size();
    }

    pub fn increase_note_size(&mut self) {
        self.gameplay_view
            .playfield_component_mut()
            .config
            .increase_note_size();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.text_brush
                .resize_view(new_size.width as f32, new_size.height as f32, &self.queue);
            self.pixel_system
                .update_size(new_size.width, new_size.height);
            self.update_component_positions();
        }
    }

    pub fn load_leaderboard_scores(&mut self) {
        // Obtenir le hash de la map sélectionnée
        let selected_hash = if let Ok(menu_state) = self.menu_state.lock() {
            menu_state.get_selected_beatmap_hash()
        } else {
            None
        };

        // Vérifier si on doit recharger (map différente ou pas encore chargé)
        let needs_reload = match (&self.current_leaderboard_hash, &selected_hash) {
            (Some(current), Some(selected)) => current != selected,
            (None, Some(_)) => true, // Pas encore chargé mais une map est sélectionnée
            (_, None) => false, // Pas de map sélectionnée
        };

        if needs_reload || !self.leaderboard_scores_loaded {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let db_path = std::path::PathBuf::from("main.db");
            
            if let Ok(db) = rt.block_on(crate::database::connection::Database::new(&db_path)) {
                let scores = if let Some(hash) = &selected_hash {
                    // Charger les scores pour la map spécifique
                    rt.block_on(crate::database::query::get_replays_for_beatmap(db.pool(), hash))
                        .unwrap_or_else(|_| Vec::new())
                } else {
                    // Pas de map sélectionnée, charger les top scores globaux
                    rt.block_on(crate::database::query::get_top_scores(db.pool(), 10))
                        .unwrap_or_else(|_| Vec::new())
                };

                if let Some(ref mut song_select) = self.song_select_screen {
                    song_select.update_leaderboard(scores);
                    song_select.set_current_beatmap_hash(selected_hash.clone());
                }
                self.leaderboard_scores_loaded = true;
                self.current_leaderboard_hash = selected_hash;
            }
        }
    }

    pub(crate) fn update_component_positions(&mut self) {
        let screen_width = self.config.width as f32;
        let screen_height = self.config.height as f32;

        let (_, _playfield_width) = self
            .gameplay_view
            .playfield_component()
            .get_bounds(&self.pixel_system);
        let playfield_screen_width = _playfield_width * screen_height / 2.0;
        let playfield_center_x = screen_width / 2.0;
        let left_x =
            ((screen_width / 2.0) - playfield_screen_width - (screen_width * 0.15).min(200.0))
                .max(20.0);
        let playfield_right_x = playfield_center_x + (playfield_screen_width / 2.0);
        let score_x = playfield_right_x + 20.0;
        let combo_y = (screen_height / 2.0) - 80.0;
        let judgement_y = combo_y + 30.0;
        let hitbar_y = combo_y + 60.0;
        let hitbar_width = playfield_screen_width * 0.8;

        self.combo_display.set_position(playfield_center_x, combo_y);
        self.judgement_flash
            .set_position(playfield_center_x, judgement_y);
        self.hit_bar.set_geometry(
            playfield_center_x - hitbar_width / 2.0,
            hitbar_y,
            hitbar_width,
            20.0,
        );
        self.score_display
            .set_position(score_x, screen_height * 0.05);
        self.accuracy_panel
            .set_position(left_x, screen_height * 0.1);
        self.judgements_panel
            .set_position(left_x, screen_height * 0.15);
    }

    pub fn handle_event(&mut self, window: &winit::window::Window, event: &WindowEvent) {
        let _ = self.egui_state.on_window_event(window, event);
    }

    // Fonction pour basculer le menu (à appeler quand on détecte Ctrl+O)
    pub fn toggle_settings(&mut self) {
        self.settings.is_open = !self.settings.is_open;
    }

    // La logique de construction de l'interface
    // Note: Cette méthode n'est plus utilisée directement car on construit l'UI dans draw.rs
    // pour éviter les problèmes de borrow. On la garde pour compatibilité.
    #[allow(dead_code)]
    pub fn ui(&mut self) {
        // Cette méthode n'est plus utilisée
    }

    // Version qui accepte le contexte egui en paramètre (pour éviter les problèmes de borrow)
    // Note: Cette méthode n'est plus utilisée non plus
    #[allow(dead_code)]
    pub(crate) fn ui_with_context(&mut self, ctx: &egui::Context) {
        if !self.settings.is_open {
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
                if ui.add(egui::Slider::new(&mut self.settings.master_volume, 0.0..=1.0).text("Volume")).changed() {
                    // Appliquer le volume immédiatement à l'engine
                    if let Ok(sink) = self.engine.audio_sink.lock() {
                        sink.set_volume(self.settings.master_volume);
                    }
                }

                ui.separator();
                ui.label("Controls");
                
                // Bouton pour ouvrir le remapping
                if ui.button("Remap Keys").clicked() {
                    self.settings.show_keybindings = true;
                }

                ui.add_space(20.0);
                if ui.button("Close (Ctrl+O)").clicked() {
                    self.settings.is_open = false;
                }
            });

        // 2. Fenêtre Centrale (Modal) pour le Keybinding
        if self.settings.show_keybindings {
            egui::Window::new("Key Bindings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]) // Centrer
                .show(ctx, |ui| {
                    ui.label("Click on a button to rebind (Not implemented yet logic-wise)");
                    
                    // Exemple de grille pour les touches
                    egui::Grid::new("keybinds_grid").striped(true).show(ui, |ui| {
                        ui.label("Column 1");
                        if ui.button("D").clicked() { /* Logique de capture de touche ici */ }
                        ui.end_row();

                        ui.label("Column 2");
                        if ui.button("F").clicked() { /* ... */ }
                        ui.end_row();
                        // etc...
                    });

                    ui.add_space(10.0);
                    if ui.button("Done").clicked() {
                        self.settings.show_keybindings = false;
                    }
                });
        }
    }
}
