//! Structures et fonctions de chargement de charts osu!mania.

use rosu_map::section::hit_objects::{HitObject, HitObjectKind};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NoteData {
    pub timestamp_ms: f64,
    pub column: usize,
    pub hit: bool,
    pub is_hold: bool,
    pub hold_duration_ms: f64,
}

impl NoteData {
    /// Creates a new tap note.
    pub fn tap(timestamp_ms: f64, column: usize) -> Self {
        Self {
            timestamp_ms,
            column,
            hit: false,
            is_hold: false,
            hold_duration_ms: 0.0,
        }
    }

    /// Creates a new hold note.
    pub fn hold(timestamp_ms: f64, column: usize, duration_ms: f64) -> Self {
        Self {
            timestamp_ms,
            column,
            hit: false,
            is_hold: true,
            hold_duration_ms: duration_ms,
        }
    }

    /// Returns the end time of this note (for holds, start + duration).
    pub fn end_time_ms(&self) -> f64 {
        self.timestamp_ms + self.hold_duration_ms
    }
}

/// Charge une map depuis un fichier .osu.
/// Retourne le chemin audio et la liste des notes.
///
/// # Panics
/// Panic si le fichier ne peut pas être lu ou si la map contient des colonnes invalides.
pub fn load_map(path: PathBuf) -> (PathBuf, Vec<NoteData>) {
    let map = rosu_map::Beatmap::from_path(&path).unwrap();
    let audio_path = path.parent().unwrap().join(map.audio_file);
    let key_count = map.circle_size as u8;

    let mut notes = Vec::new();
    for hit_object in map.hit_objects {
        if let Some(note) = parse_hit_object(&hit_object, key_count) {
            notes.push(note);
        }
    }

    (audio_path, notes)
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
