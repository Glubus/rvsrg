//! Shared channel infrastructure between system threads.
//!
//! The `SystemBus` provides a centralized communication hub for all threads
//! in the application, using lock-free channels for high-performance message passing.

use crate::input::events::{GameAction, InputCommand, RawInputEvent};
use crate::shared::snapshot::RenderState;
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

/// System-level events broadcast to all threads.
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Window resized to new dimensions.
    Resize { width: u32, height: u32 },
    /// Window lost focus.
    FocusLost,
    /// Window gained focus.
    FocusGained,
    /// Application shutdown requested.
    Quit,
}

/// Commands sent to the dedicated audio thread.
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// Load an audio file for playback.
    Load { path: PathBuf },
    /// Start playback.
    Play,
    /// Pause playback.
    Pause,
    /// Stop and reset playback position.
    Stop,
    /// Seek to a position (in seconds).
    Seek { position_secs: f32 },
    /// Change playback speed.
    SetSpeed { speed: f32 },
    /// Change volume level.
    SetVolume { volume: f32 },
}

/// Aggregates the cross-thread communication channels.
///
/// The `SystemBus` is the central hub for inter-thread communication,
/// providing channels for:
/// - Raw input events from the window
/// - Game actions from the input thread
/// - Render snapshots to the render thread
/// - System events (resize, quit, etc.)
/// - Audio commands to the audio thread
#[derive(Clone)]
pub struct SystemBus {
    /// Main → Input: raw keyboard events.
    pub raw_input_tx: Sender<RawInputEvent>,
    pub raw_input_rx: Receiver<RawInputEvent>,

    /// Commands sent to the input thread.
    pub input_cmd_tx: Sender<InputCommand>,
    pub input_cmd_rx: Receiver<InputCommand>,

    /// Input → Logic: processed gameplay actions.
    pub action_tx: Sender<GameAction>,
    pub action_rx: Receiver<GameAction>,

    /// Logic → Render: game state snapshots.
    pub render_tx: Sender<RenderState>,
    pub render_rx: Receiver<RenderState>,

    /// Main → Logic: system events.
    pub sys_tx: Sender<SystemEvent>,
    pub sys_rx: Receiver<SystemEvent>,

    /// Logic → Audio: audio commands.
    pub audio_cmd_tx: Sender<AudioCommand>,
    pub audio_cmd_rx: Receiver<AudioCommand>,

    /// Shared audio position in samples.
    /// Written by the audio thread, read by the logic thread.
    pub audio_position: Arc<AtomicU64>,

    /// Current audio sample rate.
    pub audio_sample_rate: Arc<AtomicU64>,

    /// Number of audio channels.
    pub audio_channels: Arc<AtomicU64>,
}

impl SystemBus {
    /// Creates a new system bus with all channels initialized.
    pub fn new() -> Self {
        let (raw_input_tx, raw_input_rx) = unbounded();
        let (input_cmd_tx, input_cmd_rx) = unbounded();
        let (action_tx, action_rx) = unbounded();

        // Bounded render channel: max 2 frames queued to limit latency
        let (render_tx, render_rx) = bounded(2);

        let (sys_tx, sys_rx) = unbounded();
        let (audio_cmd_tx, audio_cmd_rx) = unbounded();

        Self {
            raw_input_tx,
            raw_input_rx,
            input_cmd_tx,
            input_cmd_rx,
            action_tx,
            action_rx,
            render_tx,
            render_rx,
            sys_tx,
            sys_rx,
            audio_cmd_tx,
            audio_cmd_rx,
            audio_position: Arc::new(AtomicU64::new(0)),
            audio_sample_rate: Arc::new(AtomicU64::new(44100)),
            audio_channels: Arc::new(AtomicU64::new(2)),
        }
    }
}

impl Default for SystemBus {
    fn default() -> Self {
        Self::new()
    }
}
