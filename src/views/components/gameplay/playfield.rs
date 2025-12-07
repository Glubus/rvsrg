use crate::models::engine::{
    HIT_LINE_Y, InstanceRaw, NUM_COLUMNS, NoteData, NoteType, PixelSystem, PlayfieldConfig,
    VISIBLE_DISTANCE,
};

/// Type of visual element to render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteVisual {
    /// Regular tap note
    Tap,
    /// Mine (different texture)
    Mine,
    /// Hold note body (stretched)
    HoldBody,
    /// Hold note end cap
    HoldEnd,
    /// Burst note body (stretched)
    BurstBody,
    /// Burst note end cap
    BurstEnd,
}

/// A renderable note instance with its visual type.
pub struct NoteInstance {
    pub column: usize,
    pub visual: NoteVisual,
    pub instance: InstanceRaw,
}

pub struct PlayfieldDisplay {
    pub config: PlayfieldConfig,
}

impl PlayfieldDisplay {
    pub fn new(config: PlayfieldConfig) -> Self {
        Self { config }
    }

    pub fn get_total_width_pixels(&self) -> f32 {
        let cols = NUM_COLUMNS as f32;
        let spaces = (cols - 1.0).max(0.0);
        (cols * self.config.column_width_pixels) + (spaces * self.config.receptor_spacing_pixels)
    }

    pub fn get_bounds(&self, pixel_system: &PixelSystem) -> (f32, f32) {
        let total_width_px = self.get_total_width_pixels();
        let width_norm = pixel_system.x_pixels_to_normalized(total_width_px);
        let x = -width_norm / 2.0; // Centré
        (x, width_norm)
    }

    /// Calcule la position de chaque note visible.
    /// Returns (column, InstanceRaw) for backward compatibility.
    /// Use render_notes_typed for full note type support.
    pub fn render_notes(
        &self,
        visible_notes: &[NoteData],
        song_time: f64,
        scroll_speed_ms: f64,
        pixel_system: &PixelSystem,
    ) -> Vec<(usize, InstanceRaw)> {
        // Convert typed instances to simple format for backward compatibility
        self.render_notes_typed(visible_notes, song_time, scroll_speed_ms, pixel_system)
            .into_iter()
            .filter(|n| n.visual == NoteVisual::Tap) // Only tap notes for old system
            .map(|n| (n.column, n.instance))
            .collect()
    }

    /// Calcule la position de chaque note visible avec le type visuel.
    pub fn render_notes_typed(
        &self,
        visible_notes: &[NoteData],
        song_time: f64,
        scroll_speed_ms: f64,
        pixel_system: &PixelSystem,
    ) -> Vec<NoteInstance> {
        let (playfield_left_x, _) = self.get_bounds(pixel_system);

        // Conversion pixels -> normalisé GPU
        let column_width_norm =
            pixel_system.x_pixels_to_normalized(self.config.column_width_pixels);
        let spacing_norm = pixel_system.x_pixels_to_normalized(self.config.receptor_spacing_pixels);
        let note_width_norm = pixel_system.x_pixels_to_normalized(self.config.note_width_pixels);
        let note_height_norm = pixel_system.y_pixels_to_normalized(self.config.note_height_pixels);

        // LN body/end width is 95% of note width for visual distinction
        let ln_width_norm = note_width_norm * 0.95;

        // Offsets globaux
        let x_offset_norm = pixel_system.x_pixels_to_normalized(self.config.x_offset_pixels);
        let y_offset_norm = pixel_system.y_pixels_to_normalized(self.config.y_offset_pixels);

        let mut instances = Vec::with_capacity(visible_notes.len() * 2); // LNs can generate multiple

        for note in visible_notes {
            if note.hit {
                continue;
            }

            // Position X (commune à tous les types)
            let col_offset = note.column as f32 * (column_width_norm + spacing_norm);
            let center_x =
                playfield_left_x + col_offset + (column_width_norm / 2.0) + x_offset_norm;

            // Physique de défilement : Distance = Temps / Vitesse
            let time_to_hit = note.timestamp_ms - song_time;
            let progress = time_to_hit / scroll_speed_ms;

            let y_pos = (HIT_LINE_Y as f64
                + y_offset_norm as f64
                + (VISIBLE_DISTANCE as f64 * progress)) as f32;

            match &note.note_type {
                NoteType::Tap => {
                    instances.push(NoteInstance {
                        column: note.column,
                        visual: NoteVisual::Tap,
                        instance: InstanceRaw {
                            offset: [center_x, y_pos],
                            scale: [note_width_norm, note_height_norm],
                        },
                    });
                }

                NoteType::Mine => {
                    instances.push(NoteInstance {
                        column: note.column,
                        visual: NoteVisual::Mine,
                        instance: InstanceRaw {
                            offset: [center_x, y_pos],
                            scale: [note_width_norm, note_height_norm],
                        },
                    });
                }

                NoteType::Hold {
                    duration_ms,
                    is_held,
                    ..
                } => {
                    let end_time = note.timestamp_ms + duration_ms;
                    let end_progress = (end_time - song_time) / scroll_speed_ms;
                    let end_y_pos = (HIT_LINE_Y as f64
                        + y_offset_norm as f64
                        + (VISIBLE_DISTANCE as f64 * end_progress))
                        as f32;

                    // If being held, clamp the start to the hit line (don't go below receptors)
                    let hit_line_y = HIT_LINE_Y + y_offset_norm;
                    let clamped_y_pos = if *is_held && y_pos < hit_line_y {
                        hit_line_y
                    } else {
                        y_pos
                    };

                    let body_height = (end_y_pos - clamped_y_pos).abs();
                    let body_center_y = (clamped_y_pos + end_y_pos) / 2.0;

                    // Body (stretched, 95% width)
                    if body_height > 0.001 {
                        instances.push(NoteInstance {
                            column: note.column,
                            visual: NoteVisual::HoldBody,
                            instance: InstanceRaw {
                                offset: [center_x, body_center_y],
                                scale: [ln_width_norm, body_height],
                            },
                        });
                    }

                    // Head (tap visual) - only show if not clamped (not being held past the hit line)
                    if clamped_y_pos == y_pos {
                        instances.push(NoteInstance {
                            column: note.column,
                            visual: NoteVisual::Tap,
                            instance: InstanceRaw {
                                offset: [center_x, y_pos],
                                scale: [note_width_norm, note_height_norm],
                            },
                        });
                    }

                    // End cap (95% width)
                    instances.push(NoteInstance {
                        column: note.column,
                        visual: NoteVisual::HoldEnd,
                        instance: InstanceRaw {
                            offset: [center_x, end_y_pos],
                            scale: [ln_width_norm, note_height_norm],
                        },
                    });
                }

                NoteType::Burst {
                    duration_ms,
                    current_hits,
                    ..
                } => {
                    let end_time = note.timestamp_ms + duration_ms;
                    let end_progress = (end_time - song_time) / scroll_speed_ms;
                    let end_y_pos = (HIT_LINE_Y as f64
                        + y_offset_norm as f64
                        + (VISIBLE_DISTANCE as f64 * end_progress))
                        as f32;

                    // If started hitting, clamp the start to the hit line
                    let hit_line_y = HIT_LINE_Y + y_offset_norm;
                    let started = *current_hits > 0;
                    let clamped_y_pos = if started && y_pos < hit_line_y {
                        hit_line_y
                    } else {
                        y_pos
                    };

                    let body_height = (end_y_pos - clamped_y_pos).abs();
                    let body_center_y = (clamped_y_pos + end_y_pos) / 2.0;

                    // Body (stretched, 95% width)
                    if body_height > 0.001 {
                        instances.push(NoteInstance {
                            column: note.column,
                            visual: NoteVisual::BurstBody,
                            instance: InstanceRaw {
                                offset: [center_x, body_center_y],
                                scale: [ln_width_norm, body_height],
                            },
                        });
                    }

                    // Head (tap visual) - only show if not clamped
                    if clamped_y_pos == y_pos {
                        instances.push(NoteInstance {
                            column: note.column,
                            visual: NoteVisual::Tap,
                            instance: InstanceRaw {
                                offset: [center_x, y_pos],
                                scale: [note_width_norm, note_height_norm],
                            },
                        });
                    }

                    // End cap (95% width)
                    instances.push(NoteInstance {
                        column: note.column,
                        visual: NoteVisual::BurstEnd,
                        instance: InstanceRaw {
                            offset: [center_x, end_y_pos],
                            scale: [ln_width_norm, note_height_norm],
                        },
                    });
                }
            }
        }
        instances
    }

    /// Génère les instances pour les récepteurs fixes (en bas)
    pub fn render_receptors(&self, pixel_system: &PixelSystem) -> Vec<InstanceRaw> {
        let (playfield_left_x, _) = self.get_bounds(pixel_system);

        let column_width_norm =
            pixel_system.x_pixels_to_normalized(self.config.column_width_pixels);
        let spacing_norm = pixel_system.x_pixels_to_normalized(self.config.receptor_spacing_pixels);
        let receptor_width_norm =
            pixel_system.x_pixels_to_normalized(self.config.receptor_width_pixels);
        let receptor_height_norm =
            pixel_system.y_pixels_to_normalized(self.config.receptor_height_pixels);
        let x_offset_norm = pixel_system.x_pixels_to_normalized(self.config.x_offset_pixels);
        let y_offset_norm = pixel_system.y_pixels_to_normalized(self.config.y_offset_pixels);

        let mut instances = Vec::with_capacity(NUM_COLUMNS);

        for col in 0..NUM_COLUMNS {
            let col_offset = col as f32 * (column_width_norm + spacing_norm);
            let center_x =
                playfield_left_x + col_offset + (column_width_norm / 2.0) + x_offset_norm;
            let center_y = HIT_LINE_Y + y_offset_norm;

            instances.push(InstanceRaw {
                offset: [center_x, center_y],
                scale: [receptor_width_norm, receptor_height_norm],
            });
        }
        instances
    }
}

