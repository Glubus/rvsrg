use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HitWindowMode {
    OsuOD,
    EtternaJudge,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AspectRatioMode {
    Auto,
    Ratio16_9,
    Ratio4_3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    pub master_volume: f32,
    pub scroll_speed: f64,
    pub hit_window_mode: HitWindowMode,
    pub hit_window_value: f64,
    pub aspect_ratio_mode: AspectRatioMode,
    pub current_skin: String,

    // CORRECTION : Les clés TOML doivent être des Strings
    pub keybinds: HashMap<String, Vec<String>>,

    #[serde(skip)]
    pub is_open: bool,
    #[serde(skip)]
    pub show_keybindings: bool,
    #[serde(skip)]
    pub remapping_column: Option<usize>,
}

impl SettingsState {
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
        }
    }

    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string("settings.toml") {
            if let Ok(mut settings) = toml::from_str::<SettingsState>(&content) {
                settings.is_open = false;
                settings.show_keybindings = false;
                settings.remapping_column = None;

                if settings.keybinds.is_empty() {
                    settings.keybinds = Self::default_keybinds();
                }
                return settings;
            } else {
                eprintln!("Failed to parse settings.toml, using defaults.");
            }
        }
        Self::new()
    }

    pub fn save(&self) {
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write("settings.toml", content) {
                    eprintln!("Failed to write settings.toml: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize settings: {}", e),
        }
    }

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
