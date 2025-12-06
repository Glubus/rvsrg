//! Gameplay module containing playfield, notes, and receptor configurations.

pub mod key_modes;
pub mod notes;
pub mod playfield;
pub mod receptors;

pub use key_modes::KeyModeConfig;
pub use notes::{
    BurstConfig, HoldConfig, MineConfig, NoteColumnConfig, NoteDefaults, NotesDefaults,
};
pub use playfield::PlayfieldConfig;
pub use receptors::{ReceptorColumnConfig, ReceptorDefaults};

use serde::{Deserialize, Serialize};

/// Default gameplay configuration (fallbacks when no keymode-specific config)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameplayDefaults {
    #[serde(default)]
    pub playfield: PlayfieldConfig,

    #[serde(default)]
    pub notes: NotesDefaults,

    #[serde(default)]
    pub receptors: ReceptorDefaults,
}
