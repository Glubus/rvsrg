//! Inspector submodule - Playfield elements (notes, holds, bursts, mines, receptors)

use super::common::*;
use crate::models::skin::Skin;
use egui::{DragValue, Ui};

pub fn edit_notes_default(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Note Color", &mut skin.gameplay.notes.note.color);

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.gameplay.playfield.note_size.x,
        &mut skin.gameplay.playfield.note_size.y,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Image",
        &mut skin.gameplay.notes.note.image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_hold_body(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Body Color", &mut skin.gameplay.notes.hold.color);

    section_header(ui, "📐 Size");
    ui.horizontal(|ui| {
        ui.label("Width");
        changed |= ui
            .add(DragValue::new(&mut skin.gameplay.notes.hold.body_width).speed(1.0))
            .changed();
    });

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Body Image",
        &mut skin.gameplay.notes.hold.body_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_hold_end(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.gameplay.notes.hold.end_size.x,
        &mut skin.gameplay.notes.hold.end_size.y,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "End Image",
        &mut skin.gameplay.notes.hold.end_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_burst_body(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Body Color", &mut skin.gameplay.notes.burst.color);

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.gameplay.notes.burst.body_size.x,
        &mut skin.gameplay.notes.burst.body_size.y,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Body Image",
        &mut skin.gameplay.notes.burst.body_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_burst_end(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.gameplay.notes.burst.end_size.x,
        &mut skin.gameplay.notes.burst.end_size.y,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "End Image",
        &mut skin.gameplay.notes.burst.end_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_mines(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Mine Color", &mut skin.gameplay.notes.mine.color);

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.gameplay.notes.mine.size.x,
        &mut skin.gameplay.notes.mine.size.y,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Mine Image",
        &mut skin.gameplay.notes.mine.image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_receptors_default(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Receptor Color", &mut skin.gameplay.receptors.color);
    changed |= color_edit(
        ui,
        "Pressed Color",
        &mut skin.gameplay.receptors.pressed_color,
    );

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.gameplay.playfield.receptor_size.x,
        &mut skin.gameplay.playfield.receptor_size.y,
    );
    ui.horizontal(|ui| {
        ui.label("Spacing");
        changed |= ui
            .add(DragValue::new(&mut skin.gameplay.playfield.receptor_spacing).speed(0.5))
            .changed();
    });

    section_header(ui, "🖼️ Images");
    changed |= image_picker(
        ui,
        "Normal Image",
        &mut skin.gameplay.receptors.image,
        Some(&skin.base_path),
    );
    changed |= image_picker(
        ui,
        "Pressed Image",
        &mut skin.gameplay.receptors.pressed_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_hit_bar(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📍 Position");
    changed |= position_edit(
        ui,
        &mut skin.hud.hit_bar.position.x,
        &mut skin.hud.hit_bar.position.y,
    );

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.hud.hit_bar.size.x,
        &mut skin.hud.hit_bar.size.y,
    );
    ui.horizontal(|ui| {
        ui.label("Scale");
        changed |= ui
            .add(DragValue::new(&mut skin.hud.hit_bar.scale).speed(0.5))
            .changed();
    });

    section_header(ui, "🎨 Colors");
    changed |= color_edit(ui, "Bar Color", &mut skin.hud.hit_bar.bar_color);
    changed |= color_edit(ui, "Indicator Color", &mut skin.hud.hit_bar.indicator_color);

    section_header(ui, "👁️ Visibility");
    changed |= ui
        .checkbox(&mut skin.hud.hit_bar.visible, "Visible")
        .changed();

    changed
}

pub fn edit_playfield_position(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📍 Position");
    changed |= position_edit(
        ui,
        &mut skin.gameplay.playfield.position.x,
        &mut skin.gameplay.playfield.position.y,
    );

    section_header(ui, "📐 Column Settings");
    ui.horizontal(|ui| {
        ui.label("Column Width");
        changed |= ui
            .add(DragValue::new(&mut skin.gameplay.playfield.column_width).speed(1.0))
            .changed();
    });

    changed
}
