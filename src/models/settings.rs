#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitWindowMode {
    OsuOD,
    EtternaJudge,
}

pub struct GameSettings {
    pub is_open: bool,          // Le menu est-il ouvert ?
    pub show_keybindings: bool, // Le menu de remapping est-il ouvert ?
    pub master_volume: f32,     // 0.0 à 1.0
    pub hit_window_mode: HitWindowMode,
    pub hit_window_value: f64,  // OD (0.0-10.0) ou Judge Level (1-9)
}

impl GameSettings {
    pub fn new() -> Self {
        Self {
            is_open: false,
            show_keybindings: false,
            master_volume: 0.5,
            hit_window_mode: HitWindowMode::OsuOD,
            hit_window_value: 5.0, // OD 5 par défaut
        }
    }
}
