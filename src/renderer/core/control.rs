use super::Renderer;
use crate::renderer::pipeline::create_bind_group_layout;
use crate::renderer::texture::load_texture_from_path;
use crate::shared::snapshot::RenderState;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::PhysicalKey;

impl Renderer {
    pub fn update_menu_background(&mut self) {
        let selected_beatmapset_image = match &self.current_state {
            RenderState::Menu(menu_state) => {
                 menu_state.get_selected_beatmapset()
                    .and_then(|(bs, _)| bs.image_path.as_ref().cloned())
            },
            RenderState::Result(_) => return, 
            _ => None,
        };

        if let Some(image_path) = selected_beatmapset_image {
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
                                label: Some("BG BG"),
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
        } else if self.current_background_path.is_some() && matches!(self.current_state, RenderState::Menu(_)) {
            self.current_background_path = None;
            self.background_texture = None;
            self.background_bind_group = None;
        }
    }

    pub fn decrease_note_size(&mut self) {
        self.gameplay_view.playfield_component_mut().config.decrease_note_size();
    }
    pub fn increase_note_size(&mut self) {
        self.gameplay_view.playfield_component_mut().config.increase_note_size();
    }

    pub fn update_pixel_system_ratio(&mut self) {
        let forced_ratio = match self.settings.aspect_ratio_mode {
            crate::models::settings::AspectRatioMode::Auto => None,
            crate::models::settings::AspectRatioMode::Ratio16_9 => Some(16.0 / 9.0),
            crate::models::settings::AspectRatioMode::Ratio4_3 => Some(4.0 / 3.0),
        };
        self.pixel_system.update_size(self.config.width, self.config.height, forced_ratio);
        self.update_component_positions();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.text_brush.resize_view(new_size.width as f32, new_size.height as f32, &self.queue);
            self.update_pixel_system_ratio();
        }
    }

    pub(crate) fn update_component_positions(&mut self) {
        let screen_width = self.config.width as f32;
        let screen_height = self.config.height as f32;
        let playfield_width_px = self.gameplay_view.playfield_component().get_total_width_pixels();

        let playfield_center_x = if let Some(pos) = self.skin.config.playfield_pos { pos.x } else { screen_width / 2.0 };
        let playfield_offset_y = if let Some(pos) = self.skin.config.playfield_pos { pos.y } else { 0.0 };
        let x_offset = playfield_center_x - (screen_width / 2.0);
        self.gameplay_view.playfield_component_mut().config.x_offset_pixels = x_offset;
        self.gameplay_view.playfield_component_mut().config.y_offset_pixels = playfield_offset_y;

        let default_combo_y = (screen_height / 2.0) - 80.0;
        let default_score_x = playfield_center_x + (playfield_width_px / 2.0) + 120.0;
        let default_score_y = screen_height * 0.05;
        let default_acc_x = playfield_center_x - (playfield_width_px / 2.0) - 150.0;

        let score_pos = self.skin.config.score_pos.unwrap_or(crate::models::skin::UIElementPos { x: default_score_x, y: default_score_y });
        self.score_display.set_position(score_pos.x, score_pos.y);
        self.score_display.set_size(self.skin.config.score_text_size);

        let combo_pos = self.skin.config.combo_pos.unwrap_or(crate::models::skin::UIElementPos { x: playfield_center_x, y: default_combo_y });
        self.combo_display.set_position(combo_pos.x, combo_pos.y);
        self.combo_display.set_size(self.skin.config.combo_text_size);

        let acc_pos = self.skin.config.accuracy_pos.unwrap_or(crate::models::skin::UIElementPos { x: default_acc_x, y: screen_height * 0.1 });
        self.accuracy_panel.set_position(acc_pos.x, acc_pos.y);
        self.accuracy_panel.set_size(self.skin.config.accuracy_text_size);

        let judge_pos = self.skin.config.judgement_pos.unwrap_or(crate::models::skin::UIElementPos { x: default_acc_x, y: screen_height * 0.15 });
        self.judgements_panel.set_position(judge_pos.x, judge_pos.y);
        self.judgements_panel.set_size(self.skin.config.judgement_text_size);

        let hitbar_width = playfield_width_px * 0.8;
        let hitbar_pos = self.skin.config.hit_bar_pos.unwrap_or(crate::models::skin::UIElementPos { x: playfield_center_x - hitbar_width / 2.0, y: combo_pos.y + 60.0 });
        self.hit_bar.set_geometry(hitbar_pos.x, hitbar_pos.y, hitbar_width, self.skin.config.hit_bar_height_px);

        let flash_pos = self.skin.config.judgement_flash_pos.unwrap_or(crate::models::skin::UIElementPos { x: playfield_center_x, y: combo_pos.y + 30.0 });
        self.judgement_flash.set_position(flash_pos.x, flash_pos.y);
    }

    pub fn handle_event(&mut self, window: &winit::window::Window, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput { event: key_event, .. } = event {
            if key_event.state == ElementState::Pressed {
                if let PhysicalKey::Code(keycode) = key_event.physical_key {
                    self.last_key_pressed = Some(keycode);
                }
            }
        }

        let response = self.egui_state.on_window_event(window, event);
        response.consumed
    }

    pub fn toggle_settings(&mut self) {
        self.settings.is_open = !self.settings.is_open;
    }

    pub fn load_leaderboard_scores(&mut self) {
        let selected_hash = match &self.current_state {
             RenderState::Menu(menu_state) => menu_state.get_selected_beatmap_hash(),
             RenderState::Result(res) => res.beatmap_hash.clone(),
             _ => None,
        };
        
        let needs_reload = match (&self.current_leaderboard_hash, &selected_hash) {
            (Some(c), Some(s)) => c != s,
            (None, Some(_)) => true,
            (_, None) => false,
        };

        if needs_reload || !self.leaderboard_scores_loaded {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let db_path = std::path::PathBuf::from("main.db");
            if let Ok(db) = rt.block_on(crate::database::connection::Database::new(&db_path)) {
                let (scores, note_count_map) = if let Some(hash) = &selected_hash {
                    let replays = rt.block_on(crate::database::query::get_replays_for_beatmap(db.pool(), hash)).unwrap_or_default();
                    let count = rt.block_on(sqlx::query_scalar::<_, i32>("SELECT note_count FROM beatmap WHERE hash = ?1").bind(hash).fetch_optional(db.pool())).ok().flatten().unwrap_or(0);
                    let mut map = std::collections::HashMap::new();
                    map.insert(hash.clone(), count);
                    (replays, map)
                } else {
                    let replays = rt.block_on(crate::database::query::get_top_scores(db.pool(), 10)).unwrap_or_default();
                    let mut map = std::collections::HashMap::new();
                    for r in &replays {
                        if !map.contains_key(&r.beatmap_hash) {
                            if let Ok(Some(c)) = rt.block_on(sqlx::query_scalar::<_, i32>("SELECT note_count FROM beatmap WHERE hash = ?1").bind(&r.beatmap_hash).fetch_optional(db.pool())) {
                                map.insert(r.beatmap_hash.clone(), c);
                            }
                        }
                    }
                    (replays, map)
                };
                if let Some(ref mut song_select) = self.song_select_screen {
                    song_select.update_leaderboard(scores, note_count_map);
                    song_select.set_current_beatmap_hash(selected_hash.clone());
                }
                self.leaderboard_scores_loaded = true;
                self.current_leaderboard_hash = selected_hash;
            }
        }
    }

    #[allow(dead_code)]
    pub fn ui(&mut self) {}
    #[allow(dead_code)]
    pub(crate) fn ui_with_context(&mut self, _ctx: &egui::Context) {}
}