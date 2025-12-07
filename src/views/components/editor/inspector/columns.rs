//! Inspector submodule - Per-column editing for notes and receptors
//!
//! Allows editing images/colors for each column in a specific keymode (4K, 5K, 6K, 7K)

use super::common::*;
use crate::models::skin::Skin;
use egui::Ui;

/// Edit columns for a specific keymode
/// Returns true if anything changed
pub fn edit_columns(ui: &mut Ui, skin: &mut Skin, keymode: usize) -> bool {
    let mut changed = false;

    // Get or create the keymode config from the HashMap
    let km_config = skin.key_modes.entry(keymode).or_default();

    section_header(ui, &format!("🎹 {}K Column Configuration", keymode));

    // Ensure we have enough entries for each column
    while km_config.notes.len() < keymode {
        km_config.notes.push(Default::default());
    }
    while km_config.receptors.len() < keymode {
        km_config.receptors.push(Default::default());
    }

    // Edit each column
    for col in 0..keymode {
        let col_name = format!("Column {} ({}K)", col + 1, keymode);

        ui.collapsing(&col_name, |ui| {
            // Note image
            section_header(ui, "🎵 Note");
            if let Some(note_cfg) = km_config.notes.get_mut(col) {
                changed |=
                    image_picker(ui, "Note Image", &mut note_cfg.image, Some(&skin.base_path));
                changed |= color_edit(ui, "Note Color", &mut note_cfg.color);
            }

            // Receptor images
            section_header(ui, "⬇️ Receptor");
            if let Some(rec_cfg) = km_config.receptors.get_mut(col) {
                changed |= image_picker(
                    ui,
                    "Normal Image",
                    &mut rec_cfg.image,
                    Some(&skin.base_path),
                );
                changed |= image_picker(
                    ui,
                    "Pressed Image",
                    &mut rec_cfg.pressed_image,
                    Some(&skin.base_path),
                );
                changed |= color_edit(ui, "Normal Color", &mut rec_cfg.color);
                changed |= color_edit(ui, "Pressed Color", &mut rec_cfg.pressed_color);
            }
        });
    }

    ui.add_space(10.0);
    hint(ui, "Each column can have different images and colors");

    changed
}

/// Edit 4K columns
pub fn edit_4k_columns(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_columns(ui, skin, 4)
}

/// Edit 5K columns
pub fn edit_5k_columns(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_columns(ui, skin, 5)
}

/// Edit 6K columns
pub fn edit_6k_columns(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_columns(ui, skin, 6)
}

/// Edit 7K columns
pub fn edit_7k_columns(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_columns(ui, skin, 7)
}
