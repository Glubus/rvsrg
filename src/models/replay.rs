//! Serializable replay structures and replay simulation engine.

use crate::models::engine::hit_window::HitWindow;
use crate::models::engine::NoteData;
use crate::models::settings::HitWindowMode;
use crate::models::stats::{HitStats, Judgement};
use serde::{Deserialize, Serialize};

/// Version actuelle du format de replay.
pub const REPLAY_FORMAT_VERSION: u8 = 2;

/// Input utilisateur pur - press OU release.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayInput {
    /// Temps absolu en ms depuis le début de la map (après rate).
    pub timestamp_ms: f64,
    /// Index de colonne (0-based).
    pub column: usize,
    /// true = press, false = release.
    pub is_press: bool,
}

/// Données de replay minimalistes - seulement les inputs purs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayData {
    /// Version du format pour compatibilité future.
    pub version: u8,
    /// Tous les inputs utilisateur en ordre chronologique.
    pub inputs: Vec<ReplayInput>,
    /// Rate appliqué pendant le jeu.
    pub rate: f64,
    /// Mode de hit window utilisé.
    pub hit_window_mode: HitWindowMode,
    /// Valeur du hit window (OD ou Judge level).
    pub hit_window_value: f64,
}

impl ReplayData {
    pub fn new(rate: f64, hit_window_mode: HitWindowMode, hit_window_value: f64) -> Self {
        Self {
            version: REPLAY_FORMAT_VERSION,
            inputs: Vec::new(),
            rate,
            hit_window_mode,
            hit_window_value,
        }
    }

    /// Ajoute un input (press ou release).
    pub fn add_input(&mut self, timestamp_ms: f64, column: usize, is_press: bool) {
        self.inputs.push(ReplayInput {
            timestamp_ms,
            column,
            is_press,
        });
    }

    /// Ajoute un press.
    #[inline]
    pub fn add_press(&mut self, timestamp_ms: f64, column: usize) {
        self.add_input(timestamp_ms, column, true);
    }

    /// Ajoute un release.
    #[inline]
    pub fn add_release(&mut self, timestamp_ms: f64, column: usize) {
        self.add_input(timestamp_ms, column, false);
    }

    /// Serializes to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Reconstruit le HitWindow à partir des paramètres sauvegardés.
    pub fn build_hit_window(&self) -> HitWindow {
        match self.hit_window_mode {
            HitWindowMode::OsuOD => HitWindow::from_osu_od(self.hit_window_value),
            HitWindowMode::EtternaJudge => {
                HitWindow::from_etterna_judge(self.hit_window_value as u8)
            }
        }
    }
}

impl Default for ReplayData {
    fn default() -> Self {
        Self::new(1.0, HitWindowMode::OsuOD, 5.0)
    }
}

impl ReplayData {
    /// Crée un ReplayData vide (pour fallback/tests).
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Recalcule les stats à partir des hit_timings d'un ReplayResult.
/// Utile pour rejuger un résultat déjà simulé avec un nouveau hit window
/// SANS avoir accès à la chart originale (approximation).
pub fn rejudge_hit_timings(
    hit_timings: &[HitTiming],
    hit_window: &HitWindow,
) -> (HitStats, f64) {
    let mut stats = HitStats::new();

    for hit in hit_timings {
        let (judgement, _) = hit_window.judge(hit.timing_ms);

        match judgement {
            Judgement::Marv => stats.marv += 1,
            Judgement::Perfect => stats.perfect += 1,
            Judgement::Great => stats.great += 1,
            Judgement::Good => stats.good += 1,
            Judgement::Bad => stats.bad += 1,
            Judgement::Miss => stats.miss += 1,
            Judgement::GhostTap => stats.ghost_tap += 1,
        }
    }

    let accuracy = stats.calculate_accuracy();
    (stats, accuracy)
}

/// Timing d'un hit individuel pour les graphes et l'analyse.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HitTiming {
    /// Index de la note touchée.
    pub note_index: usize,
    /// Timing en ms (négatif = early, positif = late).
    pub timing_ms: f64,
    /// Jugement attribué.
    pub judgement: Judgement,
    /// Timestamp de la note dans la map.
    pub note_timestamp_ms: f64,
}

/// Ghost tap (press sans note correspondante).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GhostTap {
    /// Timestamp du ghost tap.
    pub timestamp_ms: f64,
    /// Colonne.
    pub column: usize,
}

/// Résultat complet d'une simulation de replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayResult {
    /// Statistiques de hits.
    pub hit_stats: HitStats,
    /// Précision calculée (0-100).
    pub accuracy: f64,
    /// Score total.
    pub score: u32,
    /// Combo maximum atteint.
    pub max_combo: u32,
    /// Détails de chaque hit pour les graphes.
    pub hit_timings: Vec<HitTiming>,
    /// Liste des ghost taps.
    pub ghost_taps: Vec<GhostTap>,
}

impl ReplayResult {
    pub fn new() -> Self {
        Self {
            hit_stats: HitStats::new(),
            accuracy: 0.0,
            score: 0,
            max_combo: 0,
            hit_timings: Vec::new(),
            ghost_taps: Vec::new(),
        }
    }
}

impl Default for ReplayResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Simule un replay sur une chart avec un hit window donné.
///
/// Cette fonction rejoue les inputs enregistrés sur la map pour
/// recalculer toutes les statistiques de manière déterministe.
pub fn simulate_replay(
    replay_data: &ReplayData,
    chart: &[NoteData],
    hit_window: &HitWindow,
) -> ReplayResult {
    let mut result = ReplayResult::new();
    let mut combo: u32 = 0;

    // Tracker les notes touchées (index -> hit).
    let mut note_hit: Vec<bool> = vec![false; chart.len()];

    // Index de tête pour optimiser la recherche.
    let mut head_index: usize = 0;

    for input in &replay_data.inputs {
        // Avant de traiter cet input, vérifier les notes manquées.
        while head_index < chart.len() {
            if note_hit[head_index] {
                head_index += 1;
                continue;
            }

            let note = &chart[head_index];
            let miss_deadline = note.timestamp_ms + hit_window.miss_ms;

            if input.timestamp_ms > miss_deadline {
                // Miss!
                note_hit[head_index] = true;
                result.hit_stats.miss += 1;
                combo = 0;

                result.hit_timings.push(HitTiming {
                    note_index: head_index,
                    timing_ms: hit_window.miss_ms,
                    judgement: Judgement::Miss,
                    note_timestamp_ms: note.timestamp_ms,
                });

                head_index += 1;
            } else {
                break;
            }
        }

        // Traiter seulement les press (les release sont ignorés pour le scoring).
        if !input.is_press {
            continue;
        }

        // Chercher la meilleure note à frapper dans cette colonne.
        let current_time = input.timestamp_ms;
        let mut best_match: Option<(usize, f64)> = None;
        let search_limit = current_time + hit_window.miss_ms;

        for i in head_index..chart.len() {
            let note = &chart[i];

            if note.timestamp_ms > search_limit {
                break;
            }

            if note.column == input.column && !note_hit[i] {
                let diff = (note.timestamp_ms - current_time).abs();
                if diff <= hit_window.miss_ms {
                    if best_match.is_none() || diff < best_match.unwrap().1 {
                        best_match = Some((i, diff));
                    }
                }
            }
        }

        if let Some((idx, _)) = best_match {
            let note = &chart[idx];
            let diff = note.timestamp_ms - current_time; // Signé: négatif = early
            let (judgement, _) = hit_window.judge(diff);

            note_hit[idx] = true;

            // Appliquer le jugement.
            match judgement {
                Judgement::Miss => {
                    result.hit_stats.miss += 1;
                    combo = 0;
                }
                Judgement::GhostTap => {
                    result.hit_stats.ghost_tap += 1;
                }
                _ => {
                    match judgement {
                        Judgement::Marv => result.hit_stats.marv += 1,
                        Judgement::Perfect => result.hit_stats.perfect += 1,
                        Judgement::Great => result.hit_stats.great += 1,
                        Judgement::Good => result.hit_stats.good += 1,
                        Judgement::Bad => result.hit_stats.bad += 1,
                        _ => {}
                    }
                    combo += 1;
                    if combo > result.max_combo {
                        result.max_combo = combo;
                    }
                    result.score += match judgement {
                        Judgement::Marv => 300,
                        Judgement::Perfect => 300,
                        Judgement::Great => 200,
                        Judgement::Good => 100,
                        Judgement::Bad => 50,
                        _ => 0,
                    };
                }
            }

            result.hit_timings.push(HitTiming {
                note_index: idx,
                timing_ms: diff,
                judgement,
                note_timestamp_ms: note.timestamp_ms,
            });
        } else {
            // Ghost tap - aucune note correspondante.
            result.hit_stats.ghost_tap += 1;
            result.ghost_taps.push(GhostTap {
                timestamp_ms: current_time,
                column: input.column,
            });
        }
    }

    // Après tous les inputs, vérifier les notes restantes non touchées (misses finaux).
    for (idx, note) in chart.iter().enumerate() {
        if !note_hit[idx] {
            result.hit_stats.miss += 1;
            result.hit_timings.push(HitTiming {
                note_index: idx,
                timing_ms: hit_window.miss_ms,
                judgement: Judgement::Miss,
                note_timestamp_ms: note.timestamp_ms,
            });
        }
    }

    // Calculer l'accuracy finale.
    result.accuracy = result.hit_stats.calculate_accuracy();

    result
}

/// Rejuge un replay avec un nouveau hit window.
///
/// Utile pour voir comment le score changerait avec des paramètres différents.
pub fn rejudge_replay(
    replay_data: &ReplayData,
    chart: &[NoteData],
    new_hit_window: &HitWindow,
) -> ReplayResult {
    simulate_replay(replay_data, chart, new_hit_window)
}
