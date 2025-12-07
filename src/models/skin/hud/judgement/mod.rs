//! Judgement flash and panel configuration module.
//!
//! Flash = the text that appears when you hit a note (centered, temporary)
//! Panel = the stats display showing counts (Marv: 100, Perfect: 50, etc.)

mod bad;
mod ghost_tap;
mod good;
mod great;
mod marv;
mod miss;
mod panel;
mod perfect;

pub use bad::JudgementFlashBad;
pub use ghost_tap::JudgementFlashGhostTap;
pub use good::JudgementFlashGood;
pub use great::JudgementFlashGreat;
pub use marv::JudgementFlashMarv;
pub use miss::JudgementFlashMiss;
pub use panel::JudgementPanelConfig;
pub use perfect::JudgementFlashPerfect;

use serde::{Deserialize, Serialize};

/// Labels for judgement text display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgementLabels {
    pub marv: String,
    pub perfect: String,
    pub great: String,
    pub good: String,
    pub bad: String,
    pub miss: String,
    pub ghost_tap: String,
}

impl Default for JudgementLabels {
    fn default() -> Self {
        Self {
            marv: "Marvelous".to_string(),
            perfect: "Perfect".to_string(),
            great: "Great".to_string(),
            good: "Good".to_string(),
            bad: "Bad".to_string(),
            miss: "Miss".to_string(),
            ghost_tap: "Ghost Tap".to_string(),
        }
    }
}

/// Complete set of all judgement flash configurations (the centered flash when hitting notes)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JudgementFlashSet {
    #[serde(default)]
    pub marv: JudgementFlashMarv,

    #[serde(default)]
    pub perfect: JudgementFlashPerfect,

    #[serde(default)]
    pub great: JudgementFlashGreat,

    #[serde(default)]
    pub good: JudgementFlashGood,

    #[serde(default)]
    pub bad: JudgementFlashBad,

    #[serde(default)]
    pub miss: JudgementFlashMiss,

    #[serde(default)]
    pub ghost_tap: JudgementFlashGhostTap,

    /// Show +/- timing indicator on judgement flash
    /// - = early hit, + = late hit
    #[serde(default)]
    pub show_timing: bool,
}

impl JudgementFlashSet {
    /// Get labels from the flash set
    pub fn labels(&self) -> JudgementLabels {
        JudgementLabels {
            marv: self.marv.label.clone(),
            perfect: self.perfect.label.clone(),
            great: self.great.label.clone(),
            good: self.good.label.clone(),
            bad: self.bad.label.clone(),
            miss: self.miss.label.clone(),
            ghost_tap: self.ghost_tap.label.clone(),
        }
    }
}

