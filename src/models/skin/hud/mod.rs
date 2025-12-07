//! HUD (Heads-Up Display) module for gameplay UI elements.

pub mod accuracy;
pub mod combo;
pub mod hit_bar;
pub mod judgement;
pub mod notes_remaining;
pub mod nps;
pub mod score;
pub mod scroll_speed;
pub mod time_left;

pub use accuracy::AccuracyConfig;
pub use combo::ComboConfig;
pub use hit_bar::HitBarConfig;
pub use judgement::{JudgementFlashSet, JudgementLabels, JudgementPanelConfig};
pub use notes_remaining::NotesRemainingConfig;
pub use nps::NpsConfig;
pub use score::ScoreConfig;
pub use scroll_speed::ScrollSpeedConfig;
pub use time_left::{TimeDisplayMode, TimeLeftConfig};

use serde::{Deserialize, Serialize};

/// Complete HUD configuration for gameplay
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HudConfig {
    #[serde(default)]
    pub score: ScoreConfig,

    #[serde(default)]
    pub combo: ComboConfig,

    #[serde(default)]
    pub accuracy: AccuracyConfig,

    #[serde(default)]
    pub nps: NpsConfig,

    #[serde(default)]
    pub hit_bar: HitBarConfig,

    /// Judgement Flash - the text that appears centered when hitting notes
    #[serde(default)]
    pub judgement: JudgementFlashSet,

    /// Judgement Panel - the stats display (Marv: 100, Perfect: 50, etc.)
    /// SEPARATE from judgement flash!
    #[serde(default)]
    pub judgement_panel: JudgementPanelConfig,

    /// Notes remaining counter
    #[serde(default)]
    pub notes_remaining: NotesRemainingConfig,

    /// Scroll speed display
    #[serde(default)]
    pub scroll_speed: ScrollSpeedConfig,

    /// Time left / Progress display (bar, circle, or text)
    #[serde(default)]
    pub time_left: TimeLeftConfig,
}

