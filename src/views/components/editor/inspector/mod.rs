//! Inspector module - Edit skin elements
//!
//! Submodules:
//! - common: shared utilities (color_edit, position_edit, etc.)
//! - playfield: notes, holds, bursts, mines, receptors
//! - hud: score, combo, accuracy, nps  
//! - judgement: flash levels (all + individual)
//! - menus: song select elements
//! - general: skin info, font
//! - columns: per-column editing (4K, 5K, 6K, 7K)

mod columns;
mod common;
mod general;
mod hud;
mod judgement;
mod menus;
mod playfield;

use super::layout::SkinEditorState;
use crate::models::skin::Skin;
use egui::{Color32, RichText, Ui};

pub struct ElementInspector;

impl ElementInspector {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut SkinEditorState, skin: &mut Skin) -> bool {
        let mut changed = false;

        if let Some(id) = &state.selected_element_id.clone() {
            ui.label(RichText::new(format!("✏️ {}", id)).strong().size(16.0));
            ui.add_space(8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                changed |= self.edit_element(ui, &id, skin);
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(RichText::new("No Selection").color(Color32::GRAY));
                ui.label("Select an element in the browser\nor click on the preview.");
            });
        }

        changed
    }

    fn edit_element(&mut self, ui: &mut Ui, id: &str, skin: &mut Skin) -> bool {
        match id {
            // ========== PLAYFIELD ==========
            "Notes - Default" => playfield::edit_notes_default(ui, skin),
            "Hold - Body" => playfield::edit_hold_body(ui, skin),
            "Hold - End" => playfield::edit_hold_end(ui, skin),
            "Burst - Body" => playfield::edit_burst_body(ui, skin),
            "Burst - End" => playfield::edit_burst_end(ui, skin),
            "💣 Mines" => playfield::edit_mines(ui, skin),
            "Receptors - Default" => playfield::edit_receptors_default(ui, skin),
            "📊 Hit Bar" => playfield::edit_hit_bar(ui, skin),
            "🎮 Playfield" => playfield::edit_playfield_position(ui, skin),

            // ========== PER-COLUMN (by keymode) ==========
            "🎹 4K Columns" => columns::edit_4k_columns(ui, skin),
            "🎹 5K Columns" => columns::edit_5k_columns(ui, skin),
            "🎹 6K Columns" => columns::edit_6k_columns(ui, skin),
            "🎹 7K Columns" => columns::edit_7k_columns(ui, skin),

            // ========== HUD ==========
            "Score Display" => hud::edit_score(ui, skin),
            "Combo Counter" => hud::edit_combo(ui, skin),
            "Accuracy" => hud::edit_accuracy(ui, skin),
            "NPS Display" => hud::edit_nps(ui, skin),
            "📝 Notes Remaining" => judgement::edit_notes_remaining(ui, skin),
            "⚡ Scroll Speed" => judgement::edit_scroll_speed(ui, skin),
            "⏱️ Time Left" => judgement::edit_time_left(ui, skin),

            // ========== JUDGEMENT ==========
            "Flash - All" => judgement::edit_flash_all(ui, skin),
            "Flash - Marvelous" => judgement::edit_marvelous(ui, skin),
            "Flash - Perfect" => judgement::edit_perfect(ui, skin),
            "Flash - Great" => judgement::edit_great(ui, skin),
            "Flash - Good" => judgement::edit_good(ui, skin),
            "Flash - Bad" => judgement::edit_bad(ui, skin),
            "Flash - Miss" => judgement::edit_miss(ui, skin),
            "Flash - Ghost Tap" => judgement::edit_ghost_tap(ui, skin),
            "📋 Judgement Panel" => judgement::edit_judgement_panel(ui, skin),

            // ========== MENUS ==========
            "Song Button" => menus::edit_song_button(ui, skin),
            "Song Button Selected" => menus::edit_song_button_selected(ui, skin),
            "Difficulty Button" => menus::edit_difficulty_button(ui, skin),
            "Search Bar" => menus::edit_search_bar(ui, skin),
            "Search Panel" => menus::edit_search_panel(ui, skin),
            "Beatmap Info" => menus::edit_beatmap_info(ui, skin),
            "Leaderboard" => menus::edit_leaderboard(ui, skin),
            "🎨 Panel Style" => menus::edit_panel_style(ui, skin),

            // ========== GENERAL ==========
            "Skin Info" => general::edit_skin_info(ui, skin),
            "Font" => general::edit_font(ui, skin),

            _ => {
                ui.label("Select an element to edit its properties.");
                false
            }
        }
    }
}
