use super::browser::AssetBrowser;
use super::inspector::ElementInspector;
use super::viewport::GamePreviewViewport;
use crate::models::skin::Skin;
use egui::{CentralPanel, Color32, Context, DragValue, RichText, SidePanel, TopBottomPanel};

/// État global de l'éditeur de skin.
pub struct SkinEditorState {
    /// L'élément actuellement sélectionné pour inspection.
    pub selected_element_id: Option<String>,
    /// La scène simulée (Menu, Gameplay 4K, Result, etc.).
    pub current_scene: EditorScene,
    /// Texture du jeu rendue off-screen (ID Egui).
    pub game_texture_id: Option<egui::TextureId>,
    /// Résolution de la prévisualisation.
    pub preview_width: u32,
    pub preview_height: u32,
}

impl SkinEditorState {
    pub fn new() -> Self {
        Self {
            selected_element_id: None,
            current_scene: EditorScene::Gameplay4K,
            game_texture_id: None,
            preview_width: 1280,
            preview_height: 720,
        }
    }

    pub fn target_aspect_ratio(&self) -> f32 {
        self.preview_width as f32 / self.preview_height as f32
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EditorScene {
    Gameplay4K,
    Gameplay7K,
    SongSelect,
    ResultScreen,
}

impl EditorScene {
    pub fn name(&self) -> &'static str {
        match self {
            EditorScene::Gameplay4K => "Gameplay (4K)",
            EditorScene::Gameplay7K => "Gameplay (7K)",
            EditorScene::SongSelect => "Song Select",
            EditorScene::ResultScreen => "Result Screen",
        }
    }
}

pub struct SkinEditorLayout {
    pub state: SkinEditorState,
    viewport: GamePreviewViewport,
    inspector: ElementInspector,
    browser: AssetBrowser,
}

impl SkinEditorLayout {
    pub fn new() -> Self {
        Self {
            state: SkinEditorState::new(),
            viewport: GamePreviewViewport::new(),
            inspector: ElementInspector::new(),
            browser: AssetBrowser::new(),
        }
    }

    /// Appelé à chaque frame par le Renderer pour dessiner l'interface de l'éditeur.
    pub fn show(
        &mut self,
        ctx: &Context,
        skin: &mut Skin,
        game_texture: Option<egui::TextureId>,
    ) -> bool {
        self.state.game_texture_id = game_texture;
        let mut any_change = false;

        // 1. Barre de Menu (Top)
        TopBottomPanel::top("editor_top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🛠 Skin Editor").strong());
                ui.separator();

                ui.label("Resolution:");
                ui.add(
                    DragValue::new(&mut self.state.preview_width)
                        .speed(1.0)
                        .range(320..=3840)
                        .suffix("px"),
                );
                ui.label("x");
                ui.add(
                    DragValue::new(&mut self.state.preview_height)
                        .speed(1.0)
                        .range(240..=2160)
                        .suffix("px"),
                );

                ui.separator();

                if ui.button("💾 Save Skin").clicked() {
                    println!("DEBUG: Save Skin button clicked!");
                    if let Err(e) = skin.save() {
                        eprintln!("Error saving skin config: {}", e);
                    } else {
                        println!("DEBUG: Skin save returned successfully.");
                    }
                }
                if ui.button("🚪 Exit Editor").clicked() {
                    // TODO: Close event
                }
            });
        });

        // 2. Panneau de Gauche (Inspecteur : Images & Transform)
        let inspector_res = SidePanel::left("editor_inspector_panel")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.heading("Inspector");
                ui.separator();
                self.inspector.show(ui, &mut self.state, skin)
            });

        any_change |= inspector_res.inner;

        // 3. Panneau de Droite (Browser & Scène)
        SidePanel::right("editor_browser_panel")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.heading("Context");
                ui.separator();
                self.browser.show(ui, &mut self.state, skin);
            });

        // 4. Zone Centrale (Viewport "Dézoomé")
        CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style())
                .fill(Color32::from_rgb(20, 20, 20))
                .show(ui, |ui| {
                    self.viewport.show(ui, &mut self.state, skin);
                });
        });

        any_change
    }
}
