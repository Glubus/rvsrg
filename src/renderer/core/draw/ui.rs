use crate::models::engine::NUM_COLUMNS;
use crate::renderer::Renderer;
use crate::shared::snapshot::RenderState;
use egui_wgpu::ScreenDescriptor;
use wgpu::{CommandBuffer, CommandEncoderDescriptor, TextureView};
use crate::core::input::actions::{KeyAction, GameAction, UIAction};
use crate::shared::messages::MainToLogic;

impl Renderer {
    pub fn update_ui(
        &mut self,
        window: &winit::window::Window,
        view: &TextureView,
    ) -> (Vec<egui::ClippedPrimitive>, egui::TexturesDelta, bool, Vec<MainToLogic>) {
        let raw_input = self.egui_state.take_egui_input(window);
        
        let mut captured_key_str: Option<String> = None;
        if self.settings.remapping_column.is_some() {
            if let Some(code) = self.last_key_pressed {
                captured_key_str = Some(format!("{:?}", code));
            }
        }
        self.last_key_pressed = None;

        let mut should_show_settings = false; 
        let mut settings_changed_this_frame = false;
        let mut ui_messages = Vec::new();

        let mut settings_show_keybindings = self.settings.show_keybindings;
        let mut remapping_column = self.settings.remapping_column;
        let mut master_volume = self.settings.master_volume;
        let mut scroll_speed = self.settings.scroll_speed;
        let mut hit_window_mode = self.settings.hit_window_mode;
        let mut hit_window_value = self.settings.hit_window_value;
        let mut aspect_ratio_mode = self.settings.aspect_ratio_mode;
        
        let num_cols_str = NUM_COLUMNS.to_string();
        let current_binds = self.settings.keybinds.get(&num_cols_str).cloned().unwrap_or_default();
        let keybinding_rows: Vec<(usize, String)> = (0..NUM_COLUMNS)
            .map(|col| {
                let key = current_binds.get(col).cloned().unwrap_or_else(|| "None".to_string());
                (col, key)
            })
            .collect();

        let btn_tex = self.song_button_texture.as_ref().map(|t| t.id());
        let btn_sel_tex = self.song_button_selected_texture.as_ref().map(|t| t.id());
        let diff_tex = self.difficulty_button_texture.as_ref().map(|t| t.id());
        let diff_sel_tex = self.difficulty_button_selected_texture.as_ref().map(|t| t.id());
        let sel_col_array = self.skin.colors.selected_color;
        let song_selected_color = egui::Color32::from_rgba_unmultiplied((sel_col_array[0]*255.) as u8, (sel_col_array[1]*255.) as u8, (sel_col_array[2]*255.) as u8, (sel_col_array[3]*255.) as u8);
        let diff_col_array = self.skin.colors.difficulty_selected_color;
        let difficulty_selected_color = egui::Color32::from_rgba_unmultiplied((diff_col_array[0]*255.) as u8, (diff_col_array[1]*255.) as u8, (diff_col_array[2]*255.) as u8, (diff_col_array[3]*255.) as u8);

        let egui_ctx = std::mem::take(&mut self.egui_ctx);
        
        let full_output = egui_ctx.run(raw_input, |ctx| {
            if matches!(self.current_state, RenderState::Menu(_)) {
                 if self.song_select_screen.is_none() {
                     self.song_select_screen = Some(crate::views::components::menu::song_select::SongSelectScreen::new());
                 }
                 if !self.leaderboard_scores_loaded {
                     self.load_leaderboard_scores();
                 }
            }

            match &mut self.current_state {
                RenderState::Result(data) => {
                    if self.result_screen.is_none() {
                        self.result_screen = Some(crate::views::components::menu::result_screen::ResultScreen::new());
                    }
                    let current_hit_window = match hit_window_mode {
                        crate::models::settings::HitWindowMode::OsuOD => crate::models::engine::hit_window::HitWindow::from_osu_od(hit_window_value),
                        crate::models::settings::HitWindowMode::EtternaJudge => crate::models::engine::hit_window::HitWindow::from_etterna_judge(hit_window_value as u8),
                    };
                    if let Some(ref mut screen) = self.result_screen {
                        let should_close = screen.render(ctx, data, &current_hit_window);
                        if should_close {
                             ui_messages.push(MainToLogic::TransitionToMenu);
                        }
                    }
                },
                RenderState::Menu(menu_state) => {
                    should_show_settings = menu_state.show_settings;
                    
                    let current_hit_window = match hit_window_mode {
                        crate::models::settings::HitWindowMode::OsuOD => crate::models::engine::hit_window::HitWindow::from_osu_od(hit_window_value),
                        crate::models::settings::HitWindowMode::EtternaJudge => crate::models::engine::hit_window::HitWindow::from_etterna_judge(hit_window_value as u8),
                    };

                    // Vérification : Doit-on afficher le résultat ou le menu standard ?
                    if menu_state.show_result {
                        if let Some(result_data) = &menu_state.last_result {
                            if self.result_screen.is_none() {
                                self.result_screen = Some(crate::views::components::menu::result_screen::ResultScreen::new());
                            }
                            if let Some(ref mut screen) = self.result_screen {
                                // Afficher l'écran de résultat
                                let should_close = screen.render(ctx, result_data, &current_hit_window);
                                if should_close {
                                    // Demander à la logique de fermer le résultat (retour menu standard)
                                    ui_messages.push(MainToLogic::TransitionToMenu);
                                }
                            }
                        }
                    } else {
                        // Affichage standard du Song Select
                        if let Some(ref mut song_select) = self.song_select_screen {
                             // On récupère maintenant un tuple (Action UI, Données Resultat si clic leaderboard)
                             let (action_opt, result_opt) = song_select.render(
                                ctx, menu_state, view, 
                                self.config.width as f32, self.config.height as f32, 
                                &current_hit_window, hit_window_mode, hit_window_value, 
                                btn_tex, btn_sel_tex, diff_tex, diff_sel_tex, 
                                song_selected_color, difficulty_selected_color
                             );
                             
                             if let Some(action) = action_opt {
                                 ui_messages.push(MainToLogic::Input(KeyAction::UI(action)));
                             }
                             
                             // Si on a cliqué sur un score dans le leaderboard
                             if let Some(result_data) = result_opt {
                                 ui_messages.push(MainToLogic::TransitionToResult(result_data));
                             }
                        }
                    }
                },
                _ => {
                    if matches!(self.current_state, RenderState::InGame(_)) && self.editor_status_text.is_some() {
                        egui::Window::new("Editor / Test Mode").default_width(300.0).anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0]).show(ctx, |ui| {
                            ui.heading("Live Settings");
                            if let Some(status) = &self.editor_status_text { ui.add_space(5.0); ui.label(egui::RichText::new(status).color(egui::Color32::YELLOW).strong()); }
                            if let Some(values) = &self.editor_values_text { ui.add_space(2.0); ui.label(egui::RichText::new(values).color(egui::Color32::CYAN).monospace()); ui.add_space(5.0); }
                            ui.separator();
                            ui.label("Select: W(Note) X(Rec) L(Lane) C(Cmb) V(Scr) B(Acc) N(Flash) J(List) K(Bar)");
                            ui.label("S: Save Config");
                        });
                    }
                }
            }

            if should_show_settings { 
                 egui::SidePanel::left("settings_panel").resizable(false).default_width(250.0).show(ctx, |ui| { 
                    ui.heading("Settings"); 
                    ui.separator();
                    ui.label("Audio"); 
                    if ui.add(egui::Slider::new(&mut master_volume, 0.0..=1.0).text("Volume")).changed() { settings_changed_this_frame = true; }
                    ui.separator();
                    ui.label("Gameplay");
                    if ui.add(egui::Slider::new(&mut scroll_speed, 100.0..=2000.0).text("Speed")).changed() { settings_changed_this_frame = true; }
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label("Hit Window");
                    ui.horizontal(|ui| {
                        if ui.radio_value(&mut hit_window_mode, crate::models::settings::HitWindowMode::OsuOD, "osu! OD").changed() { settings_changed_this_frame = true; }
                        if ui.radio_value(&mut hit_window_mode, crate::models::settings::HitWindowMode::EtternaJudge, "Etterna Judge").changed() { settings_changed_this_frame = true; }
                    });
                    match hit_window_mode {
                        crate::models::settings::HitWindowMode::OsuOD => { if ui.add(egui::Slider::new(&mut hit_window_value, 0.0..=10.0).text("OD")).changed() { settings_changed_this_frame = true; } }
                        crate::models::settings::HitWindowMode::EtternaJudge => {
                            let mut judge_f64 = hit_window_value.round().max(1.0).min(9.0);
                            if ui.add(egui::Slider::new(&mut judge_f64, 1.0..=9.0).text("Judge Level")).changed() { hit_window_value = judge_f64; settings_changed_this_frame = true; }
                        }
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    if ui.button("Key Bindings").clicked() { settings_show_keybindings = true; }
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("Press ESC or Ctrl+O to Close").weak());
                });
            }
            
             if settings_show_keybindings { 
                 egui::Window::new("Key Bindings").show(ctx, |ui| { 
                     if let Some(col) = remapping_column { 
                         ui.label(format!("Press key for Col {}...", col + 1)); 
                         if let Some(k) = &captured_key_str { 
                             let binds = self.settings.keybinds.entry(NUM_COLUMNS.to_string()).or_insert_with(Vec::new); 
                             while binds.len() <= col { binds.push("None".to_string()); } 
                             binds[col] = k.clone(); 
                             self.settings.save(); 
                             remapping_column = None; 
                             settings_changed_this_frame = true; 
                         } 
                         if ui.button("Cancel").clicked() { remapping_column = None; } 
                     } else { 
                         egui::Grid::new("kb_grid").striped(true).show(ui, |ui| { 
                             for (c, k) in keybinding_rows.iter() { 
                                 ui.label(format!("Col {}", c + 1)); 
                                 if ui.button(k).clicked() { remapping_column = Some(*c); } 
                                 ui.end_row(); 
                             } 
                         }); 
                     } 
                     if ui.button("Done").clicked() { settings_show_keybindings = false; remapping_column = None; } 
                 }); 
             }
        });

        if aspect_ratio_mode != self.settings.aspect_ratio_mode {
            self.settings.aspect_ratio_mode = aspect_ratio_mode;
            self.update_pixel_system_ratio();
            settings_changed_this_frame = true;
        }
        
        if settings_changed_this_frame {
             self.settings.master_volume = master_volume;
             self.settings.scroll_speed = scroll_speed;
             self.settings.hit_window_mode = hit_window_mode;
             self.settings.hit_window_value = hit_window_value;
             self.settings.save();
             
             ui_messages.push(MainToLogic::SettingsChanged);
        }
        
        self.settings.show_keybindings = settings_show_keybindings;
        self.settings.remapping_column = remapping_column;
        self.settings.is_open = should_show_settings;
        
        self.egui_ctx = egui_ctx;
        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        
        (tris, full_output.textures_delta, settings_changed_this_frame, ui_messages)
    }

    pub fn render_ui_layer(
        &mut self,
        view: &TextureView,
        tris: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
        window: &winit::window::Window,
    ) -> CommandBuffer {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Egui Encoder"),
            });
        let sd = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        for (id, img) in &textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, img);
        }
        self.egui_renderer
            .update_buffers(&self.device, &self.queue, &mut encoder, tris, &sd);
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let rpass_static = unsafe {
                std::mem::transmute::<&mut wgpu::RenderPass<'_>, &mut wgpu::RenderPass<'static>>(
                    &mut rpass,
                )
            };
            self.egui_renderer.render(rpass_static, tris, &sd);
        }
        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        encoder.finish()
    }
}