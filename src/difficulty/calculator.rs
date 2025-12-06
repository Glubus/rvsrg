//! Trait definition for difficulty calculators.
//!
//! This module defines the `DifficultyCalculator` trait that all difficulty
//! calculators must implement, whether they are built-in (etterna, osu) or
//! custom (Rhai scripts).

use super::BeatmapSsr;
use crate::models::engine::NoteData;
use rosu_map::Beatmap;
use std::collections::HashMap;
use std::fmt::Debug;

/// Error type for difficulty calculation failures.
#[derive(Debug, Clone)]
pub enum CalcError {
    /// The beatmap data is invalid or missing.
    InvalidBeatmap(String),
    /// The calculator failed to compute the difficulty.
    CalculationFailed(String),
    /// The requested rate is not supported.
    UnsupportedRate(f64),
    /// Generic error with message.
    Other(String),
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::InvalidBeatmap(msg) => write!(f, "Invalid beatmap: {}", msg),
            CalcError::CalculationFailed(msg) => write!(f, "Calculation failed: {}", msg),
            CalcError::UnsupportedRate(rate) => write!(f, "Unsupported rate: {}", rate),
            CalcError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CalcError {}

/// Context provided to calculators for difficulty computation.
/// Built from already-parsed note data (no re-parsing needed).
#[derive(Debug, Clone)]
pub struct CalculationContext {
    /// All notes in the beatmap.
    pub notes: Vec<NoteData>,
    /// Number of keys (4k, 7k, etc.).
    pub key_count: u8,
    /// Total duration in milliseconds.
    pub duration_ms: f64,
    /// Primary BPM of the map.
    pub bpm: f64,
    /// Playback rate multiplier (1.0 = normal).
    pub rate: f64,
    /// Notes per second (at rate 1.0).
    pub nps: f64,
    /// Results from other calculators (for hybrid calculations).
    pub other_results: HashMap<String, BeatmapSsr>,
}

impl CalculationContext {
    /// Creates a new context from already-parsed notes.
    pub fn new(notes: Vec<NoteData>, key_count: u8, bpm: f64, rate: f64) -> Self {
        let duration_ms = if notes.is_empty() {
            0.0
        } else {
            let first = notes.first().map(|n| n.timestamp_ms).unwrap_or(0.0);
            let last = notes.last().map(|n| n.end_time_ms()).unwrap_or(first);
            (last - first).max(0.0)
        };

        let duration_secs = duration_ms / 1000.0;
        let nps = if duration_secs > 0.0 {
            notes.len() as f64 / duration_secs
        } else {
            0.0
        };

        Self {
            notes,
            key_count,
            duration_ms,
            bpm,
            rate,
            nps,
            other_results: HashMap::new(),
        }
    }

    /// Creates a context from a rosu-map Beatmap.
    pub fn from_beatmap(map: &Beatmap, rate: f64) -> Self {
        use rosu_map::section::hit_objects::HitObjectKind;

        let key_count = map.circle_size as u8;

        // Parse notes from hit objects
        let notes: Vec<NoteData> = map
            .hit_objects
            .iter()
            .filter_map(|obj| {
                let column = crate::models::engine::note::x_to_column_generic(
                    match &obj.kind {
                        HitObjectKind::Circle(c) => c.pos.x as i32,
                        HitObjectKind::Hold(h) => h.pos_x as i32,
                        _ => return None,
                    },
                    key_count,
                )?;

                match &obj.kind {
                    HitObjectKind::Circle(_) => Some(NoteData::tap(obj.start_time, column)),
                    HitObjectKind::Hold(hold) => {
                        Some(NoteData::hold(obj.start_time, column, hold.duration))
                    }
                    _ => None,
                }
            })
            .collect();

        // Get BPM from timing points
        let bpm = map
            .control_points
            .timing_points
            .first()
            .map(|tp| 60000.0 / tp.beat_len)
            .unwrap_or(120.0);

        Self::new(notes, key_count, bpm, rate)
    }
}

/// Trait that all difficulty calculators must implement.
pub trait DifficultyCalculator: Send + Sync + Debug {
    /// Unique identifier (e.g., "etterna", "osu", "custom_nps").
    fn id(&self) -> &str;

    /// Human-readable display name.
    fn display_name(&self) -> &str;

    /// Version string for cache invalidation.
    fn version(&self) -> &str;

    /// Computes difficulty for the given context.
    fn calculate(&self, ctx: &CalculationContext) -> Result<BeatmapSsr, CalcError>;

    /// Whether this calculator supports arbitrary rates (continuous).
    /// If false, only discrete rates from `available_rates()` are valid.
    fn supports_arbitrary_rates(&self) -> bool {
        false
    }

    /// Returns the list of discrete rates supported if not arbitrary.
    fn available_rates(&self) -> Option<Vec<f64>> {
        None
    }

    /// Returns a full calculator ID including version.
    fn full_id(&self) -> String {
        format!("{}_{}", self.id(), self.version())
    }
}
