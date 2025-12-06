//! Common types and utilities for skin configuration.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 2D position/size configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Vec2Conf {
    pub x: f32,
    pub y: f32,
}

impl Vec2Conf {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// RGBA color type
pub type Color = [f32; 4];

/// Common color constants
pub mod colors {
    use super::Color;

    pub const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
    pub const BLACK: Color = [0.0, 0.0, 0.0, 1.0];
    pub const RED: Color = [1.0, 0.0, 0.0, 1.0];
    pub const GREEN: Color = [0.0, 1.0, 0.0, 1.0];
    pub const BLUE: Color = [0.0, 0.0, 1.0, 1.0];
    pub const YELLOW: Color = [1.0, 1.0, 0.0, 1.0];
    pub const CYAN: Color = [0.0, 1.0, 1.0, 1.0];
    pub const MAGENTA: Color = [1.0, 0.0, 1.0, 1.0];
    pub const GRAY: Color = [0.5, 0.5, 0.5, 1.0];
    pub const PINK: Color = [1.0, 0.41, 0.71, 1.0];
}

/// Load a TOML file and deserialize it
pub fn load_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    match toml::from_str(&content) {
        Ok(data) => Ok(data),
        Err(e) => {
            log::error!("Failed to parse TOML file {:?}: {}", path, e);
            Err(e.to_string())
        }
    }
}

/// Check if a file exists and return its path
pub fn check_file(base: &Path, name: &str) -> Option<PathBuf> {
    let p = base.join(name);
    if p.exists() { Some(p) } else { None }
}

/// Resolve an optional image name to a full path
pub fn resolve_image(base: &Path, image: &Option<String>, fallback: &str) -> Option<PathBuf> {
    image
        .as_ref()
        .and_then(|name| check_file(base, name))
        .or_else(|| check_file(base, fallback))
}

/// Get an image from a list by index, wrapping around if needed
pub fn get_image_from_list(list: &[String], idx: usize) -> Option<&String> {
    if list.is_empty() {
        return None;
    }
    if idx < list.len() {
        Some(&list[idx])
    } else {
        Some(&list[0])
    }
}
