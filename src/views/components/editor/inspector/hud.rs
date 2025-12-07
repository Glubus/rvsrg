//! Inspector submodule - HUD elements (score, combo, accuracy, nps)

use super::common::*;
use crate::models::skin::Skin;
use egui::{DragValue, Ui};

pub fn edit_score(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📍 Position");
    changed |= position_edit(
        ui,
        &mut skin.hud.score.position.x,
        &mut skin.hud.score.position.y,
    );

    section_header(ui, "📐 Size & Scale");
    changed |= size_edit(ui, &mut skin.hud.score.size.x, &mut skin.hud.score.size.y);
    ui.horizontal(|ui| {
        ui.label("Text Scale");
        changed |= ui
            .add(DragValue::new(&mut skin.hud.score.scale).speed(0.5))
            .changed();
    });

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Text Color", &mut skin.hud.score.color);

    section_header(ui, "📝 Format");
    ui.horizontal(|ui| {
        ui.label("Format");
        changed |= ui
            .text_edit_singleline(&mut skin.hud.score.format)
            .changed();
    });
    hint(ui, "Use {score} as placeholder");

    section_header(ui, "🖼️ Image (Optional)");
    changed |= image_picker(
        ui,
        "Background",
        &mut skin.hud.score.image,
        Some(&skin.base_path),
    );

    section_header(ui, "👁️ Visibility");
    changed |= ui
        .checkbox(&mut skin.hud.score.visible, "Visible")
        .changed();

    changed
}

pub fn edit_combo(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📍 Position");
    changed |= position_edit(
        ui,
        &mut skin.hud.combo.position.x,
        &mut skin.hud.combo.position.y,
    );

    section_header(ui, "📐 Size & Scale");
    changed |= size_edit(ui, &mut skin.hud.combo.size.x, &mut skin.hud.combo.size.y);
    ui.horizontal(|ui| {
        ui.label("Text Scale");
        changed |= ui
            .add(DragValue::new(&mut skin.hud.combo.scale).speed(0.5))
            .changed();
    });

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Text Color", &mut skin.hud.combo.color);

    section_header(ui, "📝 Format");
    ui.horizontal(|ui| {
        ui.label("Format");
        changed |= ui
            .text_edit_singleline(&mut skin.hud.combo.format)
            .changed();
    });
    hint(ui, "Use {combo} as placeholder");

    section_header(ui, "🖼️ Image (Optional)");
    changed |= image_picker(
        ui,
        "Background",
        &mut skin.hud.combo.image,
        Some(&skin.base_path),
    );

    section_header(ui, "👁️ Visibility");
    changed |= ui
        .checkbox(&mut skin.hud.combo.visible, "Visible")
        .changed();

    changed
}

pub fn edit_accuracy(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📍 Position");
    changed |= position_edit(
        ui,
        &mut skin.hud.accuracy.position.x,
        &mut skin.hud.accuracy.position.y,
    );

    section_header(ui, "📐 Size & Scale");
    changed |= size_edit(
        ui,
        &mut skin.hud.accuracy.size.x,
        &mut skin.hud.accuracy.size.y,
    );
    ui.horizontal(|ui| {
        ui.label("Text Scale");
        changed |= ui
            .add(DragValue::new(&mut skin.hud.accuracy.scale).speed(0.5))
            .changed();
    });

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Text Color", &mut skin.hud.accuracy.color);

    section_header(ui, "📝 Format");
    ui.horizontal(|ui| {
        ui.label("Format");
        changed |= ui
            .text_edit_singleline(&mut skin.hud.accuracy.format)
            .changed();
    });
    hint(ui, "Use {accuracy} as placeholder");

    section_header(ui, "🖼️ Image (Optional)");
    changed |= image_picker(
        ui,
        "Background",
        &mut skin.hud.accuracy.image,
        Some(&skin.base_path),
    );

    section_header(ui, "👁️ Visibility");
    changed |= ui
        .checkbox(&mut skin.hud.accuracy.visible, "Visible")
        .changed();

    changed
}

pub fn edit_nps(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📍 Position");
    changed |= position_edit(
        ui,
        &mut skin.hud.nps.position.x,
        &mut skin.hud.nps.position.y,
    );

    section_header(ui, "📐 Size & Scale");
    changed |= size_edit(ui, &mut skin.hud.nps.size.x, &mut skin.hud.nps.size.y);
    ui.horizontal(|ui| {
        ui.label("Text Scale");
        changed |= ui
            .add(DragValue::new(&mut skin.hud.nps.scale).speed(0.5))
            .changed();
    });

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Text Color", &mut skin.hud.nps.color);

    section_header(ui, "📝 Format");
    ui.horizontal(|ui| {
        ui.label("Format");
        changed |= ui.text_edit_singleline(&mut skin.hud.nps.format).changed();
    });
    hint(ui, "Use {nps} as placeholder");

    section_header(ui, "🖼️ Image (Optional)");
    changed |= image_picker(
        ui,
        "Background",
        &mut skin.hud.nps.image,
        Some(&skin.base_path),
    );

    section_header(ui, "👁️ Visibility");
    changed |= ui.checkbox(&mut skin.hud.nps.visible, "Visible").changed();

    changed
}

