//! Dedicated audio thread that handles all audio operations.
//!
//! This prevents audio loading/seeking from blocking the game logic thread.

use crate::system::bus::{AudioCommand, SystemBus};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

struct AudioWorker {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    current_path: Option<PathBuf>,
    speed: f32,
    volume: f32,
    sample_rate: u32,
    channels: u16,
    position_counter: Arc<std::sync::atomic::AtomicU64>,
    /// True if audio is available, false for silent mode
    has_audio: bool,
}

impl AudioWorker {
    fn new(bus: &SystemBus) -> Self {
        match OutputStream::try_default() {
            Ok((_stream, stream_handle)) => {
                log::info!("AUDIO: Device found, audio enabled");
                Self {
                    _stream: Some(_stream),
                    stream_handle: Some(stream_handle),
                    sink: None,
                    current_path: None,
                    speed: 1.0,
                    volume: 1.0,
                    sample_rate: 44100,
                    channels: 2,
                    position_counter: bus.audio_position.clone(),
                    has_audio: true,
                }
            }
            Err(e) => {
                log::warn!(
                    "AUDIO: No audio device found ({}), running in silent mode",
                    e
                );
                Self {
                    _stream: None,
                    stream_handle: None,
                    sink: None,
                    current_path: None,
                    speed: 1.0,
                    volume: 1.0,
                    sample_rate: 44100,
                    channels: 2,
                    position_counter: bus.audio_position.clone(),
                    has_audio: false,
                }
            }
        }
    }

    fn handle_command(&mut self, cmd: AudioCommand, bus: &SystemBus) {
        match cmd {
            AudioCommand::Load { path } => {
                self.load_music(&path, bus);
            }
            AudioCommand::Play => {
                if let Some(sink) = &self.sink {
                    sink.play();
                }
            }
            AudioCommand::Pause => {
                if let Some(sink) = &self.sink {
                    sink.pause();
                }
            }
            AudioCommand::Stop => {
                if let Some(sink) = self.sink.take() {
                    sink.stop();
                }
                self.position_counter.store(0, Ordering::Relaxed);
            }
            AudioCommand::Seek { position_secs } => {
                self.seek_to(position_secs, bus);
            }
            AudioCommand::SetSpeed { speed } => {
                self.speed = speed;
                if let Some(sink) = &self.sink {
                    sink.set_speed(speed);
                }
            }
            AudioCommand::SetVolume { volume } => {
                self.volume = volume;
                if let Some(sink) = &self.sink {
                    sink.set_volume(volume);
                }
            }
        }
    }

    fn load_music(&mut self, path: &Path, bus: &SystemBus) {
        self.current_path = Some(path.to_path_buf());
        self.load_from_position(0.0, bus);
    }

    fn load_from_position(&mut self, position_secs: f32, bus: &SystemBus) {
        // Skip if no audio device available
        if !self.has_audio {
            return;
        }

        let Some(path) = &self.current_path else {
            return;
        };

        // Stop l'ancien sink
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }

        let Ok(file) = File::open(path) else {
            log::error!("AUDIO: Cannot open file {:?}", path);
            return;
        };

        let Ok(source) = Decoder::new(BufReader::new(file)) else {
            log::error!("AUDIO: Cannot decode file {:?}", path);
            return;
        };

        self.sample_rate = source.sample_rate();
        self.channels = source.channels();

        // Update shared state
        bus.audio_sample_rate
            .store(self.sample_rate as u64, Ordering::Relaxed);
        bus.audio_channels
            .store(self.channels as u64, Ordering::Relaxed);

        let skip_duration = Duration::from_secs_f32(position_secs.max(0.0));
        let skipped_samples =
            (position_secs.max(0.0) as f64 * self.sample_rate as f64 * self.channels as f64) as u64;

        self.position_counter
            .store(skipped_samples, Ordering::Relaxed);

        let source_skipped = source.skip_duration(skip_duration);

        let monitor = AudioMonitor {
            inner: source_skipped,
            position_counter: self.position_counter.clone(),
        };

        let Some(stream_handle) = &self.stream_handle else {
            return;
        };

        let Ok(sink) = Sink::try_new(stream_handle) else {
            log::error!("AUDIO: Failed to create sink");
            return;
        };
        sink.set_speed(self.speed);
        sink.set_volume(self.volume);
        sink.append(monitor);
        sink.pause();

        self.sink = Some(sink);
        log::info!("AUDIO: Loaded from {:.1}s", position_secs);
    }

    fn seek_to(&mut self, position_secs: f32, bus: &SystemBus) {
        let was_playing = self.sink.as_ref().map(|s| !s.is_paused()).unwrap_or(false);

        self.load_from_position(position_secs, bus);

        if was_playing && let Some(sink) = &self.sink {
            sink.play();
        }

        log::info!("AUDIO: Seeked to {:.1}s", position_secs);
    }
}

struct AudioMonitor<I> {
    inner: I,
    position_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl<I> Iterator for AudioMonitor<I>
where
    I: Iterator,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        if item.is_some() {
            self.position_counter.fetch_add(1, Ordering::Relaxed);
        }
        item
    }
}

impl<I> Source for AudioMonitor<I>
where
    I: Source,
    I::Item: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

/// Starts the dedicated audio thread.
pub fn start_audio_thread(bus: SystemBus) {
    thread::Builder::new()
        .name("Audio Thread".to_string())
        .spawn(move || {
            log::info!("AUDIO: Thread started");

            let mut worker = AudioWorker::new(&bus);

            while let Ok(cmd) = bus.audio_cmd_rx.recv() {
                worker.handle_command(cmd, &bus);
            }

            log::info!("AUDIO: Thread stopped");
        })
        .expect("Failed to spawn Audio thread");
}
