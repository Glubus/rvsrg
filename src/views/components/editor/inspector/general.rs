//! Inspector submodule - General settings (skin info, font)

use super::common::*;
use crate::models::skin::Skin;
use egui::Ui;

pub fn edit_skin_info(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "ℹ️ Skin Information");
    ui.horizontal(|ui| {
        ui.label("Name");
        changed |= ui.text_edit_singleline(&mut skin.general.name).changed();
    });
    ui.horizontal(|ui| {
        ui.label("Version");
        changed |= ui.text_edit_singleline(&mut skin.general.version).changed();
    });
    ui.horizontal(|ui| {
        ui.label("Author");
        changed |= ui.text_edit_singleline(&mut skin.general.author).changed();
    });

    changed
}

pub fn edit_font(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "🔤 Font");
    changed |= file_picker(
        ui,
        "Font File (.ttf)",
        &mut skin.general.font,
        Some(&skin.base_path),
        "Fonts",
        &["ttf", "otf"],
    );
    hint(ui, "Place font file in the skin folder");

    changed
}
