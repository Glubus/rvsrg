//! Inspector submodule - Menu elements

use super::common::*;
use crate::models::skin::Skin;
use egui::Ui;

pub fn edit_song_button(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.menus.song_select.song_button.size.x,
        &mut skin.menus.song_select.song_button.size.y,
    );

    section_header(ui, "🎨 Normal State");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin.menus.song_select.song_button.background_color,
    );
    changed |= color_edit(
        ui,
        "Text",
        &mut skin.menus.song_select.song_button.text_color,
    );
    changed |= color_edit(
        ui,
        "Border",
        &mut skin.menus.song_select.song_button.border_color,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Button Image",
        &mut skin.menus.song_select.song_button.image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_song_button_selected(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "🎨 Selected State");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin.menus.song_select.song_button.selected_background_color,
    );
    changed |= color_edit(
        ui,
        "Text",
        &mut skin.menus.song_select.song_button.selected_text_color,
    );
    changed |= color_edit(
        ui,
        "Border",
        &mut skin.menus.song_select.song_button.selected_border_color,
    );

    section_header(ui, "🎨 Hover State");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin.menus.song_select.song_button.hover_background_color,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Selected Image",
        &mut skin.menus.song_select.song_button.selected_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_difficulty_button(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.menus.song_select.difficulty_button.size.x,
        &mut skin.menus.song_select.difficulty_button.size.y,
    );

    section_header(ui, "🎨 Normal State");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin.menus.song_select.difficulty_button.background_color,
    );
    changed |= color_edit(
        ui,
        "Text",
        &mut skin.menus.song_select.difficulty_button.text_color,
    );

    section_header(ui, "🎨 Selected State");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin
            .menus
            .song_select
            .difficulty_button
            .selected_background_color,
    );
    changed |= color_edit(
        ui,
        "Text",
        &mut skin.menus.song_select.difficulty_button.selected_text_color,
    );

    section_header(ui, "🖼️ Images");
    changed |= image_picker(
        ui,
        "Button Image",
        &mut skin.menus.song_select.difficulty_button.image,
        Some(&skin.base_path),
    );
    changed |= image_picker(
        ui,
        "Selected Image",
        &mut skin.menus.song_select.difficulty_button.selected_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_search_bar(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.menus.song_select.search_bar.size.x,
        &mut skin.menus.song_select.search_bar.size.y,
    );

    section_header(ui, "🎨 Colors");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin.menus.song_select.search_bar.background_color,
    );
    changed |= color_edit(
        ui,
        "Active BG",
        &mut skin.menus.song_select.search_bar.active_background_color,
    );
    changed |= color_edit(
        ui,
        "Text",
        &mut skin.menus.song_select.search_bar.text_color,
    );
    changed |= color_edit(
        ui,
        "Placeholder",
        &mut skin.menus.song_select.search_bar.placeholder_color,
    );
    changed |= color_edit(
        ui,
        "Border",
        &mut skin.menus.song_select.search_bar.border_color,
    );
    changed |= color_edit(
        ui,
        "Active Border",
        &mut skin.menus.song_select.search_bar.active_border_color,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Background",
        &mut skin.menus.song_select.search_bar.image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_search_panel(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.menus.song_select.search_panel.size.x,
        &mut skin.menus.song_select.search_panel.size.y,
    );

    section_header(ui, "🎨 Colors");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin.menus.song_select.search_panel.background_color,
    );
    changed |= color_edit(
        ui,
        "Border",
        &mut skin.menus.song_select.search_panel.border_color,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Background Image",
        &mut skin.menus.song_select.search_panel.background_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_beatmap_info(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.menus.song_select.beatmap_info.size.x,
        &mut skin.menus.song_select.beatmap_info.size.y,
    );

    section_header(ui, "🎨 Colors");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin.menus.song_select.beatmap_info.background_color,
    );
    changed |= color_edit(
        ui,
        "Text",
        &mut skin.menus.song_select.beatmap_info.text_color,
    );
    changed |= color_edit(
        ui,
        "Secondary Text",
        &mut skin.menus.song_select.beatmap_info.secondary_text_color,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Background Image",
        &mut skin.menus.song_select.beatmap_info.background_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_leaderboard(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "📐 Size");
    changed |= size_edit(
        ui,
        &mut skin.menus.song_select.leaderboard.size.x,
        &mut skin.menus.song_select.leaderboard.size.y,
    );

    section_header(ui, "🎨 Colors");
    changed |= color_edit(
        ui,
        "Background",
        &mut skin.menus.song_select.leaderboard.background_color,
    );
    changed |= color_edit(
        ui,
        "Text",
        &mut skin.menus.song_select.leaderboard.text_color,
    );
    changed |= color_edit(
        ui,
        "Entry BG",
        &mut skin.menus.song_select.leaderboard.entry_background_color,
    );
    changed |= color_edit(
        ui,
        "Entry Selected",
        &mut skin.menus.song_select.leaderboard.entry_selected_color,
    );

    section_header(ui, "🖼️ Image");
    changed |= image_picker(
        ui,
        "Background Image",
        &mut skin.menus.song_select.leaderboard.background_image,
        Some(&skin.base_path),
    );

    changed
}

pub fn edit_panel_style(ui: &mut Ui, skin: &mut Skin) -> bool {
    let mut changed = false;

    section_header(ui, "🎨 Panel Colors");
    changed |= color_edit(ui, "Background", &mut skin.menus.panels.background);
    changed |= color_edit(ui, "Secondary", &mut skin.menus.panels.secondary);
    changed |= color_edit(ui, "Border", &mut skin.menus.panels.border);
    changed |= color_edit(ui, "Accent", &mut skin.menus.panels.accent);
    changed |= color_edit(ui, "Accent Dim", &mut skin.menus.panels.accent_dim);

    section_header(ui, "📝 Text Colors");
    changed |= color_edit(ui, "Primary", &mut skin.menus.panels.text_primary);
    changed |= color_edit(ui, "Secondary", &mut skin.menus.panels.text_secondary);
    changed |= color_edit(ui, "Muted", &mut skin.menus.panels.text_muted);

    changed
}

