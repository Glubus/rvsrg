//! Audio stream wrapper providing timing helpers for the game engine.

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    music_sink: Sink,

    pub played_samples: Arc<AtomicUsize>,
    pub sample_rate: u32,
    pub channels: u16, // Needed to compute accurate timing offsets.
}

impl AudioManager {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().expect("No audio device found");
        let music_sink = Sink::try_new(&stream_handle).expect("Failed to create sink");

        Self {
            _stream,
            stream_handle,
            music_sink,
            played_samples: Arc::new(AtomicUsize::new(0)),
            sample_rate: 44100,
            channels: 2, // Reasonable default if no source is provided.
        }
    }

    pub fn load_music(&mut self, path: &Path) {
        if let Ok(file) = File::open(path) {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                self.sample_rate = source.sample_rate();
                self.channels = source.channels(); // Mirror the actual channel count.

                self.played_samples.store(0, Ordering::Relaxed);

                let monitor = AudioMonitor {
                    inner: source,
                    played_samples: self.played_samples.clone(),
                };

                if !self.music_sink.empty() {
                    self.music_sink.stop();
                }
                self.music_sink.append(monitor);
                self.music_sink.pause();
            }
        } else {
            log::error!("Audio file not found: {:?}", path);
        }
    }

    pub fn play(&self) {
        self.music_sink.play();
    }

    pub fn pause(&self) {
        self.music_sink.pause();
    }

    pub fn stop(&self) {
        self.music_sink.stop();
    }

    pub fn set_speed(&self, speed: f32) {
        self.music_sink.set_speed(speed);
    }

    pub fn set_volume(&self, volume: f32) {
        self.music_sink.set_volume(volume);
    }

    pub fn get_position_seconds(&self) -> f64 {
        let samples = self.played_samples.load(Ordering::Relaxed) as f64;
        let channels = self.channels.max(1) as f64; // Avoid division by zero.

        // CORRECTION : On divise par le nombre de canaux !
        // Ex: 88200 samples / (44100 Hz * 2 canaux) = 1 seconde.
        samples / (self.sample_rate as f64 * channels)
    }
}

struct AudioMonitor<I> {
    inner: I,
    played_samples: Arc<AtomicUsize>,
}

impl<I> Iterator for AudioMonitor<I>
where
    I: Iterator,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        if item.is_some() {
            self.played_samples.fetch_add(1, Ordering::Relaxed);
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
