//! Main renderer orchestrating all graphics operations.

#![allow(dead_code)]

use crate::core::input::actions::UIAction;
use crate::input::events::{EditMode, EditorTarget, GameAction};
use crate::models::skin::UIElementPos;
use crate::render::context::RenderContext;
use crate::render::draw::draw_game;
use crate::render::resources::RenderResources;
use crate::render::ui::UiOverlay;
use crate::shared::snapshot::RenderState;
use crate::views::components::menu::result_screen::ResultScreen;
use crate::views::components::menu::song_select::SongSelectScreen;
use crate::views::settings::{SettingsSnapshot, render_settings_window};
use std::sync::Arc;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::PhysicalKey;
use winit::window::Window;

pub struct Renderer {
    pub ctx: RenderContext,
    ui: UiOverlay,
    pub resources: RenderResources,
    current_state: RenderState,
    song_select_screen: SongSelectScreen,
    result_screen: ResultScreen,

    // FPS
    last_frame_time: std::time::Instant,
    frame_count: u32,
    last_fps_update: std::time::Instant,
    current_fps: f64,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let ctx = RenderContext::new(window.clone()).await;
        let ui = UiOverlay::new(window.clone(), &ctx.device, ctx.config.format);
        let mut resources = RenderResources::new(&ctx, &ui.ctx);

        resources.update_component_positions(ctx.config.width as f32, ctx.config.height as f32);

        Self {
            ctx,
            ui,
            resources,
            current_state: RenderState::Empty,
            song_select_screen: SongSelectScreen::new(),
            result_screen: ResultScreen::new(),
            last_frame_time: std::time::Instant::now(),
            frame_count: 0,
            last_fps_update: std::time::Instant::now(),
            current_fps: 0.0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.ctx.resize(new_size);
        self.resources
            .pixel_system
            .update_size(new_size.width, new_size.height, None);
        self.resources.text_brush.resize_view(
            new_size.width as f32,
            new_size.height as f32,
            &self.ctx.queue,
        );
        self.resources.update_component_positions(
            self.ctx.config.width as f32,
            self.ctx.config.height as f32,
        );
    }

    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let handled = self.ui.handle_input(window, event);

        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(code),
                    ..
                },
            ..
        } = event
            && self.resources.settings.remapping_column.is_some()
        {
            let label = format!("{:?}", code);
            self.resources.settings.push_keybind_key(label);
        }

        handled
    }

    pub fn update_state(&mut self, new_state: RenderState) {
        if let RenderState::Menu(ref menu) = new_state
            && let Some((set, _)) = menu.get_selected_beatmapset()
            && let Some(img_path) = &set.image_path
        {
            self.resources
                .load_background(&self.ctx.device, &self.ctx.queue, img_path);
        }
        self.current_state = new_state;
    }

    pub fn render(&mut self, window: &Window) -> Result<Vec<GameAction>, wgpu::SurfaceError> {
        // FPS
        self.frame_count += 1;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_fps_update);
        if elapsed.as_secs_f64() >= 1.0 {
            self.current_fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            self.last_fps_update = now;
        }

        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut actions_to_send = Vec::new();

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        // 1. Game Layer
        draw_game(
            &self.ctx,
            &mut self.resources,
            &mut encoder,
            &view,
            &self.current_state,
            self.current_fps,
        );

        // 2. UI Layer
        self.ui.begin_frame(window);
        let ctx_egui = self.ui.ctx.clone();

        match &self.current_state {
            RenderState::Menu(menu_state) => {
                if menu_state.show_settings {
                    let (snapshot, result) = {
                        let settings = &mut self.resources.settings;
                        let snapshot = SettingsSnapshot::capture(settings);
                        let result = render_settings_window(&ctx_egui, settings, &snapshot);
                        (snapshot, result)
                    };

                    if self.resources.settings.current_skin != snapshot.skin {
                        self.resources.settings.save();
                        self.resources = RenderResources::new(&self.ctx, &ctx_egui);
                        self.resources.update_component_positions(
                            self.ctx.config.width as f32,
                            self.ctx.config.height as f32,
                        );
                    }

                    if let Some(volume) = result.volume_changed {
                        actions_to_send.push(GameAction::UpdateVolume(volume));
                    }

                    if result.keybinds_updated {
                        actions_to_send.push(GameAction::ReloadKeybinds);
                    }

                    if result.request_toggle {
                        actions_to_send.push(GameAction::ToggleSettings);
                    }
                }

                let colors = &self.resources.skin.colors;
                let to_egui = |c: [f32; 4]| {
                    egui::Color32::from_rgba_unmultiplied(
                        (c[0] * 255.) as u8,
                        (c[1] * 255.) as u8,
                        (c[2] * 255.) as u8,
                        (c[3] * 255.) as u8,
                    )
                };
                let dummy_win = crate::models::engine::hit_window::HitWindow::new();

                // Créer un hit_window à jour avec les settings actuels
                let current_hit_window = match self.resources.settings.hit_window_mode {
                    crate::models::settings::HitWindowMode::OsuOD => {
                        crate::models::engine::hit_window::HitWindow::from_osu_od(
                            self.resources.settings.hit_window_value,
                        )
                    }
                    crate::models::settings::HitWindowMode::EtternaJudge => {
                        crate::models::engine::hit_window::HitWindow::from_etterna_judge(
                            self.resources.settings.hit_window_value as u8,
                        )
                    }
                };

                // Build panel textures struct
                let panel_textures = crate::views::components::menu::song_select::UIPanelTextures {
                    beatmap_info_bg: self.resources.beatmap_info_bg_texture.as_ref().map(|t| t.id()),
                    search_panel_bg: self.resources.search_panel_bg_texture.as_ref().map(|t| t.id()),
                    search_bar: self.resources.search_bar_texture.as_ref().map(|t| t.id()),
                    leaderboard_bg: self.resources.leaderboard_bg_texture.as_ref().map(|t| t.id()),
                };

                let (action_opt, result_data, search_request, _calculator_changed) = self.song_select_screen.render(
                    &ctx_egui,
                    menu_state,
                    &view,
                    self.ctx.config.width as f32,
                    self.ctx.config.height as f32,
                    &current_hit_window,
                    self.resources.settings.hit_window_mode,
                    self.resources.settings.hit_window_value,
                    self.resources.song_button_texture.as_ref().map(|t| t.id()),
                    self.resources
                        .song_button_selected_texture
                        .as_ref()
                        .map(|t| t.id()),
                    self.resources
                        .difficulty_button_texture
                        .as_ref()
                        .map(|t| t.id()),
                    self.resources
                        .difficulty_button_selected_texture
                        .as_ref()
                        .map(|t| t.id()),
                    to_egui(colors.selected_color),
                    to_egui(colors.difficulty_selected_color),
                    &panel_textures,
                );
                
                // Handle calculator change
                if let Some(calc_id) = _calculator_changed {
                    actions_to_send.push(GameAction::SetCalculator(calc_id));
                }

                if let Some(a) = action_opt {
                    match a {
                        UIAction::SetSelection(i) => {
                            actions_to_send.push(GameAction::SetSelection(i))
                        }
                        UIAction::SetDifficulty(i) => {
                            actions_to_send.push(GameAction::SetDifficulty(i))
                        }
                        UIAction::Select => actions_to_send.push(GameAction::Confirm),
                        UIAction::Back => actions_to_send.push(GameAction::Back),
                        UIAction::ToggleSettings => {
                            actions_to_send.push(GameAction::ToggleSettings)
                        }
                        _ => {}
                    }
                }

                // Traiter les clics sur les scores (ouvrir l'écran de résultat)
                if let Some(result_data) = result_data {
                    // Envoyer l'action au thread de logique pour changer d'état
                    actions_to_send.push(GameAction::SetResult(result_data));
                }

                // Traiter les requêtes de recherche
                if let Some(filters) = search_request {
                    actions_to_send.push(GameAction::ApplySearch(filters));
                }
            }

            // --- ÉDITEUR CORRIGÉ ---
            RenderState::Editor(snapshot) => {
                // Capturer les événements de souris pour l'éditeur
                let is_dragging = ctx_egui.input(|i| i.pointer.primary_down());

                if let (Some(target), true) = (snapshot.target, is_dragging) {
                    // Calculer le delta depuis la dernière position
                    // On utilise le delta de la souris depuis egui
                    let delta = ctx_egui.input(|i| {
                        let delta = i.pointer.delta();
                        (delta.x, delta.y)
                    });

                    if delta.0 != 0.0 || delta.1 != 0.0 {
                        // Envoyer l'action de modification
                        actions_to_send.push(GameAction::EditorModify {
                            x: delta.0,
                            y: delta.1,
                        });
                    }
                }

                if let Some((target, mode, dx, dy)) = snapshot.modification {
                    let config = &mut self.resources.skin.config;
                    let speed = 2.0;

                    match (target, mode) {
                        // Redimensionnement
                        (EditorTarget::Notes, EditMode::Resize) => {
                            config.note_width_px += dx * speed;
                            config.note_height_px -= dy * speed;
                        }
                        (EditorTarget::Receptors, EditMode::Resize) => {
                            config.receptor_width_px += dx * speed;
                            config.receptor_height_px -= dy * speed;
                        }
                        (EditorTarget::Combo, EditMode::Resize) => {
                            config.combo_text_size -= dy * speed
                        }
                        (EditorTarget::Score, EditMode::Resize) => {
                            config.score_text_size -= dy * speed
                        }
                        (EditorTarget::Accuracy, EditMode::Resize) => {
                            config.accuracy_text_size -= dy * speed
                        }
                        (EditorTarget::Judgement, EditMode::Resize) => {
                            config.judgement_text_size -= dy * speed
                        }
                        (EditorTarget::HitBar, EditMode::Resize) => {
                            config.hit_bar_height_px -= dy * speed
                        }

                        // Déplacement (Valable pour TOUS les targets)
                        (t, EditMode::Move) => {
                            let pos_opt = match t {
                                EditorTarget::Notes
                                | EditorTarget::Lanes
                                | EditorTarget::Receptors => &mut config.playfield_pos,
                                EditorTarget::Combo => &mut config.combo_pos,
                                EditorTarget::Score => &mut config.score_pos,
                                EditorTarget::Accuracy => &mut config.accuracy_pos,
                                EditorTarget::Judgement => &mut config.judgement_pos,
                                EditorTarget::HitBar => &mut config.hit_bar_pos,
                                _ => &mut None,
                            };
                            let p = pos_opt.get_or_insert(UIElementPos { x: 0., y: 0. });
                            p.x += dx * speed;
                            p.y -= dy * speed;
                        }
                        _ => {}
                    }
                    self.resources.update_component_positions(
                        self.ctx.config.width as f32,
                        self.ctx.config.height as f32,
                    );
                }

                if snapshot.save_requested {
                    let _ = self.resources.skin.save_user_config();
                }

                egui::Window::new("Editor")
                    .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                    .show(&ctx_egui, |ui| {
                        ui.label(&snapshot.status_text);

                        // Affichage conditionnel selon le mode
                        if let Some(target) = snapshot.target {
                            let config = &self.resources.skin.config;
                            let text = match (target, snapshot.mode) {
                                // Mode MOVE : On affiche la position
                                (t, EditMode::Move) => {
                                    let pos = match t {
                                        EditorTarget::Notes
                                        | EditorTarget::Lanes
                                        | EditorTarget::Receptors => config.playfield_pos,
                                        EditorTarget::Combo => config.combo_pos,
                                        EditorTarget::Score => config.score_pos,
                                        EditorTarget::Accuracy => config.accuracy_pos,
                                        EditorTarget::Judgement => config.judgement_pos,
                                        EditorTarget::HitBar => config.hit_bar_pos,
                                        _ => None,
                                    }
                                    .unwrap_or(UIElementPos { x: 0., y: 0. });
                                    format!("Pos: X {:.0} Y {:.0}", pos.x, pos.y)
                                }
                                // Mode RESIZE : On affiche la taille
                                (EditorTarget::Notes, EditMode::Resize) => format!(
                                    "Size: W {:.0} H {:.0}",
                                    config.note_width_px, config.note_height_px
                                ),
                                (EditorTarget::Receptors, EditMode::Resize) => format!(
                                    "Size: W {:.0} H {:.0}",
                                    config.receptor_width_px, config.receptor_height_px
                                ),
                                (EditorTarget::Combo, EditMode::Resize) => {
                                    format!("Size: {:.0}", config.combo_text_size)
                                }
                                (EditorTarget::Score, EditMode::Resize) => {
                                    format!("Size: {:.0}", config.score_text_size)
                                }
                                (EditorTarget::Accuracy, EditMode::Resize) => {
                                    format!("Size: {:.0}", config.accuracy_text_size)
                                }
                                (EditorTarget::Judgement, EditMode::Resize) => {
                                    format!("Size: {:.0}", config.judgement_text_size)
                                }
                                (EditorTarget::HitBar, EditMode::Resize) => {
                                    format!("Height: {:.0}", config.hit_bar_height_px)
                                }
                                _ => "Mode not supported".to_string(),
                            };
                            ui.label(
                                egui::RichText::new(text)
                                    .color(egui::Color32::YELLOW)
                                    .size(20.0),
                            );
                        }

                        if ui.button("Save Config (S)").clicked() {
                            actions_to_send.push(GameAction::EditorSave);
                        }
                    });
            }

            RenderState::Result(data) => {
                let hit_win = crate::models::engine::hit_window::HitWindow::new();
                if self.result_screen.render(&ctx_egui, data, &hit_win) {
                    actions_to_send.push(GameAction::Back);
                }
            }

            // Practice Mode overlay pendant le gameplay
            RenderState::InGame(snapshot) => {
                if snapshot.practice_mode {
                    egui::Area::new(egui::Id::new("practice_overlay"))
                        .fixed_pos(egui::pos2(0.0, 0.0))
                        .show(&ctx_egui, |ui| {
                            crate::views::components::PracticeOverlay::render(
                                ui,
                                snapshot.audio_time,
                                snapshot.map_duration,
                                &snapshot.checkpoints,
                                self.ctx.config.width as f32,
                            );
                        });
                }
            }
            _ => {}
        }

        self.ui.end_frame_and_draw(&self.ctx, &mut encoder, &view);
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(actions_to_send)
    }
}
