//! Note processing - update_notes, apply_judgement

use super::GameEngine;
use crate::models::engine::NoteType;
use crate::models::stats::Judgement;

impl GameEngine {
    /// Updates note states and handles misses for all note types.
    pub(crate) fn update_notes(&mut self, current_time: f64) {
        let miss_threshold = self.hit_window.miss_ms;
        let mut new_head = self.head_index;

        // Collect judgements to apply (to avoid borrow conflicts)
        let mut judgements: Vec<Judgement> = Vec::new();
        let _keys_held = self.keys_held.clone();

        while new_head < self.chart.len() {
            let note = &mut self.chart[new_head];

            // Skip already completed notes
            if note.hit {
                new_head += 1;
                continue;
            }

            let note_timestamp = note.timestamp_ms;
            let note_end_time = note.end_time_ms();

            match &mut note.note_type {
                NoteType::Tap => {
                    if current_time > note_timestamp + miss_threshold {
                        note.hit = true;
                        judgements.push(Judgement::Miss);
                        new_head += 1;
                    } else {
                        break;
                    }
                }

                NoteType::Hold {
                    is_held,
                    start_time,
                    ..
                } => {
                    if *is_held {
                        // Check if hold completed (reached end time)
                        if current_time >= note_end_time {
                            note.hit = true;
                            *is_held = false;
                            judgements.push(Judgement::Marv);
                            new_head += 1;
                        }
                        // Don't advance head_index while holding - note is still active!
                        // Break to stop processing further notes
                        break;
                    } else if start_time.is_none() && current_time > note_timestamp + miss_threshold
                    {
                        // Never started holding - miss
                        note.hit = true;
                        judgements.push(Judgement::Miss);
                        new_head += 1;
                    } else {
                        break;
                    }
                }

                NoteType::Mine => {
                    if current_time > note_timestamp + miss_threshold {
                        note.hit = true;
                        // No judgement - mines that pass are good!
                        new_head += 1;
                    } else {
                        break;
                    }
                }

                NoteType::Burst {
                    duration_ms,
                    required_hits,
                    current_hits,
                } => {
                    if current_time > note_timestamp + *duration_ms {
                        note.hit = true;
                        if *current_hits < *required_hits {
                            let ratio = *current_hits as f64 / *required_hits as f64;
                            let judgement = if ratio >= 0.8 {
                                Judgement::Great
                            } else if ratio >= 0.5 {
                                Judgement::Good
                            } else if ratio > 0.0 {
                                Judgement::Bad
                            } else {
                                Judgement::Miss
                            };
                            judgements.push(judgement);
                        }
                        new_head += 1;
                    } else {
                        break;
                    }
                }
            }
        }

        self.head_index = new_head;

        // Apply collected judgements
        for j in judgements {
            self.apply_judgement(j);
        }
    }

    /// Applies a judgement to the game state (score, combo, stats).
    pub(crate) fn apply_judgement(&mut self, j: Judgement) {
        match j {
            Judgement::Miss => {
                self.hit_stats.miss += 1;
                self.combo = 0;
                self.notes_passed += 1;
            }
            Judgement::GhostTap => {
                self.hit_stats.ghost_tap += 1;
            }
            _ => {
                match j {
                    Judgement::Marv => self.hit_stats.marv += 1,
                    Judgement::Perfect => self.hit_stats.perfect += 1,
                    Judgement::Great => self.hit_stats.great += 1,
                    Judgement::Good => self.hit_stats.good += 1,
                    Judgement::Bad => self.hit_stats.bad += 1,
                    _ => {}
                }
                self.combo += 1;
                self.max_combo = self.max_combo.max(self.combo);
                self.notes_passed += 1;
                self.score += match j {
                    Judgement::Marv | Judgement::Perfect => 300,
                    Judgement::Great => 200,
                    Judgement::Good => 100,
                    Judgement::Bad => 50,
                    _ => 0,
                };
            }
        }
    }
}
