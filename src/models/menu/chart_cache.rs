//! Cache de chart pour le menu.

use crate::models::engine::NoteData;
use std::path::PathBuf;

/// Cache de la chart actuellement sélectionnée.
/// Permet de pré-charger la map et de l'utiliser pour:
/// - Le gameplay (pas besoin de recharger)
/// - Le recalcul des replays du leaderboard
#[derive(Clone, Debug)]
pub struct ChartCache {
    /// Hash de la beatmap cachée.
    pub beatmap_hash: String,
    /// Notes de la chart.
    pub chart: Vec<NoteData>,
    /// Chemin vers le fichier audio.
    pub audio_path: PathBuf,
    /// Chemin vers le fichier .osu.
    pub map_path: PathBuf,
}


