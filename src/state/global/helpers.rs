//! Debug helpers for testing and development.

use crate::models::engine::NoteData;

/// Creates a debug chart with all note types for testing rendering.
pub(super) fn create_debug_chart() -> Vec<NoteData> {
    let mut notes = Vec::new();
    let time = 1000.0; // Start at 1 second
    let spacing = 500.0; // 500ms between note groups

    // Loop to create multiple sets of all note types
    for iteration in 0..10 {
        let base_time = time + (iteration as f64 * spacing * 8.0);

        // Tap notes (one per column)
        for col in 0..4 {
            notes.push(NoteData::tap(base_time + (col as f64 * 100.0), col));
        }

        // Hold notes (long notes)
        notes.push(NoteData::hold(base_time + spacing, 0, 800.0));
        notes.push(NoteData::hold(base_time + spacing + 200.0, 2, 600.0));

        // Mines (avoid hitting these)
        notes.push(NoteData::mine(base_time + spacing * 2.0, 1));
        notes.push(NoteData::mine(base_time + spacing * 2.0 + 200.0, 3));

        // Burst notes (mash multiple times)
        notes.push(NoteData::burst(base_time + spacing * 3.0, 0, 500.0, 3));
        notes.push(NoteData::burst(
            base_time + spacing * 3.0 + 200.0,
            2,
            400.0,
            4,
        ));

        // Mixed pattern
        notes.push(NoteData::tap(base_time + spacing * 4.0, 1));
        notes.push(NoteData::hold(base_time + spacing * 4.0, 3, 400.0));
        notes.push(NoteData::mine(base_time + spacing * 4.5, 0));
        notes.push(NoteData::burst(base_time + spacing * 5.0, 2, 300.0, 2));
    }

    // Sort by timestamp
    notes.sort_by(|a, b| a.timestamp_ms.partial_cmp(&b.timestamp_ms).unwrap());

    notes
}
