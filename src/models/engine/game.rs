use super::{
    constants::{HIT_LINE_Y, NUM_COLUMNS, VISIBLE_DISTANCE},
    hit_window::HitWindow,
    instance::InstanceRaw,
    note::{load_map, NoteData},
    pixel_system::PixelSystem,
    playfield::PlayfieldConfig,
};
use crate::models::replay::ReplayData;
use crate::models::stats::{HitStats, Judgement};
use md5::Context;
use rand::Rng;
use rodio::source::SeekError; // Import nécessaire pour try_seek
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct AudioMonitor<I> {
    inner: I,
    pub played_samples: Arc<AtomicUsize>,
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
    // CORRECTION CRITIQUE : Il faut déléguer le try_seek à la source interne
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)
    }
}

pub struct GameEngine {
    pub chart: Vec<NoteData>,
    pub head_index: usize,
    pub played_samples: Arc<AtomicUsize>,
    pub sample_rate: u32,
    pub channels: u16,
    audio_start_instant: Option<Instant>,
    engine_start_time: Instant,
    pub scroll_speed_ms: f64,
    pub notes_passed: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub hit_window: HitWindow,
    pub active_notes: Vec<(usize, NoteData)>,
    pub hit_stats: HitStats,
    pub last_hit_timing: Option<f64>,
    pub last_hit_judgement: Option<Judgement>,
    pub keys_held: Vec<bool>,
    _audio_stream: OutputStream,
    pub audio_sink: Arc<Mutex<Sink>>,
    audio_path: Option<PathBuf>,
    audio_started: bool,
    pub rate: f64,
    pub replay_data: ReplayData,
    beatmap_hash: Option<String>,
    pub intro_skipped: bool, // Nouveau champ pour empêcher le spam
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
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let now = Instant::now();
        Self {
            chart,
            head_index: 0,
            engine_start_time: now,
            audio_start_instant: None,
            played_samples: Arc::new(AtomicUsize::new(0)),
            sample_rate: 44100,
            channels: 2,
            scroll_speed_ms: 500.0,
            notes_passed: 0,
            combo: 0,
            max_combo: 0,
            hit_window: HitWindow::new(),
            active_notes: Vec::new(),
            hit_stats: HitStats::new(),
            last_hit_timing: None,
            last_hit_judgement: None,
            keys_held: vec![false; NUM_COLUMNS],
            _audio_stream: _stream,
            audio_sink: Arc::new(Mutex::new(sink)),
            audio_path: None,
            audio_started: false,
            rate: 1.0,
            replay_data: ReplayData::new(),
            beatmap_hash: None,
            intro_skipped: false,
        }
    }

    pub fn from_map(map_path: PathBuf, rate: f64) -> Self {
        let beatmap_hash = Self::calculate_file_hash(&map_path).ok();
        let (audio_path, chart) = load_map(map_path);
        let (_stream, stream_handle) = OutputStream::try_default().expect("Stream failed");
        let sink = Sink::try_new(&stream_handle).expect("Sink failed");
        sink.set_speed(rate as f32);
        let played_samples = Arc::new(AtomicUsize::new(0));
        let mut sample_rate = 44100;
        let mut channels = 2;
        match File::open(&audio_path) {
            Ok(file) => match Decoder::new(BufReader::new(file)) {
                Ok(source) => {
                    sample_rate = source.sample_rate();
                    channels = source.channels();
                    let monitor = AudioMonitor {
                        inner: source,
                        played_samples: played_samples.clone(),
                    };
                    sink.append(monitor);
                    sink.pause();
                }
                Err(e) => eprintln!("Error decoding: {}", e),
            },
            Err(e) => eprintln!("Error loading: {}", e),
        }
        Self {
            chart,
            head_index: 0,
            engine_start_time: Instant::now(),
            audio_start_instant: None,
            played_samples,
            sample_rate,
            channels,
            scroll_speed_ms: 500.0,
            notes_passed: 0,
            combo: 0,
            max_combo: 0,
            hit_window: HitWindow::new(),
            active_notes: Vec::new(),
            hit_stats: HitStats::new(),
            last_hit_timing: None,
            last_hit_judgement: None,
            keys_held: vec![false; NUM_COLUMNS],
            _audio_stream: _stream,
            audio_sink: Arc::new(Mutex::new(sink)),
            audio_path: Some(audio_path),
            audio_started: false,
            rate,
            replay_data: ReplayData::new(),
            beatmap_hash,
            intro_skipped: false,
        }
    }

    fn calculate_file_hash(file_path: &PathBuf) -> Result<String, std::io::Error> {
        let mut file = File::open(file_path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        let mut context = Context::new();
        context.consume(buffer.as_bytes());
        Ok(format!("{:x}", context.finalize()))
    }

    pub fn skip_intro(&mut self) {
        // Empêcher le spam
        if self.intro_skipped {
            return;
        }

        if let Some(first_note) = self.chart.first() {
            // Cible : 1 seconde avant la première note
            let target_ms = (first_note.timestamp_ms - 1000.0).max(0.0);
            let target_duration = Duration::from_secs_f64(target_ms / 1000.0);

            // Mise à jour physique de l'audio
            if let Ok(sink) = self.audio_sink.lock() {
                if let Err(e) = sink.try_seek(target_duration) {
                    eprintln!("Seek error: {}", e);
                }
            }

            // Mise à jour des horloges logiques
            let now = Instant::now();
            
            if self.audio_started {
                let elapsed_needed = target_ms / self.rate;
                let adjustment = Duration::from_secs_f64(elapsed_needed / 1000.0);
                self.audio_start_instant = Some(now - adjustment);
            } else {
                if target_ms >= 0.0 {
                    // Forcer le démarrage si on saute après 0.0
                    if let Ok(sink) = self.audio_sink.lock() {
                        sink.play();
                    }
                    self.audio_started = true;
                    
                    let elapsed_needed = target_ms / self.rate;
                    let adjustment = Duration::from_secs_f64(elapsed_needed / 1000.0);
                    self.audio_start_instant = Some(now - adjustment);
                } else {
                    // Toujours dans le pré-compte
                    let elapsed_needed = (target_ms + 5000.0) / self.rate;
                    let adjustment = Duration::from_secs_f64(elapsed_needed / 1000.0);
                    self.engine_start_time = now - adjustment;
                }
            }
            
            // Reset visuel
            self.last_hit_timing = None;
            self.last_hit_judgement = None;
            
            // Marquer comme sauté pour désactiver la touche
            self.intro_skipped = true;
        }
    }

    pub fn set_key_held(&mut self, col: usize, state: bool) {
        if col < self.keys_held.len() {
            self.keys_held[col] = state;
        }
    }
    pub fn get_audio_time(&self) -> f64 {
        let now = Instant::now();
        if let Some(start_time) = self.audio_start_instant {
            let raw_elapsed_ms = now.duration_since(start_time).as_secs_f64() * 1000.0;
            raw_elapsed_ms * self.rate
        } else {
            let elapsed = now.duration_since(self.engine_start_time).as_secs_f64() * 1000.0;
            -5000.0 + (elapsed * self.rate)
        }
    }
    pub fn start_audio_if_needed(&mut self, master_volume: f32) {
        if !self.audio_started {
            let current_musical_time = self.get_audio_time();
            if current_musical_time >= 0.0 {
                if let Ok(sink) = self.audio_sink.lock() {
                    sink.set_volume(master_volume);
                    sink.play();
                    self.audio_started = true;
                    self.audio_start_instant = Some(Instant::now());
                }
            }
        }
    }
    pub fn reset_time(&mut self) {
        let now = Instant::now();
        self.engine_start_time = now;
        self.audio_start_instant = None;
        self.head_index = 0;
        self.notes_passed = 0;
        self.combo = 0;
        self.max_combo = 0;
        self.active_notes.clear();
        self.hit_stats = HitStats::new();
        self.last_hit_timing = None;
        self.last_hit_judgement = None;
        self.keys_held.fill(false);
        for note in &mut self.chart {
            note.hit = false;
        }
        self.played_samples.store(0, Ordering::Relaxed);
        self.audio_started = false;
        if let Ok(sink) = self.audio_sink.lock() {
            sink.stop();
            sink.clear();
            sink.set_speed(self.rate as f32);
        }
        if let Some(ref audio_path) = self.audio_path {
            if let Ok(file) = File::open(audio_path) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    self.sample_rate = source.sample_rate();
                    self.channels = source.channels();
                    let monitor = AudioMonitor {
                        inner: source,
                        played_samples: self.played_samples.clone(),
                    };
                    if let Ok(sink) = self.audio_sink.lock() {
                        sink.append(monitor);
                        sink.pause();
                    }
                }
            }
        }
        self.replay_data = ReplayData::new();
        self.intro_skipped = false;
    }
    pub async fn save_replay(
        &self,
        db: &crate::database::connection::Database,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref hash) = self.beatmap_hash {
            let mut replay_data_with_stats = self.replay_data.clone();
            replay_data_with_stats.hit_stats = Some(self.hit_stats.clone());
            let json_data = replay_data_with_stats.to_json()?;
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;
            db.insert_replay(
                hash,
                timestamp,
                self.notes_passed as i32,
                self.hit_stats.calculate_accuracy(),
                self.max_combo as i32,
                self.rate,
                &json_data,
            )
            .await?;
            Ok(())
        } else {
            Err("No hash".into())
        }
    }
    pub fn set_volume(&self, volume: f32) {
        if let Ok(sink) = self.audio_sink.lock() {
            sink.set_volume(volume);
        }
    }
    pub fn update_hit_window(&mut self, mode: crate::models::settings::HitWindowMode, value: f64) {
        self.hit_window = match mode {
            crate::models::settings::HitWindowMode::OsuOD => {
                HitWindow::from_osu_od(value.clamp(0.0, 10.0))
            }
            crate::models::settings::HitWindowMode::EtternaJudge => {
                HitWindow::from_etterna_judge(value.clamp(1.0, 9.0) as u8)
            }
        };
    }
    pub fn process_input(&mut self, column: usize) -> Option<Judgement> {
        let audio_time = self.get_audio_time();
        let mut best_note: Option<(usize, f64)> = None;
        for (idx, note) in self.chart.iter().enumerate().skip(self.head_index) {
            if note.column == column && !note.hit {
                let musical_diff = note.timestamp_ms - audio_time;
                let normalized_diff = musical_diff / self.rate;
                let (judgement, _) = self.hit_window.judge(normalized_diff);
                if judgement != Judgement::GhostTap {
                    if let Some((_, best_diff)) = best_note {
                        if normalized_diff.abs() < best_diff.abs() {
                            best_note = Some((idx, normalized_diff));
                        }
                    } else {
                        best_note = Some((idx, normalized_diff));
                    }
                }
            }
        }
        if let Some((note_idx, normalized_diff)) = best_note {
            let (judgement, _) = self.hit_window.judge(normalized_diff);
            self.chart[note_idx].hit = true;
            self.active_notes.retain(|(idx, _)| *idx != note_idx);
            self.last_hit_timing = Some(normalized_diff);
            self.last_hit_judgement = Some(judgement);
            if judgement != Judgement::GhostTap {
                self.replay_data.add_hit(note_idx, normalized_diff);
            }
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
        self.hit_stats.ghost_tap += 1;
        self.last_hit_timing = None;
        self.last_hit_judgement = Some(Judgement::GhostTap);
        self.replay_data.add_key_press(audio_time, column);
        Some(Judgement::GhostTap)
    }
    pub fn update_active_notes(&mut self) {
        let audio_time = self.get_audio_time();
        self.active_notes.clear();
        for (idx, note) in self.chart.iter().enumerate().skip(self.head_index) {
            if !note.hit {
                let musical_diff = note.timestamp_ms - audio_time;
                let normalized_diff = musical_diff / self.rate;
                let (judgement, _) = self.hit_window.judge(normalized_diff);
                if judgement != Judgement::GhostTap {
                    self.active_notes.push((idx, note.clone()));
                }
            }
        }
    }
    pub fn detect_misses(&mut self) {
        let audio_time = self.get_audio_time();
        for (idx, note) in self.chart.iter_mut().enumerate().skip(self.head_index) {
            if !note.hit {
                let musical_diff = note.timestamp_ms - audio_time;
                let miss_threshold_musical = -150.0 * self.rate;
                if musical_diff < miss_threshold_musical {
                    note.hit = true;
                    self.hit_stats.miss += 1;
                    self.combo = 0;
                    self.notes_passed += 1;
                    self.replay_data.add_hit(idx, musical_diff / self.rate);
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
    pub fn is_game_finished(&self) -> bool {
        if self.head_index >= self.chart.len() {
            return true;
        }
        self.chart.iter().skip(self.head_index).all(|note| note.hit)
    }
    pub fn get_visible_notes(
        &mut self,
        _pixel_system: &PixelSystem,
        _playfield_config: &PlayfieldConfig,
        _playfield_x: f32,
        _playfield_width: f32,
    ) -> Vec<InstanceRaw> {
        vec![]
    }
}