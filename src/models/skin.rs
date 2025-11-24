use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPosition {
    #[serde(default)]
    pub x: Option<f32>,
    #[serde(default)]
    pub y: Option<f32>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPositions {
    #[serde(default)]
    pub playfield: Option<UIPosition>,
    #[serde(default)]
    pub combo: Option<UIPosition>,
    #[serde(default)]
    pub hit_bar: Option<UIPosition>,
    #[serde(default)]
    pub score: Option<UIPosition>,
    #[serde(default)]
    pub accuracy: Option<UIPosition>,
    #[serde(default)]
    pub judgements: Option<UIPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinConfig {
    pub skin: SkinInfo,
    pub images: ImagePaths,
    #[serde(default)]
    pub colors: Option<ColorConfig>,
    #[serde(default)]
    pub keys: Option<KeyConfig>,
    #[serde(default)]
    pub ui_positions: Option<UIPositions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    #[serde(default)]
    pub column_0: Option<Vec<String>>,
    #[serde(default)]
    pub column_1: Option<Vec<String>>,
    #[serde(default)]
    pub column_2: Option<Vec<String>>,
    #[serde(default)]
    pub column_3: Option<Vec<String>>,
    #[serde(default)]
    pub column_4: Option<Vec<String>>,
    #[serde(default)]
    pub column_5: Option<Vec<String>>,
    #[serde(default)]
    pub column_6: Option<Vec<String>>,
    #[serde(default)]
    pub column_7: Option<Vec<String>>,
    #[serde(default)]
    pub column_8: Option<Vec<String>>,
    #[serde(default)]
    pub column_9: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub font: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePaths {
    #[serde(default)]
    pub receptor: Option<String>,
    #[serde(default)]
    pub receptor_0: Option<String>,
    #[serde(default)]
    pub receptor_1: Option<String>,
    #[serde(default)]
    pub receptor_2: Option<String>,
    #[serde(default)]
    pub receptor_3: Option<String>,
    #[serde(default)]
    pub receptor_4: Option<String>,
    #[serde(default)]
    pub receptor_5: Option<String>,
    #[serde(default)]
    pub receptor_6: Option<String>,
    #[serde(default)]
    pub receptor_7: Option<String>,
    #[serde(default)]
    pub receptor_8: Option<String>,
    #[serde(default)]
    pub receptor_9: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub note_0: Option<String>,
    #[serde(default)]
    pub note_1: Option<String>,
    #[serde(default)]
    pub note_2: Option<String>,
    #[serde(default)]
    pub note_3: Option<String>,
    #[serde(default)]
    pub note_4: Option<String>,
    #[serde(default)]
    pub note_5: Option<String>,
    #[serde(default)]
    pub note_6: Option<String>,
    #[serde(default)]
    pub note_7: Option<String>,
    #[serde(default)]
    pub note_8: Option<String>,
    #[serde(default)]
    pub note_9: Option<String>,
    #[serde(default)]
    pub miss_note: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    #[serde(default = "default_receptor_color")]
    pub receptor_color: [f32; 4],
    #[serde(default = "default_note_color")]
    pub note_color: [f32; 4],
    #[serde(default = "default_marv_color")]
    pub marv: [f32; 4],
    #[serde(default = "default_perfect_color")]
    pub perfect: [f32; 4],
    #[serde(default = "default_great_color")]
    pub great: [f32; 4],
    #[serde(default = "default_good_color")]
    pub good: [f32; 4],
    #[serde(default = "default_bad_color")]
    pub bad: [f32; 4],
    #[serde(default = "default_miss_color")]
    pub miss: [f32; 4],
    #[serde(default = "default_ghost_tap_color")]
    pub ghost_tap: [f32; 4],
}

fn default_receptor_color() -> [f32; 4] {
    [0.0, 0.0, 1.0, 1.0]
}

fn default_note_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

fn default_marv_color() -> [f32; 4] {
    [0.0, 1.0, 1.0, 1.0]
}

fn default_perfect_color() -> [f32; 4] {
    [1.0, 1.0, 0.0, 1.0]
}

fn default_great_color() -> [f32; 4] {
    [0.0, 1.0, 0.0, 1.0]
}

fn default_good_color() -> [f32; 4] {
    [0.0, 0.0, 0.5, 1.0]
}

fn default_bad_color() -> [f32; 4] {
    [1.0, 0.41, 0.71, 1.0]
}

fn default_miss_color() -> [f32; 4] {
    [1.0, 0.0, 0.0, 1.0]
}

fn default_ghost_tap_color() -> [f32; 4] {
    [0.5, 0.5, 0.5, 1.0]
}

pub struct Skin {
    pub config: SkinConfig,
    pub base_path: PathBuf,
    pub key_to_column: HashMap<String, usize>,
}

impl Skin {
    pub fn load(skin_path: &Path) -> Result<Self, String> {
        let toml_path = skin_path.join("skin.toml");

        if !toml_path.exists() {
            return Err(format!("skin.toml not found in {:?}", skin_path));
        }

        let toml_content = fs::read_to_string(&toml_path)
            .map_err(|e| format!("Failed to read skin.toml: {}", e))?;

        let config: SkinConfig = toml::from_str(&toml_content)
            .map_err(|e| format!("Failed to parse skin.toml: {}", e))?;

        let key_to_column = build_keymap(&config, usize::MAX);

        Ok(Self {
            config,
            base_path: skin_path.to_path_buf(),
            key_to_column,
        })
    }

    pub fn load_default(num_columns: usize) -> Result<Self, String> {
        let default_path = Path::new("skins/default");
        let toml_name = format!("skin_{}k.toml", num_columns);
        let toml_path = default_path.join(&toml_name);

        if !toml_path.exists() {
            return Err(format!(
                "skin_{}k.toml not found in {:?}",
                num_columns, default_path
            ));
        }

        let toml_content = fs::read_to_string(&toml_path)
            .map_err(|e| format!("Failed to read {}: {}", toml_name, e))?;

        let config: SkinConfig = toml::from_str(&toml_content)
            .map_err(|e| format!("Failed to parse {}: {}", toml_name, e))?;

        let key_to_column = build_keymap(&config, num_columns);

        Ok(Self {
            config,
            base_path: default_path.to_path_buf(),
            key_to_column,
        })
    }

    pub fn get_column_for_key(&self, key_name: &str) -> Option<usize> {
        self.key_to_column.get(key_name).copied()
    }

    pub fn get_image_path(&self, image_name: &str) -> PathBuf {
        self.base_path.join(image_name)
    }

    pub fn get_receptor_path(&self, column: usize) -> Option<PathBuf> {
        let image_name = match column {
            0 => self.config.images.receptor_0.as_ref(),
            1 => self.config.images.receptor_1.as_ref(),
            2 => self.config.images.receptor_2.as_ref(),
            3 => self.config.images.receptor_3.as_ref(),
            4 => self.config.images.receptor_4.as_ref(),
            5 => self.config.images.receptor_5.as_ref(),
            6 => self.config.images.receptor_6.as_ref(),
            7 => self.config.images.receptor_7.as_ref(),
            8 => self.config.images.receptor_8.as_ref(),
            9 => self.config.images.receptor_9.as_ref(),
            _ => None,
        };

        image_name
            .or_else(|| self.config.images.receptor.as_ref())
            .map(|name| self.get_image_path(name))
    }

    pub fn get_note_path(&self, column: usize) -> Option<PathBuf> {
        let image_name = match column {
            0 => self.config.images.note_0.as_ref(),
            1 => self.config.images.note_1.as_ref(),
            2 => self.config.images.note_2.as_ref(),
            3 => self.config.images.note_3.as_ref(),
            4 => self.config.images.note_4.as_ref(),
            5 => self.config.images.note_5.as_ref(),
            6 => self.config.images.note_6.as_ref(),
            7 => self.config.images.note_7.as_ref(),
            8 => self.config.images.note_8.as_ref(),
            9 => self.config.images.note_9.as_ref(),
            _ => None,
        };

        image_name
            .or_else(|| self.config.images.note.as_ref())
            .map(|name| self.get_image_path(name))
    }

    pub fn get_miss_note_path(&self) -> Option<PathBuf> {
        self.config
            .images
            .miss_note
            .as_ref()
            .map(|name| self.get_image_path(name))
    }

    pub fn get_background_path(&self) -> Option<PathBuf> {
        self.config
            .images
            .background
            .as_ref()
            .map(|name| self.get_image_path(name))
    }

    pub fn get_receptor_color(&self) -> [f32; 4] {
        self.config
            .colors
            .as_ref()
            .map(|c| c.receptor_color)
            .unwrap_or([0.0, 0.0, 1.0, 1.0])
    }

    pub fn get_note_color(&self) -> [f32; 4] {
        self.config
            .colors
            .as_ref()
            .map(|c| c.note_color)
            .unwrap_or([1.0, 1.0, 1.0, 1.0])
    }

    pub fn get_judgement_colors(&self) -> crate::models::stats::JudgementColors {
        if let Some(colors) = &self.config.colors {
            crate::models::stats::JudgementColors {
                marv: colors.marv,
                perfect: colors.perfect,
                great: colors.great,
                good: colors.good,
                bad: colors.bad,
                miss: colors.miss,
                ghost_tap: colors.ghost_tap,
            }
        } else {
            crate::models::stats::JudgementColors::new()
        }
    }

    pub fn get_font_path(&self) -> Option<PathBuf> {
        self.config
            .skin
            .font
            .as_ref()
            .map(|font_name| self.get_image_path(font_name))
    }

    pub fn get_ui_positions(&self) -> &Option<UIPositions> {
        &self.config.ui_positions
    }
}

pub fn init_skin_structure() -> Result<(), String> {
    let skins_dir = Path::new("skins");
    let default_dir = skins_dir.join("default");

    if !skins_dir.exists() {
        fs::create_dir_all(&skins_dir)
            .map_err(|e| format!("Failed to create skins directory: {}", e))?;
    }

    if !default_dir.exists() {
        fs::create_dir_all(&default_dir)
            .map_err(|e| format!("Failed to create default skin directory: {}", e))?;
    }

    for num_cols in 4..=10 {
        let toml_name = format!("skin_{}k.toml", num_cols);
        let toml_path = default_dir.join(&toml_name);

        if !toml_path.exists() {
            let toml_content = generate_toml_for_columns(num_cols);
            fs::write(&toml_path, toml_content)
                .map_err(|e| format!("Failed to create {}: {}", toml_name, e))?;
        }
    }

    Ok(())
}

fn generate_toml_for_columns(num_columns: usize) -> String {
    let mut toml = format!(
        r#"# Configuration du skin pour {} colonnes
[skin]
name = "Default {}K"
version = "1.0.0"
author = "RVSRG"

font = "font.ttf"
[images]
receptor = "receptor.png"
"#,
        num_columns, num_columns
    );

    for i in 0..num_columns {
        toml.push_str(&format!("# receptor_{} = \"receptor_col{}.png\"\n", i, i));
    }

    toml.push_str(
        r#"
note = "note.png"
"#,
    );

    for i in 0..num_columns {
        toml.push_str(&format!("# note_{} = \"note_col{}.png\"\n", i, i));
    }

    toml.push_str(
        r#"
miss_note = "miss_note.png"
background = "background.png"

[colors]
receptor_color = [0.0, 0.0, 1.0, 1.0]
note_color = [1.0, 1.0, 1.0, 1.0]
marv = [0.0, 1.0, 1.0, 1.0]
perfect = [1.0, 1.0, 0.0, 1.0]
great = [0.0, 1.0, 0.0, 1.0]
good = [0.0, 0.0, 0.5, 1.0]
bad = [1.0, 0.41, 0.71, 1.0]
miss = [1.0, 0.0, 0.0, 1.0]
ghost_tap = [0.5, 0.5, 0.5, 1.0]

[keys]
"#,
    );

    let default_keys = match num_columns {
        4 => vec!["KeyD", "KeyF", "KeyJ", "KeyK"],
        5 => vec!["KeyD", "KeyF", "Space", "KeyJ", "KeyK"],
        6 => vec!["KeyS", "KeyD", "KeyF", "KeyJ", "KeyK", "KeyL"],
        7 => vec!["KeyA", "KeyS", "KeyD", "KeyF", "KeyJ", "KeyK", "KeyL"],
        8 => vec![
            "KeyA", "KeyS", "KeyD", "KeyF", "KeyH", "KeyJ", "KeyK", "KeyL",
        ],
        9 => vec![
            "KeyQ", "KeyA", "KeyS", "KeyD", "KeyF", "KeyH", "KeyJ", "KeyK", "KeyL",
        ],
        10 => vec![
            "KeyQ", "KeyA", "KeyS", "KeyD", "KeyF", "KeyH", "KeyJ", "KeyK", "KeyL", "KeyP",
        ],
        _ => vec!["KeyD", "KeyF", "KeyJ", "KeyK"],
    };

    for (i, key) in default_keys.iter().enumerate().take(num_columns) {
        toml.push_str(&format!("column_{} = [\"{}\"]\n", i, key));
    }

    toml.push_str(
        r#"
[ui_positions]
"#,
    );

    toml
}

fn build_keymap(config: &SkinConfig, max_columns: usize) -> HashMap<String, usize> {
    let mut key_to_column = HashMap::new();
    if let Some(keys) = &config.keys {
        let column_keys = [
            &keys.column_0,
            &keys.column_1,
            &keys.column_2,
            &keys.column_3,
            &keys.column_4,
            &keys.column_5,
            &keys.column_6,
            &keys.column_7,
            &keys.column_8,
            &keys.column_9,
        ];

        for (col_idx, col_keys_opt) in column_keys.iter().enumerate() {
            if col_idx >= max_columns {
                break;
            }
            if let Some(col_keys) = col_keys_opt {
                for key in col_keys {
                    key_to_column.insert(key.clone(), col_idx);
                }
            }
        }
    }
    key_to_column
}

