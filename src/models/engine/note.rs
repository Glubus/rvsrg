use rosu_map::section::hit_objects::{HitObject, HitObjectKind};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NoteData {
    pub timestamp_ms: f64,
    pub column: usize,
    pub hit: bool,
}

pub fn load_map(path: PathBuf) -> (PathBuf, Vec<NoteData>) {
    let map = rosu_map::Beatmap::from_path(&path).unwrap();
    let audio_path = path.parent().unwrap().join(map.audio_file);

    let mut notes = Vec::new();
    for hit_object in map.hit_objects {
        if let Ok(column) = map_x_to_column(&hit_object) {
            let adjusted_timestamp = hit_object.start_time;
            notes.push(NoteData {
                timestamp_ms: adjusted_timestamp,
                column,
                hit: false,
            });
        }
    }

    (audio_path, notes)
}

fn map_x_to_column(hit_object: &HitObject) -> Result<usize, String> {
    match hit_object.kind {
        HitObjectKind::Circle(circle) => Ok(x_to_column(circle.pos.x as i32)),
        _ => Err(format!("Hit object is not a circle: {:?}", hit_object.kind)),
    }
}

fn x_to_column(x: i32) -> usize {
    match x {
        64 => 0,
        192 => 1,
        320 => 2,
        448 => 3,
        _ => panic!("Invalid column: {}", x),
    }
}
