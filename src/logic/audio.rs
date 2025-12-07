//! Audio manager that sends commands to the dedicated audio thread.
//!
//! This module provides a thread-safe interface for controlling audio playback
//! without blocking the main game loop.

use crate::system::bus::{AudioCommand, SystemBus};
use crossbeam_channel::Sender;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Wrapper for sending commands to the audio thread.
///
/// The `AudioManager` does not perform audio operations directly.
/// Instead, it sends commands through a channel to a dedicated audio thread,
/// ensuring non-blocking audio control from the game logic thread.
pub struct AudioManager {
    cmd_tx: Sender<AudioCommand>,
    position: Arc<AtomicU64>,
    sample_rate: Arc<AtomicU64>,
    channels: Arc<AtomicU64>,
    current_speed: f32,
}

impl AudioManager {
    /// Creates a new audio manager connected to the system bus.
    pub fn new(bus: &SystemBus) -> Self {
        Self {
            cmd_tx: bus.audio_cmd_tx.clone(),
            position: bus.audio_position.clone(),
            sample_rate: bus.audio_sample_rate.clone(),
            channels: bus.audio_channels.clone(),
            current_speed: 1.0,
        }
    }

    /// Loads an audio file for playback.
    pub fn load_music(&mut self, path: &Path) {
        let _ = self.cmd_tx.send(AudioCommand::Load {
            path: path.to_path_buf(),
        });
    }

    /// Starts audio playback.
    pub fn play(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Play);
    }

    /// Pauses audio playback.
    pub fn pause(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Pause);
    }

    /// Stops playback and resets position.
    pub fn stop(&mut self) {
        let _ = self.cmd_tx.send(AudioCommand::Stop);
    }

    /// Sets the playback speed (rate).
    pub fn set_speed(&mut self, speed: f32) {
        self.current_speed = speed;
        let _ = self.cmd_tx.send(AudioCommand::SetSpeed { speed });
    }

    /// Sets the master volume (0.0 to 1.0).
    pub fn set_volume(&mut self, volume: f32) {
        let _ = self.cmd_tx.send(AudioCommand::SetVolume { volume });
    }

    /// Seeks to a position in seconds.
    ///
    /// This operation is non-blocking; the audio thread handles the seek asynchronously.
    pub fn seek(&mut self, position_seconds: f32) {
        let _ = self.cmd_tx.send(AudioCommand::Seek {
            position_secs: position_seconds,
        });
    }

    /// Returns the current playback position in seconds.
    ///
    /// The position is calculated from the sample count shared atomically
    /// with the audio thread.
    pub fn get_position_seconds(&self) -> f64 {
        let samples = self.position.load(Ordering::Relaxed) as f64;
        let sample_rate = self.sample_rate.load(Ordering::Relaxed).max(1) as f64;
        let channels = self.channels.load(Ordering::Relaxed).max(1) as f64;

        samples / (sample_rate * channels)
    }

    /// Returns whether a seek operation is in progress.
    ///
    /// Currently always returns `false` as seeks are handled asynchronously.
    pub fn is_seeking(&self) -> bool {
        false
    }
}

