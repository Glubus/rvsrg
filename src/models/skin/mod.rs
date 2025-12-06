//! Skin configuration and loading.
//!
//! This module provides a modular skin system with self-contained elements.
//! Each element contains its own position, size, colors, and optional images.
//! Supports multi-keymode (4K, 5K, 6K, 7K) with per-column configurations.

#![allow(dead_code)]

pub mod common;
pub mod editor;
pub mod gameplay;
pub mod general;
pub mod hud;
pub mod menus;

pub use common::{
    /*Color,*/ Vec2Conf, check_file,
    /*get_image_from_list,*/ load_toml, /*resolve_image*/
};
pub use editor::EditorConfig;
pub use gameplay::{
    /*BurstConfig,*/ GameplayDefaults,
    /*HoldConfig,*/ KeyModeConfig, /*MineConfig,*/
    /*NoteColumnConfig,*/
    /*NoteDefaults,*/ NotesDefaults, PlayfieldConfig,
    /*ReceptorColumnConfig,*/ ReceptorDefaults,
};
pub use general::SkinGeneral;
pub use hud::{
    AccuracyConfig, ComboConfig, HitBarConfig, HudConfig, JudgementFlashSet, JudgementLabels,
    NpsConfig, ScoreConfig,
};
pub use menus::{MenusConfig, PanelStyleConfig, SongSelectConfig};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main skin structure containing all configuration
#[derive(Debug, Clone)]
pub struct Skin {
    pub base_path: PathBuf,
    pub general: SkinGeneral,
    pub hud: HudConfig,
    pub gameplay: GameplayDefaults,
    pub menus: MenusConfig,
    pub editor: EditorConfig,

    /// Per-keymode configurations (4K, 5K, 6K, 7K, etc.)
    pub key_modes: HashMap<usize, KeyModeConfig>,

    /// Background image
    pub background: Option<PathBuf>,
}

impl Skin {
    /// Load a skin from the skins directory
    pub fn load(skin_name: &str) -> Result<Self, String> {
        let base_path = Path::new("skins").join(skin_name);
        if !base_path.exists() {
            if skin_name == "default" {
                eprintln!("Default skin folder missing, recreating structure...");
                let _ = init_skin_structure();
            } else {
                return Err(format!("Skin folder not found: {:?}", base_path));
            }
        }

        // Config directory path
        let conf_path = base_path.join("conf");

        // Load general info
        let general: SkinGeneral = load_toml(&conf_path.join("general.toml")).unwrap_or_default();

        // Load HUD config
        let hud: HudConfig = load_toml(&conf_path.join("hud.toml")).unwrap_or_default();

        // Load gameplay defaults
        let gameplay: GameplayDefaults =
            load_toml(&conf_path.join("gameplay.toml")).unwrap_or_default();

        // Load menus config
        let menus: MenusConfig = load_toml(&conf_path.join("menus.toml")).unwrap_or_default();

        // Load editor config (if exists)
        let editor: EditorConfig = load_toml(&conf_path.join("editor.toml")).unwrap_or_default();

        Ok(Self {
            base_path: base_path.clone(),
            general,
            hud,
            gameplay,
            menus,
            editor,
            key_modes: HashMap::new(),
            background: check_file(&base_path, "background.png"),
        })
    }

    /// Save the current configuration
    pub fn save(&self) -> Result<(), String> {
        let conf_path = self.base_path.join("conf");

        // Ensure conf directory exists
        if !conf_path.exists() {
            fs::create_dir_all(&conf_path).map_err(|e| e.to_string())?;
        }

        let hud_path = conf_path.join("hud.toml");
        let hud_content = toml::to_string_pretty(&self.hud).map_err(|e| e.to_string())?;
        fs::write(hud_path, hud_content).map_err(|e| e.to_string())?;

        let gameplay_path = conf_path.join("gameplay.toml");
        let gameplay_content = toml::to_string_pretty(&self.gameplay).map_err(|e| e.to_string())?;
        log::info!("Saving gameplay config: {}", gameplay_path.display());
        log::info!(
            "Gameplay content check (note image): {:?}",
            self.gameplay.notes.note.image
        );
        fs::write(gameplay_path, gameplay_content).map_err(|e| e.to_string())?;

        let menus_path = conf_path.join("menus.toml");
        let menus_content = toml::to_string_pretty(&self.menus).map_err(|e| e.to_string())?;
        fs::write(menus_path, menus_content).map_err(|e| e.to_string())?;

        // Save key modes (4k.toml, 7k.toml, etc.)
        for (key, config) in &self.key_modes {
            let filename = format!("{}k.toml", key);
            let path = conf_path.join(&filename);
            let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
            log::info!("Saving keymode config: {}", path.display());
            fs::write(path, content).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Load key mode specific configuration (conf/4k.toml, conf/7k.toml, etc.)
    pub fn load_key_mode(&mut self, key_count: usize) {
        if self.key_modes.contains_key(&key_count) {
            return;
        }
        let path = self
            .base_path
            .join("conf")
            .join(format!("{}k.toml", key_count));
        if path.exists() {
            if let Ok(mode) = load_toml::<KeyModeConfig>(&path) {
                self.key_modes.insert(key_count, mode);
            }
        }
    }

    /// Get key mode config, loading if necessary
    pub fn get_key_mode(&mut self, key_count: usize) -> Option<&KeyModeConfig> {
        self.load_key_mode(key_count);
        self.key_modes.get(&key_count)
    }

    // ===== Receptor helpers =====

    /// Get receptor image for a specific column in a keymode
    pub fn get_receptor_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        // Try keymode-specific first
        if let Some(km) = self.key_modes.get(&key_count) {
            if let Some(receptor) = km.get_receptor(col) {
                if let Some(ref img) = receptor.image {
                    return Some(self.base_path.join(img));
                }
            }
        }
        // Fall back to defaults
        self.gameplay
            .receptors
            .image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "receptor.png"))
    }

    /// Get receptor pressed image for a specific column
    pub fn get_receptor_pressed_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        if let Some(km) = self.key_modes.get(&key_count) {
            if let Some(receptor) = km.get_receptor(col) {
                if let Some(ref img) = receptor.pressed_image {
                    return Some(self.base_path.join(img));
                }
            }
        }
        self.gameplay
            .receptors
            .pressed_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "receptor_pressed.png"))
    }

    // ===== Note helpers =====

    /// Get note image for a specific column
    pub fn get_note_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        if let Some(km) = self.key_modes.get(&key_count) {
            if let Some(note) = km.get_note(col) {
                if let Some(ref img) = note.image {
                    return Some(self.base_path.join(img));
                }
            }
        }
        self.gameplay
            .notes
            .note
            .image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "note.png"))
    }

    // ===== Hold helpers =====

    /// Get hold body image for a specific column
    pub fn get_hold_body_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        if let Some(km) = self.key_modes.get(&key_count) {
            if let Some(hold) = km.get_hold(col) {
                if let Some(ref img) = hold.body_image {
                    return Some(self.base_path.join(img));
                }
            }
        }
        self.gameplay
            .notes
            .hold
            .body_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "hold_body.png"))
    }

    /// Get hold end image for a specific column
    pub fn get_hold_end_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        if let Some(km) = self.key_modes.get(&key_count) {
            if let Some(hold) = km.get_hold(col) {
                if let Some(ref img) = hold.end_image {
                    return Some(self.base_path.join(img));
                }
            }
        }
        self.gameplay
            .notes
            .hold
            .end_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "hold_end.png"))
            .or_else(|| check_file(&self.base_path, "note.png"))
    }

    // ===== Burst helpers =====

    /// Get burst body image for a specific column
    pub fn get_burst_body_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        if let Some(km) = self.key_modes.get(&key_count) {
            if let Some(burst) = km.get_burst(col) {
                if let Some(ref img) = burst.body_image {
                    return Some(self.base_path.join(img));
                }
            }
        }
        self.gameplay
            .notes
            .burst
            .body_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "burst_body.png"))
    }

    /// Get burst end image for a specific column
    pub fn get_burst_end_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        if let Some(km) = self.key_modes.get(&key_count) {
            if let Some(burst) = km.get_burst(col) {
                if let Some(ref img) = burst.end_image {
                    return Some(self.base_path.join(img));
                }
            }
        }
        self.gameplay
            .notes
            .burst
            .end_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "burst_end.png"))
            .or_else(|| check_file(&self.base_path, "note.png"))
    }

    // ===== Mine helpers =====

    /// Get mine image for a specific column
    pub fn get_mine_image(&self, key_count: usize, col: usize) -> Option<PathBuf> {
        if let Some(km) = self.key_modes.get(&key_count) {
            if let Some(mine) = km.get_mine(col) {
                if let Some(ref img) = mine.image {
                    return Some(self.base_path.join(img));
                }
            }
        }
        self.gameplay
            .notes
            .mine
            .image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "mine.png"))
            .or_else(|| check_file(&self.base_path, "note.png"))
    }

    // ===== Other helpers =====

    /// Get font path if specified
    pub fn get_font_path(&self) -> Option<PathBuf> {
        self.general.font.as_ref().map(|f| self.base_path.join(f))
    }

    /// Get judgement labels from skin
    pub fn get_judgement_labels(&self) -> JudgementLabels {
        self.hud.judgement.labels()
    }

    // ===== Menu image helpers =====

    pub fn get_song_button_image(&self) -> Option<PathBuf> {
        self.menus
            .song_select
            .song_button
            .image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "song_button.png"))
    }

    pub fn get_song_button_selected_image(&self) -> Option<PathBuf> {
        self.menus
            .song_select
            .song_button
            .selected_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "song_button_selected.png"))
    }

    pub fn get_difficulty_button_image(&self) -> Option<PathBuf> {
        self.menus
            .song_select
            .difficulty_button
            .image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "difficulty_button.png"))
    }

    pub fn get_difficulty_button_selected_image(&self) -> Option<PathBuf> {
        self.menus
            .song_select
            .difficulty_button
            .selected_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "difficulty_button_selected.png"))
    }

    pub fn get_beatmap_info_background_image(&self) -> Option<PathBuf> {
        self.menus
            .song_select
            .beatmap_info
            .background_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "beatmap_info_bg.png"))
    }

    pub fn get_search_panel_background_image(&self) -> Option<PathBuf> {
        self.menus
            .song_select
            .search_panel
            .background_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "search_panel_bg.png"))
    }

    pub fn get_search_bar_image(&self) -> Option<PathBuf> {
        self.menus
            .song_select
            .search_bar
            .image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "search_bar.png"))
    }

    pub fn get_leaderboard_background_image(&self) -> Option<PathBuf> {
        self.menus
            .song_select
            .leaderboard
            .background_image
            .as_ref()
            .map(|name| self.base_path.join(name))
            .or_else(|| check_file(&self.base_path, "leaderboard_bg.png"))
    }
}

/// Initialize the default skin structure
pub fn init_skin_structure() -> Result<(), String> {
    let skins_dir = Path::new("skins");
    let default_dir = skins_dir.join("default");
    let conf_dir = default_dir.join("conf");

    if !skins_dir.exists() {
        fs::create_dir_all(skins_dir).map_err(|e| e.to_string())?;
    }
    if !default_dir.exists() {
        fs::create_dir_all(&default_dir).map_err(|e| e.to_string())?;
    }
    if !conf_dir.exists() {
        fs::create_dir_all(&conf_dir).map_err(|e| e.to_string())?;
    }

    // Create general.toml
    if !conf_dir.join("general.toml").exists() {
        let general = SkinGeneral::default();
        let content = toml::to_string_pretty(&general).map_err(|e| e.to_string())?;
        fs::write(conf_dir.join("general.toml"), content).map_err(|e| e.to_string())?;
    }

    // Create hud.toml
    if !conf_dir.join("hud.toml").exists() {
        let hud = HudConfig::default();
        let content = toml::to_string_pretty(&hud).map_err(|e| e.to_string())?;
        fs::write(conf_dir.join("hud.toml"), content).map_err(|e| e.to_string())?;
    }

    // Create gameplay.toml
    if !conf_dir.join("gameplay.toml").exists() {
        let gameplay = GameplayDefaults::default();
        let content = toml::to_string_pretty(&gameplay).map_err(|e| e.to_string())?;
        fs::write(conf_dir.join("gameplay.toml"), content).map_err(|e| e.to_string())?;
    }

    // Create menus.toml
    if !conf_dir.join("menus.toml").exists() {
        let menus = MenusConfig::default();
        let content = toml::to_string_pretty(&menus).map_err(|e| e.to_string())?;
        fs::write(conf_dir.join("menus.toml"), content).map_err(|e| e.to_string())?;
    }

    Ok(())
}
