//! Données de résultat de partie.

use crate::models::replay::{ReplayData, ReplayResult};
use crate::models::stats::HitStats;

/// Données complètes d'un résultat de partie.
#[derive(Clone, Debug, PartialEq)]
pub struct GameResultData {
    pub hit_stats: HitStats,
    /// Inputs purs enregistrés pendant le jeu.
    pub replay_data: ReplayData,
    /// Résultat de la simulation du replay (pour affichage des graphes).
    pub replay_result: ReplayResult,
    pub score: u32,
    pub accuracy: f64,
    pub max_combo: u32,
    pub beatmap_hash: Option<String>,
    pub rate: f64,
    pub judge_text: String,
}


