//! Note structures and chart loading for VSRG multi-formats.
//!
//! Uses rhythm-open-exchange (ROX) as the underlying note representation.
//! Supports: .osu (mania/taiko), .qua, .sm, .ssc, .json
//!
//! # Time Units
//!
//! All times are in **microseconds (i64)** to match ROX precision.
//! 1 second = 1,000,000 microseconds (µs)
//!
//! # Architecture
//!
//! - `rhythm_open_exchange::Note` - The underlying note data (time, column, type)
//! - `NoteData` - Wrapper that adds gameplay state (hit, is_held, current_hits)

use rhythm_open_exchange::codec::auto_decode;
use rhythm_open_exchange::{Note as RoxNote, NoteType as RoxNoteType};
use std::path::PathBuf;

// Re-export ROX types for external use
pub use rhythm_open_exchange::NoteType;

/// Microseconds per second.
pub const US_PER_SECOND: i64 = 1_000_000;
/// Microseconds per millisecond.
pub const US_PER_MS: i64 = 1_000;

/// Gameplay state for hold notes.
#[derive(Clone, Debug, Default)]
pub struct HoldState {
    /// When the player started holding (None if not started), in microseconds.
    pub start_time_us: Option<i64>,
    /// Whether currently being held.
    pub is_held: bool,
}

/// Gameplay state for burst/roll notes.
#[derive(Clone, Debug, Default)]
pub struct BurstState {
    /// How many times hit so far.
    pub current_hits: u8,
    /// Required hits to complete (calculated from duration).
    pub required_hits: u8,
}

/// Runtime gameplay state that varies during play.
#[derive(Clone, Debug, Default)]
pub struct NoteState {
    /// Whether this note has been fully completed.
    pub hit: bool,
    /// State for hold notes.
    pub hold: HoldState,
    /// State for burst notes.
    pub burst: BurstState,
}

impl NoteState {
    /// Reset all gameplay state for a new session.
    pub fn reset(&mut self) {
        self.hit = false;
        self.hold = HoldState::default();
        self.burst = BurstState::default();
    }
}

/// A single note in a rhythm game chart with gameplay state.
///
/// Wraps `rhythm_open_exchange::Note` and adds mutable state for gameplay.
/// All times are in microseconds (i64).
#[derive(Clone, Debug)]
pub struct NoteData {
    /// The underlying ROX note (immutable chart data).
    inner: RoxNote,
    /// Mutable gameplay state.
    pub state: NoteState,
}

impl NoteData {
    /// Create a NoteData from a ROX Note.
    pub fn new(note: RoxNote) -> Self {
        let required_hits = match note.note_type {
            RoxNoteType::Burst { duration_us } => {
                // 10 hits per second
                let duration_secs = duration_us as f64 / US_PER_SECOND as f64;
                (duration_secs * 10.0).max(1.0) as u8
            }
            _ => 1,
        };

        Self {
            inner: note,
            state: NoteState {
                hit: false,
                hold: HoldState::default(),
                burst: BurstState {
                    current_hits: 0,
                    required_hits,
                },
            },
        }
    }

    // ========== Convenience constructors ==========

    /// Create a tap note at the given time and column.
    pub fn tap(time_us: i64, column: u8) -> Self {
        Self::new(RoxNote::tap(time_us, column))
    }

    /// Create a hold note at the given time, column, and duration.
    pub fn hold(time_us: i64, column: u8, duration_us: i64) -> Self {
        Self::new(RoxNote::hold(time_us, duration_us, column))
    }

    /// Create a mine note at the given time and column.
    pub fn mine(time_us: i64, column: u8) -> Self {
        Self::new(RoxNote::mine(time_us, column))
    }

    /// Create a burst/roll note at the given time, column, and duration.
    pub fn burst(time_us: i64, column: u8, duration_us: i64) -> Self {
        Self::new(RoxNote::burst(time_us, duration_us, column))
    }

    // ========== Accessors to inner ROX Note ==========

    /// When the note should be hit (in microseconds).
    #[inline]
    pub fn time_us(&self) -> i64 {
        self.inner.time_us
    }

    /// Which column/lane (0-indexed).
    #[inline]
    pub fn column(&self) -> usize {
        self.inner.column as usize
    }

    /// The ROX note type.
    #[inline]
    pub fn note_type(&self) -> &RoxNoteType {
        &self.inner.note_type
    }

    /// Access the inner ROX note.
    #[inline]
    pub fn inner(&self) -> &RoxNote {
        &self.inner
    }

    // ========== Type checks ==========

    /// Returns true if this is a tap note.
    #[inline]
    pub fn is_tap(&self) -> bool {
        matches!(self.inner.note_type, RoxNoteType::Tap)
    }

    /// Returns true if this is a hold note.
    #[inline]
    pub fn is_hold(&self) -> bool {
        matches!(self.inner.note_type, RoxNoteType::Hold { .. })
    }

    /// Returns true if this is a burst/roll note.
    #[inline]
    pub fn is_burst(&self) -> bool {
        matches!(self.inner.note_type, RoxNoteType::Burst { .. })
    }

    /// Returns true if this is a mine.
    #[inline]
    pub fn is_mine(&self) -> bool {
        matches!(self.inner.note_type, RoxNoteType::Mine)
    }

    // ========== Duration helpers (all in microseconds) ==========

    /// Returns the duration in microseconds for holds/bursts, 0 for others.
    #[inline]
    pub fn duration_us(&self) -> i64 {
        match self.inner.note_type {
            RoxNoteType::Hold { duration_us } | RoxNoteType::Burst { duration_us } => duration_us,
            _ => 0,
        }
    }

    /// Returns the end time in microseconds (start + duration).
    #[inline]
    pub fn end_time_us(&self) -> i64 {
        self.inner.time_us + self.duration_us()
    }

    /// Returns true if this note has a duration (hold or burst).
    #[inline]
    pub fn has_duration(&self) -> bool {
        matches!(
            self.inner.note_type,
            RoxNoteType::Hold { .. } | RoxNoteType::Burst { .. }
        )
    }

    // ========== Gameplay helpers ==========

    /// Returns true if this note should be hit (not a mine).
    #[inline]
    pub fn should_hit(&self) -> bool {
        !self.is_mine()
    }

    /// Returns the number of hits required for this note.
    #[inline]
    pub fn required_hits(&self) -> u8 {
        if self.is_mine() {
            0
        } else {
            self.state.burst.required_hits
        }
    }

    /// Whether this note has been completed.
    #[inline]
    pub fn hit(&self) -> bool {
        self.state.hit
    }

    /// Set hit state.
    #[inline]
    pub fn set_hit(&mut self, hit: bool) {
        self.state.hit = hit;
    }

    /// Creates a copy of this note with all runtime state reset.
    /// Used when starting a new gameplay session from cached chart.
    pub fn reset(&self) -> Self {
        let mut note = self.clone();
        note.state.reset();
        note
    }
}

impl From<RoxNote> for NoteData {
    fn from(note: RoxNote) -> Self {
        NoteData::new(note)
    }
}

impl From<&RoxNote> for NoteData {
    fn from(note: &RoxNote) -> Self {
        NoteData::new(note.clone())
    }
}

// Re-export RoxChart for external use
pub use rhythm_open_exchange::RoxChart;

/// Loads a chart from a file (multi-format via ROX).
/// Supports: .osu (mania/taiko), .qua, .sm, .ssc, .json
/// Returns the full RoxChart for metadata access and difficulty calculation.
pub fn load_chart(path: &std::path::Path) -> Result<RoxChart, String> {
    auto_decode(path).map_err(|e| format!("Failed to load chart {:?}: {}", path, e))
}

/// Safe version of load_chart that returns Option instead of Result.
pub fn load_chart_safe(path: &std::path::Path) -> Option<RoxChart> {
    auto_decode(path).ok()
}

/// Convert a RoxChart's notes to gameplay NoteData.
/// Call this when entering gameplay with the chart.
pub fn notes_from_chart(chart: &RoxChart) -> Vec<NoteData> {
    chart.notes.iter().map(NoteData::from).collect()
}

/// Get the audio path from a chart file path.
pub fn audio_path_from_chart(chart_path: &std::path::Path, chart: &RoxChart) -> Option<PathBuf> {
    chart_path
        .parent()
        .map(|p| p.join(&chart.metadata.audio_file))
}

/// Legacy function for backwards compatibility.
/// Loads a map and returns (audio_path, notes).
pub fn load_map(path: PathBuf) -> Result<(PathBuf, Vec<NoteData>), String> {
    let chart = load_chart(&path)?;
    let audio_path = audio_path_from_chart(&path, &chart)
        .ok_or_else(|| format!("Invalid path (no parent): {:?}", path))?;
    let notes = notes_from_chart(&chart);
    Ok((audio_path, notes))
}

/// Legacy function for backwards compatibility.
/// Safe version that returns Option.
pub fn load_map_safe(path: &PathBuf) -> Option<(PathBuf, Vec<NoteData>)> {
    let chart = load_chart_safe(path)?;
    let audio_path = audio_path_from_chart(path, &chart)?;
    let notes = notes_from_chart(&chart);
    Some((audio_path, notes))
}

// ========== Conversion helpers ==========

/// Convert microseconds to milliseconds (f64 for compatibility).
#[inline]
pub fn us_to_ms(us: i64) -> f64 {
    us as f64 / US_PER_MS as f64
}

/// Convert milliseconds to microseconds.
#[inline]
pub fn ms_to_us(ms: f64) -> i64 {
    (ms * US_PER_MS as f64) as i64
}
