use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinGeneral {
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub font: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinColors {
    #[serde(default = "default_white")]
    pub receptor_color: [f32; 4],
    #[serde(default = "default_white")]
    pub note_color: [f32; 4],
    #[serde(default = "default_selected")]
    pub selected_color: [f32; 4],
    #[serde(default = "default_diff_selected")]
    pub difficulty_selected_color: [f32; 4],
    #[serde(default = "default_cyan")]
    pub marv: [f32; 4],
    #[serde(default = "default_yellow")]
    pub perfect: [f32; 4],
    #[serde(default = "default_green")]
    pub great: [f32; 4],
    #[serde(default = "default_blue")]
    pub good: [f32; 4],
    #[serde(default = "default_pink")]
    pub bad: [f32; 4],
    #[serde(default = "default_red")]
    pub miss: [f32; 4],
    #[serde(default = "default_gray")]
    pub ghost_tap: [f32; 4],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UIElementPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinUserConfig {
    #[serde(default = "default_note_size")]
    pub note_width_px: f32,
    #[serde(default = "default_note_size")]
    pub note_height_px: f32,
    #[serde(default = "default_note_size")]
    pub receptor_width_px: f32,
    #[serde(default = "default_note_size")]
    pub receptor_height_px: f32,
    pub column_width_px: f32,
    #[serde(default)]
    pub receptor_spacing_px: f32,
    #[serde(default = "default_text_size")]
    pub combo_text_size: f32,
    #[serde(default = "default_text_size")]
    pub score_text_size: f32,
    #[serde(default = "default_text_size")]
    pub accuracy_text_size: f32,
    #[serde(default = "default_text_size")]
    pub judgement_text_size: f32,
    #[serde(default = "default_hitbar_height")]
    pub hit_bar_height_px: f32,
    #[serde(default)]
    pub playfield_pos: Option<UIElementPos>,
    #[serde(default)]
    pub combo_pos: Option<UIElementPos>,
    #[serde(default)]
    pub score_pos: Option<UIElementPos>,
    #[serde(default)]
    pub accuracy_pos: Option<UIElementPos>,
    #[serde(default)]
    pub judgement_pos: Option<UIElementPos>,
    #[serde(default)]
    pub judgement_flash_pos: Option<UIElementPos>,
    #[serde(default)]
    pub hit_bar_pos: Option<UIElementPos>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinKeyMode {
    pub receptor_images: Vec<String>,
    #[serde(default)]
    pub receptor_pressed_images: Vec<String>,
    pub note_images: Vec<String>,
    pub hit_bar_pos: Option<f32>,
}

pub struct Skin {
    pub base_path: PathBuf,
    pub general: SkinGeneral,
    pub colors: SkinColors,
    pub config: SkinUserConfig,
    pub key_modes: HashMap<usize, SkinKeyMode>,
    pub background: Option<PathBuf>,
    pub miss_note: Option<PathBuf>,
    pub song_button: Option<PathBuf>,
    pub song_button_selected: Option<PathBuf>,
    pub difficulty_button: Option<PathBuf>,
    pub difficulty_button_selected: Option<PathBuf>,
}

impl Skin {
    pub fn load(skin_name: &str) -> Result<Self, String> {
        let base_path = Path::new("skins").join(skin_name);
        if !base_path.exists() {
            if skin_name == "default" {
                eprintln!("Default skin folder missing, recreating structure...");
                let _ = init_skin_structure();
                if !base_path.exists() {
                    return Err(format!(
                        "Failed to recreate default skin at {:?}",
                        base_path
                    ));
                }
            } else {
                return Err(format!("Skin folder not found: {:?}", base_path));
            }
        }
        let general: SkinGeneral = load_toml(&base_path.join("general.toml"))?;
        let colors: SkinColors = load_toml(&base_path.join("colors.toml"))?;
        let config: SkinUserConfig = load_toml(&base_path.join("conf.toml"))?;
        Ok(Self {
            base_path: base_path.clone(),
            general,
            colors,
            config,
            key_modes: HashMap::new(),
            background: check_file(&base_path, "background.png"),
            miss_note: check_file(&base_path, "miss_note.png"),
            song_button: check_file(&base_path, "song_button.png"),
            song_button_selected: check_file(&base_path, "song_button_selected.png"),
            difficulty_button: check_file(&base_path, "difficulty_button.png"),
            difficulty_button_selected: check_file(&base_path, "difficulty_button_selected.png"),
        })
    }
    pub fn save_user_config(&self) -> Result<(), String> {
        let path = self.base_path.join("conf.toml");
        let content = toml::to_string_pretty(&self.config).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }
    pub fn load_key_mode(&mut self, key_count: usize) {
        if self.key_modes.contains_key(&key_count) {
            return;
        }
        let path = self.base_path.join(format!("{}k.toml", key_count));
        if path.exists() {
            if let Ok(mode) = load_toml::<SkinKeyMode>(&path) {
                self.key_modes.insert(key_count, mode);
            } else {
                eprintln!("Failed to parse {}k.toml", key_count);
            }
        }
    }
    pub fn get_receptor_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        self.key_modes
            .get(&key_count)
            .and_then(|m| get_image_from_list(&m.receptor_images, col))
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "receptor.png"))
    }
    pub fn get_receptor_pressed_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        self.key_modes
            .get(&key_count)
            .and_then(|m| get_image_from_list(&m.receptor_pressed_images, col))
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "receptor_pressed.png"))
    }
    pub fn get_note_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        self.key_modes
            .get(&key_count)
            .and_then(|m| get_image_from_list(&m.note_images, col))
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "note.png"))
    }
    pub fn get_font_path(&self) -> Option<PathBuf> {
        self.general.font.as_ref().map(|f| self.base_path.join(f))
    }
}

fn load_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    toml::from_str(&content).map_err(|e| e.to_string())
}
fn check_file(base: &Path, name: &str) -> Option<PathBuf> {
    let p = base.join(name);
    if p.exists() { Some(p) } else { None }
}
fn get_image_from_list(list: &[String], idx: usize) -> Option<&String> {
    if list.is_empty() {
        return None;
    }
    if idx < list.len() {
        Some(&list[idx])
    } else {
        Some(&list[0])
    }
}

fn default_white() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_selected() -> [f32; 4] {
    [1.0, 0.0, 0.0, 1.0]
}
fn default_diff_selected() -> [f32; 4] {
    [1.0, 1.0, 0.0, 1.0]
}
fn default_cyan() -> [f32; 4] {
    [0.0, 1.0, 1.0, 1.0]
}
fn default_yellow() -> [f32; 4] {
    [1.0, 1.0, 0.0, 1.0]
}
fn default_green() -> [f32; 4] {
    [0.0, 1.0, 0.0, 1.0]
}
fn default_blue() -> [f32; 4] {
    [0.0, 0.0, 0.5, 1.0]
}
fn default_pink() -> [f32; 4] {
    [1.0, 0.41, 0.71, 1.0]
}
fn default_red() -> [f32; 4] {
    [1.0, 0.0, 0.0, 1.0]
}
fn default_gray() -> [f32; 4] {
    [0.5, 0.5, 0.5, 1.0]
}
fn default_note_size() -> f32 {
    90.0
}
fn default_text_size() -> f32 {
    20.0
}
fn default_hitbar_height() -> f32 {
    20.0
}

pub fn init_skin_structure() -> Result<(), String> {
    let skins_dir = Path::new("skins");
    let default_dir = skins_dir.join("default");
    if !skins_dir.exists() {
        fs::create_dir_all(&skins_dir).map_err(|e| e.to_string())?;
    }
    if !default_dir.exists() {
        fs::create_dir_all(&default_dir).map_err(|e| e.to_string())?;
    }
    if !default_dir.join("general.toml").exists() {
        fs::write(
            default_dir.join("general.toml"),
            "name=\"Default Skin\"\nversion=\"1.0\"\nauthor=\"System\"\nfont=\"font.ttf\"\n",
        )
        .map_err(|e| e.to_string())?;
    }
    if !default_dir.join("colors.toml").exists() {
        fs::write(default_dir.join("colors.toml"), "receptor_color=[0.0,0.0,1.0,1.0]\nnote_color=[1.0,1.0,1.0,1.0]\nselected_color=[1.0,0.0,0.0,1.0]\ndifficulty_selected_color=[1.0,1.0,0.0,1.0]\nmarv=[0.0,1.0,1.0,1.0]\nperfect=[1.0,1.0,0.0,1.0]\ngreat=[0.0,1.0,0.0,1.0]\ngood=[0.0,0.0,0.5,1.0]\nbad=[1.0,0.41,0.71,1.0]\nmiss=[1.0,0.0,0.0,1.0]\nghost_tap=[0.5,0.5,0.5,1.0]\n").map_err(|e| e.to_string())?;
    }
    if !default_dir.join("conf.toml").exists() {
        fs::write(default_dir.join("conf.toml"), "column_width_px=100.0\nreceptor_spacing_px=0.0\nnote_width_px=90.0\nnote_height_px=90.0\nreceptor_width_px=90.0\nreceptor_height_px=90.0\ncombo_text_size=48.0\nscore_text_size=24.0\naccuracy_text_size=20.0\njudgement_text_size=16.0\nhit_bar_height_px=20.0\n").map_err(|e| e.to_string())?;
    }
    for k in 4..=10 {
        let path = default_dir.join(format!("{}k.toml", k));
        if !path.exists() {
            fs::write(&path, "receptor_images=[\"receptor.png\"]\nreceptor_pressed_images=[\"receptor_pressed.png\"]\nnote_images=[\"note.png\"]\n").map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
