//! User settings and configuration.
//!
//! This module handles loading/saving settings from `settings.toml`
//! and provides the configuration UI state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Hit window calculation mode.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HitWindowMode {
    /// osu! Overall Difficulty (OD) based timing.
    OsuOD,
    /// Etterna/Quaver judge level based timing.
    EtternaJudge,
}

/// Aspect ratio mode for the playfield.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AspectRatioMode {
    /// Automatic based on window size.
    Auto,
    /// Force 16:9 aspect ratio.
    Ratio16_9,
    /// Force 4:3 aspect ratio.
    Ratio4_3,
}

/// Persistent user settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    /// Master volume (0.0 to 1.0).
    pub master_volume: f32,
    /// Scroll speed in milliseconds.
    pub scroll_speed: f64,
    /// Hit window calculation mode.
    pub hit_window_mode: HitWindowMode,
    /// Hit window value (OD or judge level).
    pub hit_window_value: f64,
    /// Aspect ratio mode.
    pub aspect_ratio_mode: AspectRatioMode,
    /// Current skin name.
    pub current_skin: String,

    /// Keybinds per key count (key = "4", "5", etc.).
    pub keybinds: HashMap<String, Vec<String>>,

    /// Whether settings panel is open (UI state, not persisted).
    #[serde(skip)]
    pub is_open: bool,
    /// Whether keybindings section is shown.
    #[serde(skip)]
    pub show_keybindings: bool,
    /// Column count being remapped (if any).
    #[serde(skip)]
    pub remapping_column: Option<usize>,
    /// Buffer for keys being captured during remapping.
    #[serde(skip)]
    pub remapping_buffer: Vec<String>,
}

impl SettingsState {
    /// Creates default settings.
    pub fn new() -> Self {
        Self {
            master_volume: 0.5,
            scroll_speed: 500.0,
            hit_window_mode: HitWindowMode::OsuOD,
            hit_window_value: 5.0,
            aspect_ratio_mode: AspectRatioMode::Auto,
            current_skin: "default".to_string(),
            keybinds: Self::default_keybinds(),

            is_open: false,
            show_keybindings: false,
            remapping_column: None,
            remapping_buffer: Vec::new(),
        }
    }

    /// Loads settings from `settings.toml`, or returns defaults if not found.
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string("settings.toml") {
            if let Ok(mut settings) = toml::from_str::<SettingsState>(&content) {
                settings.is_open = false;
                settings.show_keybindings = false;
                settings.remapping_column = None;
                settings.remapping_buffer = Vec::new();

                if settings.keybinds.is_empty() {
                    settings.keybinds = Self::default_keybinds();
                }
                return settings;
            }
            eprintln!("Failed to parse settings.toml, using defaults.");
        }
        Self::new()
    }

    /// Saves settings to `settings.toml`.
    pub fn save(&self) {
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write("settings.toml", content) {
                    eprintln!("Failed to write settings.toml: {e}");
                }
            }
            Err(e) => eprintln!("Failed to serialize settings: {e}"),
        }
    }

    /// Resets keybinds to defaults.
    pub fn reset_keybinds(&mut self) {
        self.keybinds = Self::default_keybinds();
    }

    /// Begins capturing keybinds for a specific column count.
    pub fn begin_keybind_capture(&mut self, columns: usize) {
        self.remapping_column = Some(columns);
        self.remapping_buffer.clear();
    }

    /// Cancels the current keybind capture.
    pub fn cancel_keybind_capture(&mut self) {
        self.remapping_column = None;
        self.remapping_buffer.clear();
    }

    /// Adds a key to the capture buffer during remapping.
    pub fn push_keybind_key(&mut self, key_label: String) {
        let Some(target_columns) = self.remapping_column else {
            return;
        };

        if !self.remapping_buffer.contains(&key_label) {
            self.remapping_buffer.push(key_label);
        }

        if self.remapping_buffer.len() >= target_columns {
            let column_key = target_columns.to_string();
            self.keybinds
                .insert(column_key, self.remapping_buffer.clone());
            self.remapping_buffer.clear();
            self.remapping_column = None;
        }
    }

    /// Returns the default keybinds for 4K, 5K, 6K, and 7K.
    fn default_keybinds() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        map.insert(
            "4".to_string(),
            vec![
                "KeyD".to_string(),
                "KeyF".to_string(),
                "KeyJ".to_string(),
                "KeyK".to_string(),
            ],
        );
        map.insert(
            "5".to_string(),
            vec![
                "KeyD".to_string(),
                "KeyF".to_string(),
                "Space".to_string(),
                "KeyJ".to_string(),
                "KeyK".to_string(),
            ],
        );
        map.insert(
            "6".to_string(),
            vec![
                "KeyS".to_string(),
                "KeyD".to_string(),
                "KeyF".to_string(),
                "KeyJ".to_string(),
                "KeyK".to_string(),
                "KeyL".to_string(),
            ],
        );
        map.insert(
            "7".to_string(),
            vec![
                "KeyS".to_string(),
                "KeyD".to_string(),
                "KeyF".to_string(),
                "Space".to_string(),
                "KeyJ".to_string(),
                "KeyK".to_string(),
                "KeyL".to_string(),
            ],
        );
        map
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

