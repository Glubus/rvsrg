use crate::models::settings::{HitWindowMode, SettingsState};
use log::info;

#[derive(Clone)]
pub struct SettingsSnapshot {
    pub skin: String,
    pub hit_window_mode: HitWindowMode,
    pub hit_window_value: f64,
    pub master_volume: f32,
}

impl SettingsSnapshot {
    pub fn capture(settings: &SettingsState) -> Self {
        Self {
            skin: settings.current_skin.clone(),
            hit_window_mode: settings.hit_window_mode,
            hit_window_value: settings.hit_window_value,
            master_volume: settings.master_volume,
        }
    }
}

pub struct SettingsWindowResult {
    pub request_toggle: bool,
    pub volume_changed: Option<f32>,
    pub keybinds_updated: bool,
    pub hit_window_changed: Option<(HitWindowMode, f64)>,
}

pub fn render_settings_window(
    ctx: &egui::Context,
    settings: &mut SettingsState,
    snapshot: &SettingsSnapshot,
) -> SettingsWindowResult {
    let mut request_toggle = false;
    let mut volume_changed = None;
    let mut hit_window_changed = None;
    let mut open = true;
    let mut keybinds_updated = false;

    egui::Window::new("Settings")
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Skin");
            let mut skins = vec!["default".to_string()];
            if let Ok(entries) = std::fs::read_dir("skins") {
                for entry in entries.flatten() {
                    if entry.path().is_dir()
                        && let Some(name) = entry.file_name().to_str()
                        && name != "default"
                    {
                        skins.push(name.to_string());
                    }
                }
            }
            egui::ComboBox::from_label("Skin")
                .selected_text(&settings.current_skin)
                .show_ui(ui, |ui| {
                    for skin_name in skins {
                        ui.selectable_value(
                            &mut settings.current_skin,
                            skin_name.clone(),
                            skin_name,
                        );
                    }
                });

            ui.separator();
            ui.heading("Audio");
            ui.add(
                egui::Slider::new(&mut settings.master_volume, 0.0..=1.0)
                    .text("Master Volume")
                    .step_by(0.01),
            );

            ui.add(
                egui::Slider::new(&mut settings.global_audio_offset_ms, -100.0..=100.0)
                    .text("Audio Offset (ms)")
                    .step_by(1.0),
            );
            ui.label("Adjust if notes and audio are out of sync.");

            if (settings.master_volume - snapshot.master_volume).abs() > f32::EPSILON {
                volume_changed = Some(settings.master_volume);
            }

            ui.separator();
            ui.heading("Gameplay");
            ui.horizontal(|ui| {
                if ui.button("-50").clicked() {
                    settings.scroll_speed = (settings.scroll_speed - 50.0).max(100.0);
                }
                if ui.button("-10").clicked() {
                    settings.scroll_speed = (settings.scroll_speed - 10.0).max(100.0);
                }
                ui.add(
                    egui::Slider::new(&mut settings.scroll_speed, 100.0..=1500.0)
                        .text("Scroll Speed (ms)")
                        .step_by(10.0),
                );
                if ui.button("+10").clicked() {
                    settings.scroll_speed = (settings.scroll_speed + 10.0).min(1500.0);
                }
                if ui.button("+50").clicked() {
                    settings.scroll_speed = (settings.scroll_speed + 50.0).min(1500.0);
                }
            });
            ui.label("Lower = faster notes, Higher = slower notes");

            ui.separator();
            ui.heading("Judgement");
            egui::ComboBox::from_label("Mode")
                .selected_text(match settings.hit_window_mode {
                    HitWindowMode::OsuOD => "Osu! Overall Diff",
                    HitWindowMode::EtternaJudge => "Etterna Judge",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut settings.hit_window_mode,
                        HitWindowMode::OsuOD,
                        "Osu! Overall Diff",
                    );
                    ui.selectable_value(
                        &mut settings.hit_window_mode,
                        HitWindowMode::EtternaJudge,
                        "Etterna Judge",
                    );
                });

            match settings.hit_window_mode {
                HitWindowMode::OsuOD => {
                    ui.add(
                        egui::Slider::new(&mut settings.hit_window_value, 0.0..=12.0)
                            .text("Overall Difficulty")
                            .step_by(0.1),
                    );
                }
                HitWindowMode::EtternaJudge => {
                    ui.add(
                        egui::Slider::new(&mut settings.hit_window_value, 1.0..=15.0)
                            .text("Judge")
                            .step_by(1.0),
                    );
                    settings.hit_window_value = settings.hit_window_value.round();
                }
            }

            ui.separator();
            ui.heading("Keybinds");
            ui.label("Choose a keymode below, then press the required keys in order.");
            let mut columns: Vec<_> = settings.keybinds.keys().cloned().collect();
            columns.sort_by_key(|key| key.parse::<usize>().unwrap_or(0));
            for column in columns {
                let Ok(column_count) = column.parse::<usize>() else {
                    continue;
                };
                let existing = settings
                    .keybinds
                    .get(&column)
                    .cloned()
                    .unwrap_or_default()
                    .join(", ");
                ui.horizontal(|ui| {
                    ui.label(format!("{:>2}K", column_count));
                    let label = if existing.is_empty() {
                        "(no keys set)".to_string()
                    } else {
                        existing.clone()
                    };
                    ui.label(label);

                    if settings.remapping_column == Some(column_count) {
                        ui.label(format!(
                            "Listening... {}/{}",
                            settings.remapping_buffer.len(),
                            column_count
                        ));
                        if ui.button("Cancel").clicked() {
                            settings.cancel_keybind_capture();
                        }
                    } else if ui.button("Rebind").clicked() {
                        settings.begin_keybind_capture(column_count);
                    }
                });
            }
            if ui.button("Reset keybinds to defaults").clicked() {
                settings.reset_keybinds();
                settings.cancel_keybind_capture();
            }

            if ui.button("Save").clicked() {
                settings.save();

                if settings.hit_window_mode != snapshot.hit_window_mode
                    || (settings.hit_window_value - snapshot.hit_window_value).abs() > f64::EPSILON
                {
                    info!(
                        "Settings: Hit window updated -> mode {:?}, value {:.2}",
                        settings.hit_window_mode, settings.hit_window_value
                    );
                    hit_window_changed =
                        Some((settings.hit_window_mode, settings.hit_window_value));
                }

                if (settings.master_volume - snapshot.master_volume).abs() > f32::EPSILON {
                    info!(
                        "Settings: Master volume updated -> {:.2}",
                        settings.master_volume
                    );
                }

                info!("Settings: Keybinds saved");
                keybinds_updated = true;

                request_toggle = true;
            }
        });

    if !open {
        request_toggle = true;
    }

    SettingsWindowResult {
        request_toggle,
        volume_changed,
        keybinds_updated,
        hit_window_changed,
    }
}
