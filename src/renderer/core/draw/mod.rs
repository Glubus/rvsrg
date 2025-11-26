use crate::renderer::Renderer;
use std::time::Instant;

pub mod game;
pub mod ui;

impl Renderer {
    // Modifié pour retourner les infos UI
    pub fn render(&mut self, window: &winit::window::Window) -> Result<(bool, Vec<crate::shared::messages::MainToLogic>), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let now = Instant::now();
        let delta_secs = now.duration_since(self.last_fps_update).as_secs_f64();
        if delta_secs > 0.0 {
            let instantaneous = 1.0 / delta_secs;
            const SMOOTHING: f64 = 0.15;
            self.fps = if self.fps == 0.0 {
                instantaneous
            } else {
                self.fps * (1.0 - SMOOTHING) + instantaneous * SMOOTHING
            };
        }
        self.last_fps_update = now;

        // On récupère (tris, textures, settings_changed, messages)
        let (ui_tris, ui_textures, settings_changed, ui_messages) = self.update_ui(window, &view);

        let game_cmd = self.render_game_layer(&view)?;
        let ui_cmd = self.render_ui_layer(&view, &ui_tris, &ui_textures, window);

        self.queue.submit([game_cmd, ui_cmd]);

        output.present();
        
        // On retourne les flags importants
        Ok((settings_changed, ui_messages))
    }
}