//! Inspector submodule - Judgement Flash and Panel (now SEPARATE)

use super::common::*;
use crate::models::skin::Skin;
use egui::{DragValue, Ui};

/// Edit ALL judgement flashes at once (position + size for all)
pub fn edit_flash_all(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📍 Default Position (applies to ALL flashes)");

    // Use marv as reference
    let ref_x = skin.hud.judgement.marv.position.x;
    let ref_y = skin.hud.judgement.marv.position.y;
    let ref_w = skin.hud.judgement.marv.size.x;
    let ref_h = skin.hud.judgement.marv.size.y;

    let mut new_x = ref_x;
    let mut new_y = ref_y;
    let mut new_w = ref_w;
    let mut new_h = ref_h;

    let pos_changed = position_edit(ui, &mut new_x, &mut new_y);

    section_header(ui, "📐 Default Size (applies to ALL flashes)");
    let size_changed = size_edit(ui, &mut new_w, &mut new_h);

    if pos_changed {
        let dx = new_x - ref_x;
        let dy = new_y - ref_y;

        skin.hud.judgement.marv.position.x += dx;
        skin.hud.judgement.marv.position.y += dy;
        skin.hud.judgement.perfect.position.x += dx;
        skin.hud.judgement.perfect.position.y += dy;
        skin.hud.judgement.great.position.x += dx;
        skin.hud.judgement.great.position.y += dy;
        skin.hud.judgement.good.position.x += dx;
        skin.hud.judgement.good.position.y += dy;
        skin.hud.judgement.bad.position.x += dx;
        skin.hud.judgement.bad.position.y += dy;
        skin.hud.judgement.miss.position.x += dx;
        skin.hud.judgement.miss.position.y += dy;
        skin.hud.judgement.ghost_tap.position.x += dx;
        skin.hud.judgement.ghost_tap.position.y += dy;

        changed = true;
    }

    if size_changed {
        skin.hud.judgement.marv.size.x = new_w;
        skin.hud.judgement.marv.size.y = new_h;
        skin.hud.judgement.perfect.size.x = new_w;
        skin.hud.judgement.perfect.size.y = new_h;
        skin.hud.judgement.great.size.x = new_w;
        skin.hud.judgement.great.size.y = new_h;
        skin.hud.judgement.good.size.x = new_w;
        skin.hud.judgement.good.size.y = new_h;
        skin.hud.judgement.bad.size.x = new_w;
        skin.hud.judgement.bad.size.y = new_h;
        skin.hud.judgement.miss.size.x = new_w;
        skin.hud.judgement.miss.size.y = new_h;
        skin.hud.judgement.ghost_tap.size.x = new_w;
        skin.hud.judgement.ghost_tap.size.y = new_h;

        changed = true;
    }

    section_header(ui, "⏱️ Timing Indicator");
    changed |= ui
        .checkbox(&mut skin.hud.judgement.show_timing, "Show +/- (early/late)")
        .changed();
    hint(ui, "- = early hit, + = late hit");

    hint(ui, "This moves/resizes all judgement flashes together");

    changed
}

/// Edit a single judgement flash
fn edit_judgement_flash(
    ui: &mut Ui,
    name: &str,
    label: &mut String,
    color: &mut [f32; 4],
    pos_x: &mut f32,
    pos_y: &mut f32,
    size_x: &mut f32,
    size_y: &mut f32,
    visible: &mut bool,
    image: &mut Option<String>,
    dest_folder: Option<&std::path::Path>,
) -> bool {
    let mut changed = false;

    section_header(ui, "📝 Label");
    ui.horizontal(|ui| {
        ui.label("Text");
        changed |= ui.text_edit_singleline(label).changed();
    });
    hint(ui, &format!("Default: \"{}\"", name));

    section_header(ui, "📍 Position");
    changed |= position_edit(ui, pos_x, pos_y);

    section_header(ui, "📐 Size");
    changed |= size_edit(ui, size_x, size_y);

    section_header(ui, "🎨 Color");
    changed |= color_edit(ui, "Flash Color", color);

    section_header(ui, "🖼️ Image (Optional)");
    changed |= image_picker(ui, "Replace with image", image, dest_folder);
    hint(ui, "If set, image replaces text");

    section_header(ui, "👁️ Visibility");
    changed |= ui.checkbox(visible, "Visible").changed();

    changed
}

pub fn edit_marvelous(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_judgement_flash(
        ui,
        "Marvelous",
        &mut skin.hud.judgement.marv.label,
        &mut skin.hud.judgement.marv.color,
        &mut skin.hud.judgement.marv.position.x,
        &mut skin.hud.judgement.marv.position.y,
        &mut skin.hud.judgement.marv.size.x,
        &mut skin.hud.judgement.marv.size.y,
        &mut skin.hud.judgement.marv.visible,
        &mut skin.hud.judgement.marv.image,
        Some(&skin.base_path),
    )
}

pub fn edit_perfect(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_judgement_flash(
        ui,
        "Perfect",
        &mut skin.hud.judgement.perfect.label,
        &mut skin.hud.judgement.perfect.color,
        &mut skin.hud.judgement.perfect.position.x,
        &mut skin.hud.judgement.perfect.position.y,
        &mut skin.hud.judgement.perfect.size.x,
        &mut skin.hud.judgement.perfect.size.y,
        &mut skin.hud.judgement.perfect.visible,
        &mut skin.hud.judgement.perfect.image,
        Some(&skin.base_path),
    )
}

pub fn edit_great(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_judgement_flash(
        ui,
        "Great",
        &mut skin.hud.judgement.great.label,
        &mut skin.hud.judgement.great.color,
        &mut skin.hud.judgement.great.position.x,
        &mut skin.hud.judgement.great.position.y,
        &mut skin.hud.judgement.great.size.x,
        &mut skin.hud.judgement.great.size.y,
        &mut skin.hud.judgement.great.visible,
        &mut skin.hud.judgement.great.image,
        Some(&skin.base_path),
    )
}

pub fn edit_good(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_judgement_flash(
        ui,
        "Good",
        &mut skin.hud.judgement.good.label,
        &mut skin.hud.judgement.good.color,
        &mut skin.hud.judgement.good.position.x,
        &mut skin.hud.judgement.good.position.y,
        &mut skin.hud.judgement.good.size.x,
        &mut skin.hud.judgement.good.size.y,
        &mut skin.hud.judgement.good.visible,
        &mut skin.hud.judgement.good.image,
        Some(&skin.base_path),
    )
}

pub fn edit_bad(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_judgement_flash(
        ui,
        "Bad",
        &mut skin.hud.judgement.bad.label,
        &mut skin.hud.judgement.bad.color,
        &mut skin.hud.judgement.bad.position.x,
        &mut skin.hud.judgement.bad.position.y,
        &mut skin.hud.judgement.bad.size.x,
        &mut skin.hud.judgement.bad.size.y,
        &mut skin.hud.judgement.bad.visible,
        &mut skin.hud.judgement.bad.image,
        Some(&skin.base_path),
    )
}

pub fn edit_miss(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_judgement_flash(
        ui,
        "Miss",
        &mut skin.hud.judgement.miss.label,
        &mut skin.hud.judgement.miss.color,
        &mut skin.hud.judgement.miss.position.x,
        &mut skin.hud.judgement.miss.position.y,
        &mut skin.hud.judgement.miss.size.x,
        &mut skin.hud.judgement.miss.size.y,
        &mut skin.hud.judgement.miss.visible,
        &mut skin.hud.judgement.miss.image,
        Some(&skin.base_path),
    )
}

pub fn edit_ghost_tap(ui: &mut Ui, skin: &mut Skin) -> bool {
    edit_judgement_flash(
        ui,
        "Ghost Tap",
        &mut skin.hud.judgement.ghost_tap.label,
        &mut skin.hud.judgement.ghost_tap.color,
        &mut skin.hud.judgement.ghost_tap.position.x,
        &mut skin.hud.judgement.ghost_tap.position.y,
        &mut skin.hud.judgement.ghost_tap.size.x,
        &mut skin.hud.judgement.ghost_tap.size.y,
        &mut skin.hud.judgement.ghost_tap.visible,
        &mut skin.hud.judgement.ghost_tap.image,
        Some(&skin.base_path),
    )
}

/// Edit Judgement Panel - COMPLETELY SEPARATE from Flash!
pub fn edit_judgement_panel(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;
    let panel = &mut skin.hud.judgement_panel;

    section_header(ui, "📍 Position");
    changed |= position_edit(ui, &mut panel.position.x, &mut panel.position.y);

    section_header(ui, "📐 Size");
    changed |= size_edit(ui, &mut panel.size.x, &mut panel.size.y);
    ui.horizontal(|ui| {
        ui.label("Text Scale");
        changed |= ui
            .add(DragValue::new(&mut panel.text_scale).speed(0.5))
            .changed();
    });

    section_header(ui, "🎨 Panel Colors (separate from Flash!)");
    changed |= color_edit(ui, "Marvelous", &mut panel.marv_color);
    changed |= color_edit(ui, "Perfect", &mut panel.perfect_color);
    changed |= color_edit(ui, "Great", &mut panel.great_color);
    changed |= color_edit(ui, "Good", &mut panel.good_color);
    changed |= color_edit(ui, "Bad", &mut panel.bad_color);
    changed |= color_edit(ui, "Miss", &mut panel.miss_color);
    changed |= color_edit(ui, "Ghost Tap", &mut panel.ghost_tap_color);

    section_header(ui, "👁️ Visibility");
    changed |= ui.checkbox(&mut panel.visible, "Visible").changed();

    changed
}

/// Edit Notes Remaining display
pub fn edit_notes_remaining(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;
    let cfg = &mut skin.hud.notes_remaining;

    section_header(ui, "📍 Position");
    changed |= position_edit(ui, &mut cfg.position.x, &mut cfg.position.y);

    section_header(ui, "📐 Size & Scale");
    changed |= size_edit(ui, &mut cfg.size.x, &mut cfg.size.y);
    ui.horizontal(|ui| {
        ui.label("Text Scale");
        changed |= ui.add(DragValue::new(&mut cfg.scale).speed(0.5)).changed();
    });

    section_header(ui, "🎨 Color");
    changed |= color_edit(ui, "Text Color", &mut cfg.color);

    section_header(ui, "📝 Format");
    ui.horizontal(|ui| {
        ui.label("Format");
        changed |= ui.text_edit_singleline(&mut cfg.format).changed();
    });
    hint(ui, "Use {remaining} as placeholder");

    section_header(ui, "👁️ Visibility");
    changed |= ui.checkbox(&mut cfg.visible, "Visible").changed();

    changed
}

/// Edit Scroll Speed display
pub fn edit_scroll_speed(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;
    let cfg = &mut skin.hud.scroll_speed;

    section_header(ui, "📍 Position");
    changed |= position_edit(ui, &mut cfg.position.x, &mut cfg.position.y);

    section_header(ui, "📐 Size & Scale");
    changed |= size_edit(ui, &mut cfg.size.x, &mut cfg.size.y);
    ui.horizontal(|ui| {
        ui.label("Text Scale");
        changed |= ui.add(DragValue::new(&mut cfg.scale).speed(0.5)).changed();
    });

    section_header(ui, "🎨 Color");
    changed |= color_edit(ui, "Text Color", &mut cfg.color);

    section_header(ui, "📝 Format");
    ui.horizontal(|ui| {
        ui.label("Format");
        changed |= ui.text_edit_singleline(&mut cfg.format).changed();
    });
    hint(ui, "Use {speed} as placeholder");

    section_header(ui, "👁️ Visibility");
    changed |= ui.checkbox(&mut cfg.visible, "Visible").changed();

    changed
}

/// Edit Time Left / Progress display
pub fn edit_time_left(ui: &mut Ui, skin: &mut Skin) -> bool {
    use crate::models::skin::hud::TimeDisplayMode;

    let mut changed = false;
    let cfg = &mut skin.hud.time_left;

    section_header(ui, "🎛️ Display Mode");
    egui::ComboBox::from_label("Mode")
        .selected_text(match cfg.mode {
            TimeDisplayMode::Bar => "Bar",
            TimeDisplayMode::Circle => "Circle",
            TimeDisplayMode::Text => "Text",
        })
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(
                    cfg.mode == TimeDisplayMode::Bar,
                    "Bar (horizontal progress)",
                )
                .clicked()
            {
                cfg.mode = TimeDisplayMode::Bar;
                changed = true;
            }
            if ui
                .selectable_label(cfg.mode == TimeDisplayMode::Circle, "Circle (watch-like)")
                .clicked()
            {
                cfg.mode = TimeDisplayMode::Circle;
                changed = true;
            }
            if ui
                .selectable_label(cfg.mode == TimeDisplayMode::Text, "Text (minutes:seconds)")
                .clicked()
            {
                cfg.mode = TimeDisplayMode::Text;
                changed = true;
            }
        });

    section_header(ui, "📍 Position");
    changed |= position_edit(ui, &mut cfg.position.x, &mut cfg.position.y);

    section_header(ui, "📐 Size");
    changed |= size_edit(ui, &mut cfg.size.x, &mut cfg.size.y);

    match cfg.mode {
        TimeDisplayMode::Bar => {
            section_header(ui, "🎨 Bar Colors");
            changed |= color_edit(ui, "Progress", &mut cfg.progress_color);
            changed |= color_edit(ui, "Background", &mut cfg.background_color);
            changed |= color_edit(ui, "Border", &mut cfg.border_color);

            ui.horizontal(|ui| {
                ui.label("Border Width");
                changed |= ui
                    .add(egui::DragValue::new(&mut cfg.border_width).speed(0.5))
                    .changed();
            });

            section_header(ui, "🖼️ Images (Optional)");
            changed |= image_picker(
                ui,
                "Background",
                &mut cfg.background_image,
                Some(&skin.base_path),
            );
            changed |= image_picker(
                ui,
                "Progress Fill",
                &mut cfg.progress_image,
                Some(&skin.base_path),
            );
        }
        TimeDisplayMode::Circle => {
            section_header(ui, "🎨 Circle Colors");
            changed |= color_edit(ui, "Progress", &mut cfg.progress_color);
            changed |= color_edit(ui, "Background", &mut cfg.background_color);
            changed |= color_edit(ui, "Border", &mut cfg.border_color);

            ui.horizontal(|ui| {
                ui.label("Radius");
                changed |= ui
                    .add(egui::DragValue::new(&mut cfg.circle_radius).speed(1.0))
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("Border Width");
                changed |= ui
                    .add(egui::DragValue::new(&mut cfg.border_width).speed(0.5))
                    .changed();
            });

            section_header(ui, "🖼️ Images (Optional)");
            changed |= image_picker(
                ui,
                "Circle Image",
                &mut cfg.circle_image,
                Some(&skin.base_path),
            );
        }
        TimeDisplayMode::Text => {
            section_header(ui, "🎨 Text Color");
            changed |= color_edit(ui, "Text", &mut cfg.text_color);

            ui.horizontal(|ui| {
                ui.label("Text Scale");
                changed |= ui
                    .add(egui::DragValue::new(&mut cfg.text_scale).speed(0.5))
                    .changed();
            });

            section_header(ui, "📝 Format");
            ui.horizontal(|ui| {
                ui.label("Format");
                changed |= ui.text_edit_singleline(&mut cfg.format).changed();
            });
            hint(ui, "Use {elapsed}, {remaining}, {total}, {percent}");
        }
    }

    section_header(ui, "👁️ Visibility");
    changed |= ui.checkbox(&mut cfg.visible, "Visible").changed();

    changed
}

