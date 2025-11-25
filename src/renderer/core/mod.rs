use crate::models::engine::{GameEngine, PixelSystem};
use crate::models::menu::MenuState;
use crate::models::settings::SettingsState;
use crate::models::skin::Skin;
use crate::views::components::menu::result_screen::ResultScreen;
use crate::views::components::menu::song_select::SongSelectScreen;
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, ScoreDisplay,
};
use crate::views::gameplay::GameplayView;
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use wgpu_text::TextBrush;

pub mod control;
pub mod draw;
pub mod init;

pub struct Renderer {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    pub(crate) note_bind_groups: Vec<wgpu::BindGroup>,
    pub(crate) receptor_bind_groups: Vec<wgpu::BindGroup>,
    pub(crate) receptor_pressed_bind_groups: Vec<wgpu::BindGroup>,
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) receptor_buffer: wgpu::Buffer,
    pub engine: GameEngine,
    pub(crate) text_brush: TextBrush,
    pub(crate) last_fps_update: Instant,
    pub(crate) fps: f64,
    pub(crate) pixel_system: PixelSystem,
    pub skin: Skin,
    pub(crate) gameplay_view: GameplayView,
    pub(crate) score_display: ScoreDisplay,
    pub(crate) accuracy_panel: AccuracyDisplay,
    pub(crate) judgements_panel: JudgementPanel,
    pub(crate) combo_display: ComboDisplay,
    pub(crate) judgement_flash: JudgementFlash,
    pub(crate) hit_bar: HitBarDisplay,
    pub menu_state: Arc<Mutex<MenuState>>,
    pub(crate) background_texture: Option<wgpu::Texture>,
    pub(crate) background_bind_group: Option<wgpu::BindGroup>,
    pub(crate) background_pipeline: Option<wgpu::RenderPipeline>,
    pub(crate) background_sampler: wgpu::Sampler,
    pub(crate) current_background_path: Option<String>,
    pub(crate) quad_pipeline: wgpu::RenderPipeline,
    pub(crate) quad_buffer: wgpu::Buffer,
    pub(crate) egui_ctx: egui::Context,
    pub(crate) egui_state: EguiState,
    pub(crate) egui_renderer: EguiRenderer,
    pub settings: SettingsState,
    pub editor_status_text: Option<String>,
    pub editor_values_text: Option<String>,
    pub(crate) leaderboard_scores_loaded: bool,
    pub(crate) current_leaderboard_hash: Option<String>,
    pub(crate) song_select_screen: Option<SongSelectScreen>,
    pub(crate) result_screen: Option<ResultScreen>,
    pub(crate) song_button_texture: Option<egui::TextureHandle>,
    pub(crate) song_button_selected_texture: Option<egui::TextureHandle>,
    pub(crate) difficulty_button_texture: Option<egui::TextureHandle>,
    pub(crate) difficulty_button_selected_texture: Option<egui::TextureHandle>,
}
impl Renderer {}
