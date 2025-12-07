//! Notes module containing note type configurations.

mod burst;
mod hold;
mod mine;
mod note;

pub use burst::BurstConfig;
pub use hold::HoldConfig;
pub use mine::MineConfig;
pub use note::{NoteColumnConfig, NoteDefaults};

use serde::{Deserialize, Serialize};

/// Default configurations for notes (fallbacks)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotesDefaults {
    #[serde(default)]
    pub note: NoteDefaults,

    #[serde(default)]
    pub hold: HoldConfig,

    #[serde(default)]
    pub burst: BurstConfig,

    #[serde(default)]
    pub mine: MineConfig,
}

