//! Debug helpers for testing and development.

use crate::models::engine::{NoteData, US_PER_MS};

/// Creates a debug chart with all note types for testing rendering.
pub(super) fn create_debug_chart() -> Vec<NoteData> {
    let mut notes = Vec::new();
    let time_us: i64 = 1_000_000; // Start at 1 second (in µs)
    let spacing_us: i64 = 500 * US_PER_MS; // 500ms between note groups

    // Loop to create multiple sets of all note types
    for iteration in 0..10 {
        let base_time_us = time_us + (iteration as i64 * spacing_us * 8);

        // Tap notes (one per column)
        for col in 0..4u8 {
            notes.push(NoteData::tap(
                base_time_us + (col as i64 * 100 * US_PER_MS),
                col,
            ));
        }

        // Hold notes (long notes)
        notes.push(NoteData::hold(
            base_time_us + spacing_us,
            0,
            800 * US_PER_MS,
        ));
        notes.push(NoteData::hold(
            base_time_us + spacing_us + 200 * US_PER_MS,
            2,
            600 * US_PER_MS,
        ));

        // Mines (avoid hitting these)
        notes.push(NoteData::mine(base_time_us + spacing_us * 2, 1));
        notes.push(NoteData::mine(
            base_time_us + spacing_us * 2 + 200 * US_PER_MS,
            3,
        ));

        // Burst notes (mash multiple times)
        notes.push(NoteData::burst(
            base_time_us + spacing_us * 3,
            0,
            500 * US_PER_MS,
        ));
        notes.push(NoteData::burst(
            base_time_us + spacing_us * 3 + 200 * US_PER_MS,
            2,
            400 * US_PER_MS,
        ));

        // Mixed pattern
        notes.push(NoteData::tap(base_time_us + spacing_us * 4, 1));
        notes.push(NoteData::hold(
            base_time_us + spacing_us * 4,
            3,
            400 * US_PER_MS,
        ));
        notes.push(NoteData::mine(base_time_us + spacing_us * 9 / 2, 0)); // 4.5 * spacing
        notes.push(NoteData::burst(
            base_time_us + spacing_us * 5,
            2,
            300 * US_PER_MS,
        ));
    }

    // Sort by timestamp
    notes.sort_by_key(|n| n.time_us());

    notes
}
