use super::layout::{EditorScene, SkinEditorState};
use crate::models::skin::Skin;
use egui::{ComboBox, RichText, Ui};

pub struct AssetBrowser;

impl AssetBrowser {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut SkinEditorState, _skin: &mut Skin) {
        ui.label("Current Scene");
        ComboBox::from_id_salt("scene_selector_right")
            .selected_text(state.current_scene.name())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.current_scene,
                    EditorScene::Gameplay4K,
                    "Gameplay (4K)",
                );
                ui.selectable_value(
                    &mut state.current_scene,
                    EditorScene::Gameplay7K,
                    "Gameplay (7K)",
                );
                ui.selectable_value(
                    &mut state.current_scene,
                    EditorScene::SongSelect,
                    "Song Select",
                );
                ui.selectable_value(
                    &mut state.current_scene,
                    EditorScene::ResultScreen,
                    "Result Screen",
                );
            });

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(5.0);

        ui.label(RichText::new("Scene Hierarchy").strong());
        egui::ScrollArea::vertical().show(ui, |ui| {
            // ========== PLAYFIELD ==========
            ui.collapsing("🎮 Playfield", |ui| {
                ui.collapsing("📝 Notes (Defaults)", |ui| {
                    self.item(ui, state, "Notes - Default");
                });
                ui.collapsing("🔗 Holds (LN)", |ui| {
                    self.item(ui, state, "Hold - Body");
                    self.item(ui, state, "Hold - End");
                });
                ui.collapsing("⚡ Bursts", |ui| {
                    self.item(ui, state, "Burst - Body");
                    self.item(ui, state, "Burst - End");
                });
                self.item(ui, state, "💣 Mines");
                ui.collapsing("🎯 Receptors (Defaults)", |ui| {
                    self.item(ui, state, "Receptors - Default");
                });
                self.item(ui, state, "📊 Hit Bar");
            });

            // ========== PER-COLUMN by KEYMODE ==========
            ui.collapsing("🎹 Per-Column (Keymodes)", |ui| {
                self.item(ui, state, "🎹 4K Columns");
                self.item(ui, state, "🎹 5K Columns");
                self.item(ui, state, "🎹 6K Columns");
                self.item(ui, state, "🎹 7K Columns");
            });

            // ========== HUD ==========
            ui.collapsing("📺 HUD", |ui| {
                ui.collapsing("📈 Score & Stats", |ui| {
                    self.item(ui, state, "Score Display");
                    self.item(ui, state, "Combo Counter");
                    self.item(ui, state, "Accuracy");
                    self.item(ui, state, "NPS Display");
                    ui.separator();
                    self.item(ui, state, "📝 Notes Remaining");
                    self.item(ui, state, "⚡ Scroll Speed");
                    self.item(ui, state, "⏱️ Time Left");
                });

                // Judgement Flash - the centered text when hitting notes
                ui.collapsing("⚡ Judgement Flash", |ui| {
                    self.item(ui, state, "Flash - All");
                    ui.separator();
                    self.item(ui, state, "Flash - Marvelous");
                    self.item(ui, state, "Flash - Perfect");
                    self.item(ui, state, "Flash - Great");
                    self.item(ui, state, "Flash - Good");
                    self.item(ui, state, "Flash - Bad");
                    self.item(ui, state, "Flash - Miss");
                    self.item(ui, state, "Flash - Ghost Tap");
                });

                // Judgement Panel - the stats display (SEPARATE from flash!)
                self.item(ui, state, "📋 Judgement Panel");
            });

            // ========== MENUS ==========
            ui.collapsing("📁 Menus", |ui| {
                self.item(ui, state, "Background");
                ui.collapsing("🎵 Song Select", |ui| {
                    self.item(ui, state, "Song Button");
                    self.item(ui, state, "Song Button Selected");
                    self.item(ui, state, "Difficulty Button");
                    self.item(ui, state, "Search Bar");
                    self.item(ui, state, "Search Panel");
                    self.item(ui, state, "Beatmap Info");
                    self.item(ui, state, "Leaderboard");
                });
                self.item(ui, state, "🎨 Panel Style");
            });

            // ========== GENERAL ==========
            ui.collapsing("⚙️ General", |ui| {
                self.item(ui, state, "Skin Info");
                self.item(ui, state, "Font");
            });
        });
    }

    fn item(&self, ui: &mut Ui, state: &mut SkinEditorState, id: &str) {
        let display_name = id.trim_start_matches(|c: char| !c.is_alphabetic() && c != '-');
        let is_selected = state.selected_element_id.as_deref() == Some(id);
        if ui.selectable_label(is_selected, display_name).clicked() {
            state.selected_element_id = Some(id.to_string());
        }
    }
}

