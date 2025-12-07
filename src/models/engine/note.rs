//! Structures et fonctions de chargement de charts osu!mania.

use rosu_map::section::hit_objects::{HitObject, HitObjectKind};
use std::path::PathBuf;

/// Type of note in a rhythm game chart.
#[derive(Clone, Debug)]
pub enum NoteType {
    /// Simple tap note - press and release.
    Tap,

    /// Hold/long note - press and hold for duration.
    Hold {
        duration_ms: f64,
        /// When the player started holding (None if not started).
        start_time: Option<f64>,
        /// Whether currently being held.
        is_held: bool,
    },

    /// Mine/bomb - must NOT be pressed (penalty if hit).
    Mine,

    /// Burst/mash note - must be pressed multiple times within duration.
    Burst {
        duration_ms: f64,
        required_hits: u8,
        /// How many times hit so far.
        current_hits: u8,
    },
}

impl NoteType {
    /// Creates a new Hold with default state.
    pub fn new_hold(duration_ms: f64) -> Self {
        NoteType::Hold {
            duration_ms,
            start_time: None,
            is_held: false,
        }
    }

    /// Creates a new Burst with default state.
    pub fn new_burst(duration_ms: f64, required_hits: u8) -> Self {
        NoteType::Burst {
            duration_ms,
            required_hits,
            current_hits: 0,
        }
    }

    /// Returns true if this is a hold note.
    pub fn is_hold(&self) -> bool {
        matches!(self, NoteType::Hold { .. })
    }

    /// Returns true if this is a mine.
    pub fn is_mine(&self) -> bool {
        matches!(self, NoteType::Mine)
    }

    /// Returns true if this is a tap note.
    pub fn is_tap(&self) -> bool {
        matches!(self, NoteType::Tap)
    }

    /// Returns true if this is a burst/mash note.
    pub fn is_burst(&self) -> bool {
        matches!(self, NoteType::Burst { .. })
    }

    /// Returns the duration if this is a hold or burst, 0 otherwise.
    pub fn duration(&self) -> f64 {
        match self {
            NoteType::Hold { duration_ms, .. } => *duration_ms,
            NoteType::Burst { duration_ms, .. } => *duration_ms,
            _ => 0.0,
        }
    }

    /// Returns the required hits for burst notes, 1 for others.
    pub fn required_hits(&self) -> u8 {
        match self {
            NoteType::Burst { required_hits, .. } => *required_hits,
            NoteType::Mine => 0,
            _ => 1,
        }
    }

    /// Returns true if this note should be hit (not a mine).
    pub fn should_hit(&self) -> bool {
        !self.is_mine()
    }

    /// Returns true if this note has a duration (hold or burst).
    pub fn has_duration(&self) -> bool {
        matches!(self, NoteType::Hold { .. } | NoteType::Burst { .. })
    }

    /// Resets the runtime state (for new gameplay session).
    pub fn reset(&mut self) {
        match self {
            NoteType::Hold {
                start_time,
                is_held,
                ..
            } => {
                *start_time = None;
                *is_held = false;
            }
            NoteType::Burst { current_hits, .. } => {
                *current_hits = 0;
            }
            _ => {}
        }
    }
}

/// A single note in a rhythm game chart.
#[derive(Clone, Debug)]
pub struct NoteData {
    /// When the note should be hit (in milliseconds).
    pub timestamp_ms: f64,
    /// Which column/lane (0-indexed).
    pub column: usize,
    /// Whether this note has been fully completed.
    pub hit: bool,
    /// The type of note (tap, hold, mine, burst) with its state.
    pub note_type: NoteType,
}

impl NoteData {
    /// Creates a new tap note.
    pub fn tap(timestamp_ms: f64, column: usize) -> Self {
        Self {
            timestamp_ms,
            column,
            hit: false,
            note_type: NoteType::Tap,
        }
    }

    /// Creates a new hold note.
    pub fn hold(timestamp_ms: f64, column: usize, duration_ms: f64) -> Self {
        Self {
            timestamp_ms,
            column,
            hit: false,
            note_type: NoteType::new_hold(duration_ms),
        }
    }

    /// Creates a new mine note.
    pub fn mine(timestamp_ms: f64, column: usize) -> Self {
        Self {
            timestamp_ms,
            column,
            hit: false,
            note_type: NoteType::Mine,
        }
    }

    /// Creates a burst/mash note.
    pub fn burst(timestamp_ms: f64, column: usize, duration_ms: f64, required_hits: u8) -> Self {
        Self {
            timestamp_ms,
            column,
            hit: false,
            note_type: NoteType::new_burst(duration_ms, required_hits),
        }
    }

    /// Returns the end time of this note.
    /// For holds/bursts: start + duration. For others: same as start.
    pub fn end_time_ms(&self) -> f64 {
        self.timestamp_ms + self.note_type.duration()
    }

    /// Returns true if this is a hold note.
    pub fn is_hold(&self) -> bool {
        self.note_type.is_hold()
    }

    /// Returns true if this is a mine.
    pub fn is_mine(&self) -> bool {
        self.note_type.is_mine()
    }

    /// Returns true if this is a tap note.
    pub fn is_tap(&self) -> bool {
        self.note_type.is_tap()
    }

    /// Returns true if this is a burst/mash note.
    pub fn is_burst(&self) -> bool {
        self.note_type.is_burst()
    }

    /// Returns the duration (for holds/bursts), 0 otherwise.
    pub fn hold_duration_ms(&self) -> f64 {
        self.note_type.duration()
    }

    /// Returns true if this note should be hit.
    pub fn should_hit(&self) -> bool {
        self.note_type.should_hit()
    }

    /// Returns the number of hits required for this note.
    pub fn required_hits(&self) -> u8 {
        self.note_type.required_hits()
    }

    /// Returns true if this note has a duration (hold or burst).
    pub fn has_duration(&self) -> bool {
        self.note_type.has_duration()
    }

    /// Creates a copy of this note with all runtime state reset.
    /// Used when starting a new gameplay session from cached chart.
    pub fn reset(&self) -> Self {
        let mut note = self.clone();
        note.hit = false;
        note.note_type.reset();
        note
    }
}

/// Charge une map depuis un fichier .osu.
/// Retourne le chemin audio et la liste des notes, ou une erreur si le chargement échoue.
pub fn load_map(path: PathBuf) -> Result<(PathBuf, Vec<NoteData>), String> {
    let map = rosu_map::Beatmap::from_path(&path)
        .map_err(|e| format!("Failed to load beatmap {:?}: {}", path, e))?;

    let audio_path = path
        .parent()
        .ok_or_else(|| format!("Invalid path (no parent): {:?}", path))?
        .join(&map.audio_file);

    let key_count = map.circle_size as u8;

    let mut notes = Vec::new();
    for hit_object in map.hit_objects {
        if let Some(note) = parse_hit_object(&hit_object, key_count) {
            notes.push(note);
        }
    }

    Ok((audio_path, notes))
}

/// Charge une map depuis un fichier .osu, version safe qui retourne Option.
/// Utilisé pour le cache où on ne veut pas panic.
pub fn load_map_safe(path: &PathBuf) -> Option<(PathBuf, Vec<NoteData>)> {
    let map = rosu_map::Beatmap::from_path(path).ok()?;
    let audio_path = path.parent()?.join(&map.audio_file);
    let key_count = map.circle_size as u8;

    let mut notes = Vec::new();
    for hit_object in map.hit_objects {
        if let Some(note) = parse_hit_object(&hit_object, key_count) {
            notes.push(note);
        }
    }

    Some((audio_path, notes))
}

/// Parse un HitObject osu! et retourne une NoteData.
pub fn parse_hit_object(hit_object: &HitObject, key_count: u8) -> Option<NoteData> {
    match &hit_object.kind {
        HitObjectKind::Circle(circle) => {
            let column = x_to_column_generic(circle.pos.x as i32, key_count)?;
            Some(NoteData::tap(hit_object.start_time, column))
        }
        HitObjectKind::Hold(hold) => {
            // For holds, use the end_info position which contains the x coordinate
            let column = x_to_column_generic(hold.pos_x as i32, key_count)?;
            Some(NoteData::hold(hit_object.start_time, column, hold.duration))
        }
        _ => None, // On ignore les sliders, spinners, etc.
    }
}

/// Parse un HitObject osu! et retourne l'index de colonne si c'est un cercle valide.
/// (Backward compat - préférer parse_hit_object)
pub fn parse_hit_object_column(hit_object: &HitObject) -> Option<usize> {
    match &hit_object.kind {
        HitObjectKind::Circle(circle) => x_to_column(circle.pos.x as i32),
        _ => None,
    }
}

/// Convertit une position X osu!mania en index de colonne (générique).
pub fn x_to_column_generic(x: i32, key_count: u8) -> Option<usize> {
    let column_width = 512.0 / key_count as f32;
    let col = (x as f32 / column_width).floor() as usize;
    if col < key_count as usize {
        Some(col)
    } else {
        None
    }
}

/// Convertit une position X osu!mania en index de colonne.
/// Supporte 4K, 5K, 6K, 7K.
pub fn x_to_column(x: i32) -> Option<usize> {
    // osu!mania utilise une grille de 512 pixels de large.
    // Pour 4K: colonnes à 64, 192, 320, 448
    // Pour 7K: colonnes à 36, 109, 182, 256, 329, 402, 475
    // Formule générale: column_width = 512 / key_count
    // Position = column_width / 2 + column_index * column_width

    // Valeurs communes pour 4K (le plus fréquent)
    match x {
        64 => Some(0),
        192 => Some(1),
        320 => Some(2),
        448 => Some(3),
        // 5K (center column at 256)
        51 => Some(0),
        153 => Some(1),
        // 256 handled by 7K below (same center position)
        358 => Some(3),
        460 => Some(4),
        // 6K
        42 => Some(0),
        128 => Some(1),
        213 => Some(2),
        298 => Some(3),
        384 => Some(4),
        469 => Some(5),
        // 7K (center column at 256, also used by 5K)
        36 => Some(0),
        109 => Some(1),
        182 => Some(2),
        256 => Some(2), // Center column for 5K/7K
        329 => Some(4),
        402 => Some(5),
        475 => Some(6),
        _ => {
            // Fallback: try to calculate column
            // Assume 4K by default
            let column_width = 512 / 4;
            let col = (x - column_width / 2) / column_width;
            if (0..10).contains(&col) {
                Some(col as usize)
            } else {
                log::warn!("Unknown column position: {x}");
                None
            }
        }
    }
}

