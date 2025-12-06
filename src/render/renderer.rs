//! Main renderer orchestrating all graphics operations.

#![allow(dead_code)]

use crate::core::input::actions::UIAction;
use crate::input::events::GameAction;
use crate::render::context::RenderContext;
use crate::render::draw::draw_game;
use crate::render::mock_data::create_mock_state;
use crate::render::resources::RenderResources;
use crate::render::ui::UiOverlay;
use crate::shared::snapshot::RenderState;
use crate::views::components::editor::SkinEditorLayout;
use crate::views::components::menu::result_screen::ResultScreen;
use crate::views::components::menu::song_select::SongSelectScreen;
use crate::views::settings::{SettingsSnapshot, render_settings_window};
use std::sync::Arc;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::PhysicalKey;
use winit::window::Window;

pub struct Renderer {
    pub ctx: RenderContext,

    // UI Principale (affichée à l'écran)
    ui: UiOverlay,

    // UI Secondaire (pour le rendu dans la texture de l'éditeur)
    offscreen_ui: UiOverlay,

    pub resources: RenderResources,
    current_state: RenderState,

    // Screens
    song_select_screen: SongSelectScreen,
    result_screen: ResultScreen,
    skin_editor: SkinEditorLayout,

    // Offscreen Rendering (pour l'éditeur)
    offscreen_texture: Option<wgpu::Texture>,
    offscreen_view: Option<wgpu::TextureView>,
    offscreen_id: Option<egui::TextureId>,
    offscreen_size: (u32, u32),

    // FPS
    last_frame_time: std::time::Instant,
    frame_count: u32,
    last_fps_update: std::time::Instant,
    current_fps: f64,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let ctx = RenderContext::new(window.clone()).await;

        // Instance UI pour la fenêtre principale
        let ui = UiOverlay::new(window.clone(), &ctx.device, ctx.config.format);

        // Instance UI pour le rendu offscreen (prévisualisation)
        // On utilise le même format de pixel que la swapchain pour simplifier
        let offscreen_ui = UiOverlay::new(window.clone(), &ctx.device, ctx.config.format);

        let mut resources = RenderResources::new(&ctx, &ui.ctx);

        // Positionnement initial des éléments
        resources.update_component_positions(ctx.config.width as f32, ctx.config.height as f32);

        Self {
            ctx,
            ui,
            offscreen_ui,
            resources,
            current_state: RenderState::Empty,

            song_select_screen: SongSelectScreen::new(),
            result_screen: ResultScreen::new(),
            skin_editor: SkinEditorLayout::new(),

            offscreen_texture: None,
            offscreen_view: None,
            offscreen_id: None,
            offscreen_size: (0, 0),

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

    /// Prépare la texture offscreen pour le rendu de l'éditeur
    fn ensure_offscreen_texture(&mut self, width: u32, height: u32) {
        if self.offscreen_texture.is_some() && self.offscreen_size == (width, height) {
            return;
        }

        // Libérer l'ancienne texture Egui si elle existe
        if let Some(id) = self.offscreen_id {
            self.ui.free_texture(id);
        }

        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.ctx.config.format,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("Editor Offscreen Texture"),
            view_formats: &[],
        };

        let texture = self.ctx.device.create_texture(&texture_desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Enregistrement de la nouvelle texture dans l'UI principale pour l'afficher
        let id = self
            .ui
            .register_texture(&self.ctx.device, &view, wgpu::FilterMode::Linear);

        self.offscreen_texture = Some(texture);
        self.offscreen_view = Some(view);
        self.offscreen_id = Some(id);
        self.offscreen_size = (width, height);

        log::info!("RENDER: Created offscreen texture {}x{}", width, height);
    }

    pub fn render(&mut self, window: &Window) -> Result<Vec<GameAction>, wgpu::SurfaceError> {
        // --- FPS Calculation ---
        self.frame_count += 1;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_fps_update);
        if elapsed.as_secs_f64() >= 1.0 {
            self.current_fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            self.last_fps_update = now;
        }

        // Préparation de la frame
        let output = self.ctx.surface.get_current_texture()?;
        let swapchain_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut actions_to_send = Vec::new();

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        let is_editor = matches!(self.current_state, RenderState::Editor(_));

        // =================================================================================
        // 1. GAME RENDERING LAYER (Offscreen ou Onscreen)
        // =================================================================================

        if is_editor {
            // --- MODE ÉDITEUR : RENDU OFFSCREEN ---

            // 1. Récupérer la résolution désirée depuis l'éditeur
            let w = self.skin_editor.state.preview_width;
            let h = self.skin_editor.state.preview_height;
            self.ensure_offscreen_texture(w, h);

            if let Some(target_view) = &self.offscreen_view {
                // 2. Adapter le système de coordonnées à la résolution offscreen
                self.resources.pixel_system.update_size(w, h, None);

                // 3. Créer l'état factice (Mock)
                let mock_state = create_mock_state(self.skin_editor.state.current_scene);

                // 4. Rendu WGPU (Jeu / Background / Notes)
                draw_game(
                    &self.ctx,
                    &mut self.resources,
                    &mut encoder,
                    target_view,
                    &mock_state,
                    self.current_fps,
                );

                // 5. Rendu EGUI OFFSCREEN (Menus SongSelect / Result)
                // C'est ici qu'on dessine l'UI du menu DANS la texture
                match &mock_state {
                    RenderState::Menu(menu_state) => {
                        self.offscreen_ui.begin_frame(window); // Dummy inputs pour l'offscreen
                        let ctx_off = self.offscreen_ui.ctx.clone();

                        let hit_win = crate::models::engine::hit_window::HitWindow::new();
                        let menus = &self.resources.skin.menus;

                        let to_egui = |c: [f32; 4]| {
                            egui::Color32::from_rgba_unmultiplied(
                                (c[0] * 255.) as u8,
                                (c[1] * 255.) as u8,
                                (c[2] * 255.) as u8,
                                (c[3] * 255.) as u8,
                            )
                        };

                        let panel_textures =
                            crate::views::components::menu::song_select::UIPanelTextures {
                                beatmap_info_bg: self
                                    .resources
                                    .beatmap_info_bg_texture
                                    .as_ref()
                                    .map(|t| t.id()),
                                search_panel_bg: self
                                    .resources
                                    .search_panel_bg_texture
                                    .as_ref()
                                    .map(|t| t.id()),
                                search_bar: self
                                    .resources
                                    .search_bar_texture
                                    .as_ref()
                                    .map(|t| t.id()),
                                leaderboard_bg: self
                                    .resources
                                    .leaderboard_bg_texture
                                    .as_ref()
                                    .map(|t| t.id()),
                            };

                        // Rendu du menu avec la taille offscreen
                        self.song_select_screen.render(
                            &ctx_off,
                            menu_state,
                            target_view,
                            w as f32,
                            h as f32, // Dimensions de la preview !
                            &hit_win,
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
                            to_egui(menus.song_select.song_button.selected_border_color),
                            to_egui(menus.song_select.difficulty_button.selected_text_color),
                            &panel_textures,
                        );

                        // Finaliser le rendu Egui offscreen dans la texture
                        self.offscreen_ui
                            .end_frame_and_draw(&self.ctx, &mut encoder, target_view);
                    }
                    RenderState::Result(data) => {
                        self.offscreen_ui.begin_frame(window);
                        let ctx_off = self.offscreen_ui.ctx.clone();
                        let hit_win = crate::models::engine::hit_window::HitWindow::new();

                        self.result_screen.render(&ctx_off, data, &hit_win);

                        self.offscreen_ui
                            .end_frame_and_draw(&self.ctx, &mut encoder, target_view);
                    }
                    _ => {} // En jeu (InGame), le HUD est géré par draw_game via wgpu_text
                }

                // 6. Restaurer la taille réelle de la fenêtre pour le rendu principal
                self.resources.pixel_system.update_size(
                    self.ctx.config.width,
                    self.ctx.config.height,
                    None,
                );
            }
        } else {
            // --- MODE NORMAL : RENDU ONSCREEN ---
            draw_game(
                &self.ctx,
                &mut self.resources,
                &mut encoder,
                &swapchain_view,
                &self.current_state,
                self.current_fps,
            );
        }

        // =================================================================================
        // 2. UI LAYER PRINCIPALE (SWAPCHAIN)
        // =================================================================================

        self.ui.begin_frame(window);
        let ctx_egui = self.ui.ctx.clone();

        match &self.current_state {
            RenderState::Menu(menu_state) => {
                // Gestion de la fenêtre de Settings (Popup)
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
                    if let Some((mode, value)) = result.hit_window_changed {
                        actions_to_send.push(GameAction::UpdateHitWindow { mode, value });
                    }
                    if result.keybinds_updated {
                        actions_to_send.push(GameAction::ReloadKeybinds);
                    }
                    if result.request_toggle {
                        actions_to_send.push(GameAction::ToggleSettings);
                    }
                }

                let menus = &self.resources.skin.menus;
                let to_egui = |c: [f32; 4]| {
                    egui::Color32::from_rgba_unmultiplied(
                        (c[0] * 255.) as u8,
                        (c[1] * 255.) as u8,
                        (c[2] * 255.) as u8,
                        (c[3] * 255.) as u8,
                    )
                };
                let mut hit_window = match self.resources.settings.hit_window_mode {
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
                let panel_textures = crate::views::components::menu::song_select::UIPanelTextures {
                    beatmap_info_bg: self
                        .resources
                        .beatmap_info_bg_texture
                        .as_ref()
                        .map(|t| t.id()),
                    search_panel_bg: self
                        .resources
                        .search_panel_bg_texture
                        .as_ref()
                        .map(|t| t.id()),
                    search_bar: self.resources.search_bar_texture.as_ref().map(|t| t.id()),
                    leaderboard_bg: self
                        .resources
                        .leaderboard_bg_texture
                        .as_ref()
                        .map(|t| t.id()),
                };

                let (action_opt, result_data, search_request, calculator_changed) =
                    self.song_select_screen.render(
                        &ctx_egui,
                        menu_state,
                        &swapchain_view,
                        self.ctx.config.width as f32,
                        self.ctx.config.height as f32,
                        &hit_window,
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
                        to_egui(menus.song_select.song_button.selected_border_color),
                        to_egui(menus.song_select.difficulty_button.selected_text_color),
                        &panel_textures,
                    );

                if let Some(calc_id) = calculator_changed {
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

                if let Some(result_data) = result_data {
                    actions_to_send.push(GameAction::SetResult(result_data));
                }

                if let Some(filters) = search_request {
                    actions_to_send.push(GameAction::ApplySearch(filters));
                }
            }

            RenderState::Editor(_snapshot) => {
                // Affiche l'UI de l'éditeur
                // Affiche l'UI de l'éditeur
                if self
                    .skin_editor
                    .show(&ctx_egui, &mut self.resources.skin, self.offscreen_id)
                {
                    let s = self.resources.skin.clone();
                    self.resources.reload_textures(&self.ctx, &ctx_egui, &s);
                }

                // MISE À JOUR TEMPS RÉEL DES POSITIONS
                // On met à jour les RenderResources avec les dimensions de la preview
                // si elle existe, sinon avec la taille écran.
                // Cela permet de voir les éléments bouger instantanément.
                let (w, h) = if let Some(_) = &self.offscreen_view {
                    (
                        self.skin_editor.state.preview_width as f32,
                        self.skin_editor.state.preview_height as f32,
                    )
                } else {
                    (self.ctx.config.width as f32, self.ctx.config.height as f32)
                };

                self.resources.update_component_positions(w, h);
            }

            RenderState::Result(data) => {
                let mut hit_window_updated = false;

                if data.show_settings {
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
                    if let Some((mode, value)) = result.hit_window_changed {
                        actions_to_send.push(GameAction::UpdateHitWindow { mode, value });
                        hit_window_updated = true;
                    }
                    if result.keybinds_updated {
                        actions_to_send.push(GameAction::ReloadKeybinds);
                    }
                    if result.request_toggle {
                        actions_to_send.push(GameAction::ToggleSettings);
                    }
                }

                // Only render result screen if settings didn't just trigger a re-judge
                // (though technically concurrent rendering is fine, this follows Menu pattern)
                let hit_win = crate::models::engine::hit_window::HitWindow::new();
                if self.result_screen.render(&ctx_egui, data, &hit_win) {
                    actions_to_send.push(GameAction::Back);
                }
            }

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

        self.ui
            .end_frame_and_draw(&self.ctx, &mut encoder, &swapchain_view);
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(actions_to_send)
    }
}
