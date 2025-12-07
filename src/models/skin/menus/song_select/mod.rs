//! Song select menu module.

mod beatmap_info;
mod difficulty_button;
mod leaderboard;
mod rating_colors;
mod search_bar;
mod search_panel;
mod song_button;

pub use beatmap_info::BeatmapInfoConfig;
pub use difficulty_button::DifficultyButtonConfig;
pub use leaderboard::LeaderboardConfig;
pub use rating_colors::RatingColorsConfig;
pub use search_bar::SearchBarConfig;
pub use search_panel::SearchPanelConfig;
pub use song_button::SongButtonConfig;

use serde::{Deserialize, Serialize};

/// Complete song select menu configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SongSelectConfig {
    #[serde(default)]
    pub song_button: SongButtonConfig,

    #[serde(default)]
    pub difficulty_button: DifficultyButtonConfig,

    #[serde(default)]
    pub search_bar: SearchBarConfig,

    #[serde(default)]
    pub search_panel: SearchPanelConfig,

    #[serde(default)]
    pub beatmap_info: BeatmapInfoConfig,

    #[serde(default)]
    pub leaderboard: LeaderboardConfig,

    #[serde(default)]
    pub rating_colors: RatingColorsConfig,
}

