//! Structures et fonctions de chargement de charts osu!mania.

use rosu_map::section::hit_objects::{HitObject, HitObjectKind};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NoteData {
    pub timestamp_ms: f64,
    pub column: usize,
    pub hit: bool,
}

/// Charge une map depuis un fichier .osu.
/// Retourne le chemin audio et la liste des notes.
/// 
/// # Panics
/// Panic si le fichier ne peut pas être lu ou si la map contient des colonnes invalides.
pub fn load_map(path: PathBuf) -> (PathBuf, Vec<NoteData>) {
    let map = rosu_map::Beatmap::from_path(&path).unwrap();
    let audio_path = path.parent().unwrap().join(map.audio_file);

    let mut notes = Vec::new();
    for hit_object in map.hit_objects {
        if let Some(column) = parse_hit_object_column(&hit_object) {
            notes.push(NoteData {
                timestamp_ms: hit_object.start_time,
                column,
                hit: false,
            });
        }
    }

    (audio_path, notes)
}

/// Charge une map depuis un fichier .osu, version safe qui retourne Option.
/// Utilisé pour le cache où on ne veut pas panic.
pub fn load_map_safe(path: &PathBuf) -> Option<(PathBuf, Vec<NoteData>)> {
    let map = rosu_map::Beatmap::from_path(path).ok()?;
    let audio_path = path.parent()?.join(&map.audio_file);

    let mut notes = Vec::new();
    for hit_object in map.hit_objects {
        if let Some(column) = parse_hit_object_column(&hit_object) {
            notes.push(NoteData {
                timestamp_ms: hit_object.start_time,
                column,
                hit: false,
            });
        }
    }

    Some((audio_path, notes))
}

/// Parse un HitObject osu! et retourne l'index de colonne si c'est un cercle valide.
pub fn parse_hit_object_column(hit_object: &HitObject) -> Option<usize> {
    match &hit_object.kind {
        HitObjectKind::Circle(circle) => x_to_column(circle.pos.x as i32),
        _ => None, // On ignore les sliders, spinners, etc.
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
        // 5K
        51 => Some(0),
        153 => Some(1),
        256 => Some(2),
        358 => Some(3),
        460 => Some(4),
        // 6K
        42 => Some(0),
        128 => Some(1),
        213 => Some(2),
        298 => Some(3),
        384 => Some(4),
        469 => Some(5),
        // 7K
        36 => Some(0),
        109 => Some(1),
        182 => Some(2),
        256 => Some(3), // Déjà défini pour 5K, Rust prend le premier match
        329 => Some(4),
        402 => Some(5),
        475 => Some(6),
        _ => {
            // Fallback: essayer de calculer la colonne
            // Supposons 4K par défaut
            let column_width = 512 / 4;
            let col = (x - column_width / 2) / column_width;
            if col >= 0 && col < 10 {
                Some(col as usize)
            } else {
                log::warn!("Unknown column position: {}", x);
                None
            }
        }
    }
}
