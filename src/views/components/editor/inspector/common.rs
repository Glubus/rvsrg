//! Inspector submodule - common utilities for element editing

use egui::{Color32, DragValue, RichText, Ui};

/// Helper to edit a color
pub fn color_edit(ui: &mut Ui, label: &str, color: &mut [f32; 4]) -> bool {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.color_edit_button_rgba_unmultiplied(color).changed()
    })
    .inner
}

/// Helper to edit position X/Y
pub fn position_edit(ui: &mut Ui, x: &mut f32, y: &mut f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("X");
        changed |= ui.add(DragValue::new(x).speed(1.0).suffix("px")).changed();
        ui.label("Y");
        changed |= ui.add(DragValue::new(y).speed(1.0).suffix("px")).changed();
    });
    changed
}

/// Helper to edit size W/H
pub fn size_edit(ui: &mut Ui, w: &mut f32, h: &mut f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("W");
        changed |= ui.add(DragValue::new(w).speed(1.0)).changed();
        ui.label("H");
        changed |= ui.add(DragValue::new(h).speed(1.0)).changed();
    });
    changed
}

/// Helper to pick an image file and optionally copy it to destination
pub fn image_picker(
    ui: &mut Ui,
    label: &str,
    image: &mut Option<String>,
    dest_folder: Option<&std::path::Path>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        if ui.button("📂").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg"])
                .pick_file()
            {
                // Get just the filename
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy().to_string();

                    // Copy file if destination is provided
                    if let Some(dest) = dest_folder {
                        let dest_path = dest.join(filename);
                        if let Err(e) = std::fs::copy(&path, &dest_path) {
                            eprintln!("Failed to copy image: {}", e);
                        }
                    }

                    *image = Some(filename_str);
                    changed = true;
                }
            }
        }

        let display = image
            .as_ref()
            .map(|s| s.split(['/', '\\']).last().unwrap_or(s).to_string())
            .unwrap_or_else(|| "None".to_string());

        ui.label(format!("{}: {}", label, display));

        if image.is_some() && ui.small_button("❌").clicked() {
            *image = None;
            changed = true;
        }
    });
    changed
}

/// Section header
pub fn section_header(ui: &mut Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(title).strong());
    ui.add_space(4.0);
}

/// Hint text
pub fn hint(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(Color32::GRAY));
}

/// Generic file picker with custom filter
pub fn file_picker(
    ui: &mut Ui,
    label: &str,
    path_str: &mut Option<String>,
    dest_folder: Option<&std::path::Path>,
    filter_name: &str,
    extensions: &[&str],
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        if ui.button("📂").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(filter_name, extensions)
                .pick_file()
            {
                // Get just the filename
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy().to_string();

                    // Copy file if destination is provided
                    if let Some(dest) = dest_folder {
                        let dest_path = dest.join(&filename);
                        if let Err(e) = std::fs::copy(&path, &dest_path) {
                            eprintln!("Failed to copy file: {}", e);
                        }
                    }

                    *path_str = Some(filename_str);
                    changed = true;
                }
            }
        }

        let display = path_str
            .as_ref()
            .map(|s| s.split(['/', '\\']).last().unwrap_or(s).to_string())
            .unwrap_or_else(|| "None".to_string());

        ui.label(format!("{}: {}", label, display));

        if path_str.is_some() && ui.small_button("❌").clicked() {
            *path_str = None;
            changed = true;
        }
    });

    changed
}

