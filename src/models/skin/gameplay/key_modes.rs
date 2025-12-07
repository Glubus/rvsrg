//! Key mode specific configuration (4K, 5K, 6K, 7K, etc.)

use super::notes::{BurstConfig, HoldConfig, MineConfig, NoteColumnConfig};
use super::receptors::ReceptorColumnConfig;
use crate::models::skin::common::Vec2Conf;
use serde::{Deserialize, Serialize};

/// Configuration for a specific key mode (e.g., 4K, 7K)
/// Each column can have its own note/receptor images and colors
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyModeConfig {
    /// Per-column receptor configurations
    /// Index 0 = leftmost column
    #[serde(default)]
    pub receptors: Vec<ReceptorColumnConfig>,

    /// Per-column note configurations
    #[serde(default)]
    pub notes: Vec<NoteColumnConfig>,

    /// Per-column hold configurations (optional, falls back to defaults)
    #[serde(default)]
    pub holds: Vec<HoldConfig>,

    /// Per-column burst configurations (optional)
    #[serde(default)]
    pub bursts: Vec<BurstConfig>,

    /// Per-column mine configurations (optional)
    #[serde(default)]
    pub mines: Vec<MineConfig>,

    /// Override column width for this keymode
    #[serde(default)]
    pub column_width: Option<f32>,

    /// Override playfield position for this keymode
    #[serde(default)]
    pub playfield_position: Option<Vec2Conf>,

    /// Override hit bar position
    #[serde(default)]
    pub hit_bar_pos: Option<f32>,
}

impl KeyModeConfig {
    /// Get receptor config for a specific column, or None if not defined
    pub fn get_receptor(&self, col: usize) -> Option<&ReceptorColumnConfig> {
        if col < self.receptors.len() {
            Some(&self.receptors[col])
        } else if !self.receptors.is_empty() {
            // Wrap around to first if only one defined (symmetric skin)
            Some(&self.receptors[0])
        } else {
            None
        }
    }

    /// Get note config for a specific column
    pub fn get_note(&self, col: usize) -> Option<&NoteColumnConfig> {
        if col < self.notes.len() {
            Some(&self.notes[col])
        } else if !self.notes.is_empty() {
            Some(&self.notes[0])
        } else {
            None
        }
    }

    /// Get hold config for a specific column
    pub fn get_hold(&self, col: usize) -> Option<&HoldConfig> {
        if col < self.holds.len() {
            Some(&self.holds[col])
        } else if !self.holds.is_empty() {
            Some(&self.holds[0])
        } else {
            None
        }
    }

    /// Get burst config for a specific column
    pub fn get_burst(&self, col: usize) -> Option<&BurstConfig> {
        if col < self.bursts.len() {
            Some(&self.bursts[col])
        } else if !self.bursts.is_empty() {
            Some(&self.bursts[0])
        } else {
            None
        }
    }

    /// Get mine config for a specific column
    pub fn get_mine(&self, col: usize) -> Option<&MineConfig> {
        if col < self.mines.len() {
            Some(&self.mines[col])
        } else if !self.mines.is_empty() {
            Some(&self.mines[0])
        } else {
            None
        }
    }
}
