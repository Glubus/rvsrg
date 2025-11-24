use crate::models::engine::{GameEngine, PixelSystem};
use crate::models::menu::MenuState;
use crate::models::skin::Skin;
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, ScoreDisplay,
};
use crate::views::components::menu::song_select::SongSelectScreen;
use crate::views::gameplay::GameplayView;
use crate::views::result::ResultView;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use wgpu_text::TextBrush;

use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;
use crate::models::settings::GameSettings;

// Déclaration des sous-modules
mod control;
mod draw;
mod init;

pub struct Renderer {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) note_bind_groups: Vec<wgpu::BindGroup>,
    pub(crate) receptor_bind_groups: Vec<wgpu::BindGroup>,
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) receptor_buffer: wgpu::Buffer,
    pub engine: GameEngine, // Gardé public comme dans l'original
    pub(crate) text_brush: TextBrush,
    pub(crate) frame_count: u64,
    pub(crate) last_fps_update: Instant,
    pub(crate) fps: f64,
    pub(crate) pixel_system: PixelSystem,
    pub skin: Skin, // Gardé public
    pub(crate) gameplay_view: GameplayView,
    pub(crate) result_view: ResultView,
    pub(crate) score_display: ScoreDisplay,
    pub(crate) accuracy_panel: AccuracyDisplay,
    pub(crate) judgements_panel: JudgementPanel,
    pub(crate) combo_display: ComboDisplay,
    pub(crate) judgement_flash: JudgementFlash,
    pub(crate) hit_bar: HitBarDisplay,
    pub menu_state: Arc<Mutex<MenuState>>, // Gardé public
    
    // Background pour le menu
    pub(crate) background_texture: Option<wgpu::Texture>,
    pub(crate) background_bind_group: Option<wgpu::BindGroup>,
    pub(crate) background_pipeline: Option<wgpu::RenderPipeline>,
    pub(crate) background_sampler: wgpu::Sampler,
    pub(crate) current_background_path: Option<String>,
    
    // Pipeline pour les quads colorés (panels, cards)
    pub(crate) quad_pipeline: wgpu::RenderPipeline,
    pub(crate) quad_buffer: wgpu::Buffer,

    pub(crate) egui_ctx: egui::Context,
    pub(crate) egui_state: EguiState,
    pub(crate) egui_renderer: EguiRenderer,
    pub settings: GameSettings, // On stocke l'état ici
    pub(crate) leaderboard_scores_loaded: bool, // Cache pour éviter de charger les scores à chaque frame
    pub(crate) current_leaderboard_hash: Option<String>, // Hash de la map pour laquelle le leaderboard est chargé
    pub(crate) song_select_screen: Option<SongSelectScreen>, // Song select avec egui
}

// Utilitaires privés pour le renderer (fallback skin, etc.)
impl Renderer {
    pub(crate) fn create_fallback_skin() -> Skin {
        Skin {
            config: crate::models::skin::SkinConfig {
                skin: crate::models::skin::SkinInfo {
                    name: "Fallback".to_string(),
                    version: "1.0.0".to_string(),
                    author: "System".to_string(),
                    font: None,
                },
                images: crate::models::skin::ImagePaths {
                    receptor: None, receptor_0: None, receptor_1: None, receptor_2: None,
                    receptor_3: None, receptor_4: None, receptor_5: None, receptor_6: None,
                    receptor_7: None, receptor_8: None, receptor_9: None, note: None,
                    note_0: None, note_1: None, note_2: None, note_3: None, note_4: None,
                    note_5: None, note_6: None, note_7: None, note_8: None, note_9: None,
                    miss_note: None, background: None,
                },
                keys: None,
                colors: None,
                ui_positions: None,
            },
            base_path: PathBuf::from("skins/default"),
            key_to_column: {
                let mut map = std::collections::HashMap::new();
                map.insert("KeyD".to_string(), 0);
                map.insert("KeyF".to_string(), 1);
                map.insert("KeyJ".to_string(), 2);
                map.insert("KeyK".to_string(), 3);
                map
            },
        }
    }
}