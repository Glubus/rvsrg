use super::{
    constants::{HIT_LINE_Y, NUM_COLUMNS, VISIBLE_DISTANCE},
    hit_window::HitWindow,
    instance::InstanceRaw,
    note::{NoteData, load_map},
    pixel_system::PixelSystem,
    playfield::PlayfieldConfig,
};
use crate::models::stats::{HitStats, Judgement};
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct GameEngine {
    pub chart: Vec<NoteData>,
    pub head_index: usize,
    pub start_time: Instant,
    pub scroll_speed_ms: f64,
    pub notes_passed: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub hit_window: HitWindow,
    pub active_notes: Vec<(usize, NoteData)>,
    pub hit_stats: HitStats,
    pub last_hit_timing: Option<f64>,
    pub last_hit_judgement: Option<Judgement>,
    _audio_stream: OutputStream,
    pub audio_sink: Arc<Mutex<Sink>>,
    audio_path: Option<PathBuf>,
    audio_started: bool,
    rate: f64,
}

impl GameEngine {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let mut chart = Vec::new();
        let mut current_time = 1000.0;

        for _ in 0..2000 {
            chart.push(NoteData {
                timestamp_ms: current_time,
                column: rng.random_range(0..NUM_COLUMNS),
                hit: false,
            });
            current_time += rng.random_range(50.0..500.0);
        }

        let (_stream, stream_handle) =
            OutputStream::try_default().unwrap_or_else(|_| OutputStream::try_default().unwrap());
        let sink = Sink::try_new(&stream_handle).unwrap();

        Self {
            chart,
            head_index: 0,
            start_time: Instant::now(),
            scroll_speed_ms: 500.0,
            notes_passed: 0,
            combo: 0,
            max_combo: 0,
            hit_window: HitWindow::new(),
            active_notes: Vec::new(),
            hit_stats: HitStats::new(),
            last_hit_timing: None,
            last_hit_judgement: None,
            _audio_stream: _stream,
            audio_sink: Arc::new(Mutex::new(sink)),
            audio_path: None,
            audio_started: false,
            rate: 1.0,
        }
    }

    pub fn from_map(map_path: PathBuf, rate: f64) -> Self {
        let (audio_path, chart) = load_map(map_path, rate);

        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Impossible de créer le stream audio");

        let sink = Sink::try_new(&stream_handle).expect("Impossible de créer le sink audio");

        sink.set_speed(rate as f32);

        match File::open(&audio_path) {
            Ok(file) => match Decoder::new(BufReader::new(file)) {
                Ok(source) => {
                    sink.append(source);
                    sink.pause();
                }
                Err(e) => {
                    eprintln!("Error: Unable to decode audio from {:?}: {}", audio_path, e);
                }
            },
            Err(e) => {
                eprintln!("Error: Unable to load audio from {:?}: {}", audio_path, e);
            }
        }

        let start_time = Instant::now();
        let sink_arc = Arc::new(Mutex::new(sink));

        Self {
            chart,
            head_index: 0,
            start_time,
            scroll_speed_ms: 500.0,
            notes_passed: 0,
            combo: 0,
            max_combo: 0,
            hit_window: HitWindow::new(),
            active_notes: Vec::new(),
            hit_stats: HitStats::new(),
            last_hit_timing: None,
            last_hit_judgement: None,
            _audio_stream: _stream,
            audio_sink: sink_arc,
            audio_path: Some(audio_path),
            audio_started: false,
            rate,
        }
    }

    pub fn get_game_time(&self) -> f64 {
        let now = Instant::now();
        let elapsed_ms = now.duration_since(self.start_time).as_secs_f64() * 1000.0;
        elapsed_ms - 5000.0
    }

    pub fn start_audio_if_needed(&mut self) {
        if !self.audio_started {
            let game_time = self.get_game_time();
            if game_time >= 0.0 {
                if let Ok(sink) = self.audio_sink.lock() {
                    sink.play();
                    self.audio_started = true;
                }
            }
        }
    }

    pub fn reset_time(&mut self) {
        self.start_time = Instant::now();
        self.head_index = 0;
        self.notes_passed = 0;
        self.combo = 0;
        self.max_combo = 0;
        self.active_notes.clear();
        self.hit_stats = HitStats::new();
        self.last_hit_timing = None;
        self.last_hit_judgement = None;
        for note in &mut self.chart {
            note.hit = false;
        }
        if let Ok(sink) = self.audio_sink.lock() {
            sink.stop();
            sink.clear();
        }

        if let Some(ref audio_path) = self.audio_path {
            match File::open(audio_path) {
                Ok(file) => match Decoder::new(BufReader::new(file)) {
                    Ok(source) => {
                        if let Ok(sink) = self.audio_sink.lock() {
                            sink.append(source);
                            sink.pause();
                        }
                        self.audio_started = false;
                    }
                    Err(e) => {
                        eprintln!("Error: Unable to decode audio from {:?}: {}", audio_path, e);
                    }
                },
                Err(e) => {
                    eprintln!("Error: Unable to load audio from {:?}: {}", audio_path, e);
                }
            }
        }
    }

    pub fn process_input(&mut self, column: usize) -> Option<Judgement> {
        let song_time = self.get_game_time();

        let mut best_note: Option<(usize, f64)> = None;

        for (idx, note) in self.chart.iter().enumerate().skip(self.head_index) {
            if note.column == column && !note.hit {
                let time_diff = note.timestamp_ms - song_time;

                let (judgement, _) = self.hit_window.judge(time_diff);
                if judgement != Judgement::GhostTap {
                    if let Some((_, best_diff)) = best_note {
                        if time_diff.abs() < best_diff.abs() {
                            best_note = Some((idx, time_diff));
                        }
                    } else {
                        best_note = Some((idx, time_diff));
                    }
                }
            }
        }

        if let Some((note_idx, time_diff)) = best_note {
            let (judgement, _) = self.hit_window.judge(time_diff);
            self.chart[note_idx].hit = true;
            self.active_notes.retain(|(idx, _)| *idx != note_idx);

            self.last_hit_timing = Some(time_diff);
            self.last_hit_judgement = Some(judgement);

            match judgement {
                Judgement::Marv => {
                    self.hit_stats.marv += 1;
                    self.combo += 1;
                }
                Judgement::Perfect => {
                    self.hit_stats.perfect += 1;
                    self.combo += 1;
                }
                Judgement::Great => {
                    self.hit_stats.great += 1;
                    self.combo += 1;
                }
                Judgement::Good => {
                    self.hit_stats.good += 1;
                    self.combo += 1;
                }
                Judgement::Bad => {
                    self.hit_stats.bad += 1;
                    self.combo += 1;
                }
                Judgement::Miss => {
                    self.hit_stats.miss += 1;
                    self.combo = 0;
                }
                Judgement::GhostTap => {
                    self.hit_stats.ghost_tap += 1;
                }
            }

            if self.combo > self.max_combo {
                self.max_combo = self.combo;
            }

            if judgement != Judgement::GhostTap {
                self.notes_passed += 1;
            }

            return Some(judgement);
        }

        let has_notes_in_column = self
            .chart
            .iter()
            .skip(self.head_index)
            .any(|note| note.column == column && !note.hit);

        if has_notes_in_column {
            self.hit_stats.ghost_tap += 1;
            self.last_hit_timing = None;
            self.last_hit_judgement = Some(Judgement::GhostTap);
            return Some(Judgement::GhostTap);
        }

        self.hit_stats.ghost_tap += 1;
        self.last_hit_timing = None;
        self.last_hit_judgement = Some(Judgement::GhostTap);
        Some(Judgement::GhostTap)
    }

    pub fn update_active_notes(&mut self) {
        let song_time = self.get_game_time();

        self.active_notes.clear();

        for (idx, note) in self.chart.iter().enumerate().skip(self.head_index) {
            if !note.hit {
                let time_diff = note.timestamp_ms - song_time;
                let (judgement, _) = self.hit_window.judge(time_diff);
                if judgement != Judgement::GhostTap {
                    self.active_notes.push((idx, note.clone()));
                }
            }
        }
    }

    pub fn detect_misses(&mut self) {
        let song_time = self.get_game_time();

        for note in self.chart.iter_mut().skip(self.head_index) {
            if !note.hit {
                let time_diff = note.timestamp_ms - song_time;

                if time_diff < -150.0 {
                    note.hit = true;
                    self.hit_stats.miss += 1;
                    self.combo = 0;
                    self.notes_passed += 1;
                }
            }
        }
    }

    pub fn get_remaining_notes(&self) -> usize {
        self.chart
            .iter()
            .skip(self.head_index)
            .filter(|note| !note.hit)
            .count()
    }

    pub fn get_visible_notes(
        &mut self,
        pixel_system: &PixelSystem,
        playfield_config: &PlayfieldConfig,
        playfield_x: f32,
        _playfield_width: f32,
    ) -> Vec<InstanceRaw> {
        let now = Instant::now();
        let song_time = now.duration_since(self.start_time).as_secs_f64() * 1000.0;

        let max_future_time = song_time + self.scroll_speed_ms;
        let min_past_time = song_time - 200.0;

        while self.head_index < self.chart.len() {
            if self.chart[self.head_index].timestamp_ms < min_past_time {
                self.head_index += 1;
                self.notes_passed += 1;
            } else {
                break;
            }
        }

        let column_width_norm =
            pixel_system.pixels_to_normalized(playfield_config.column_width_pixels);
        let note_width_norm = pixel_system.pixels_to_normalized(playfield_config.note_width_pixels);
        let note_height_norm =
            pixel_system.pixels_to_normalized(playfield_config.note_height_pixels);

        let mut instances = Vec::with_capacity(500);

        for note in self.chart.iter().skip(self.head_index) {
            if note.timestamp_ms > max_future_time {
                break;
            }

            let time_to_hit = note.timestamp_ms - song_time;
            let progress = time_to_hit / self.scroll_speed_ms;

            let y_pos = HIT_LINE_Y + (VISIBLE_DISTANCE * progress as f32);

            let center_x =
                playfield_x + (note.column as f32 * column_width_norm) + (column_width_norm / 2.0);

            instances.push(InstanceRaw {
                offset: [center_x, y_pos],
                scale: [note_width_norm, note_height_norm],
            });
        }

        instances
    }
}
