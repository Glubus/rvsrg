//! Menus module containing all menu configurations.

pub mod panels;
pub mod song_select;

pub use panels::PanelStyleConfig;
pub use song_select::SongSelectConfig;

use serde::{Deserialize, Serialize};

/// Complete menus configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenusConfig {
    #[serde(default)]
    pub song_select: SongSelectConfig,

    #[serde(default)]
    pub panels: PanelStyleConfig,
}
