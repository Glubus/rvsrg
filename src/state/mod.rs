//! State management module.
//!
//! This module contains all game state types and their logic:
//! - `MenuState` - Song selection menu
//! - `GameEngine` - Active gameplay
//! - `EditorState` - Beatmap/skin editor (placeholder)
//! - `GameResultData` - Post-game results
//!
//! Each state implements common traits for snapshots, updates, and action handling.

pub mod editor;
pub mod game;
pub mod global;
pub mod menu;
pub mod result;
pub mod traits;

// Re-exports for convenient access
pub use game::GameEngine;
pub use menu::{ChartCache, DifficultyCache, MenuState, RateCacheEntry};
pub use result::GameResultData;
pub use traits::{ActionContext, HandleAction, Snapshot, Transition, Update};
