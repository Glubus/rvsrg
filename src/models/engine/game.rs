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
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// --- AUDIO MONITOR ---
pub struct AudioMonitor<I> {
    inner: I,
    pub played_samples: Arc<AtomicUsize>,
}

impl<I> Iterator for AudioMonitor<I> where I: Iterator {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        if item.is_some() { self.played_samples.fetch_add(1, Ordering::Relaxed); }
        item
    }
}

impl<I> Source for AudioMonitor<I> where I: Source, I::Item: rodio::Sample {
    fn current_frame_len(&self) -> Option<usize> { self.inner.current_frame_len() }
    fn channels(&self) -> u16 { self.inner.channels() }
    fn sample_rate(&self) -> u32 { self.inner.sample_rate() }
    fn total_duration(&self) -> Option<Duration> { self.inner.total_duration() }
}

// --- GAME ENGINE ---

pub struct GameEngine {
    pub chart: Vec<NoteData>,
    pub head_index: usize,
    
    // Audio / Temps
    pub played_samples: Arc<AtomicUsize>, 
    pub sample_rate: u32,
    pub channels: u16,
    audio_start_instant: Option<Instant>,
    engine_start_time: Instant,
    
    // Gameplay
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
    pub rate: f64,
    
    pub replay_data: ReplayData,
    beatmap_hash: Option<String>,
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
            _audio_stream: _stream,
            audio_sink: Arc::new(Mutex::new(sink)),
            audio_path: None,
            audio_started: false,
            rate: 1.0,
            replay_data: ReplayData::new(),
            beatmap_hash: None,
        }
    }

    pub fn from_map(map_path: PathBuf, rate: f64) -> Self {
        let beatmap_hash = Self::calculate_file_hash(&map_path).ok();
        
        // CHARGEMENT NATIF : On garde les timestamps originaux.
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
                    let monitor = AudioMonitor { inner: source, played_samples: played_samples.clone() };
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
            _audio_stream: _stream,
            audio_sink: Arc::new(Mutex::new(sink)),
            audio_path: Some(audio_path),
            audio_started: false,
            rate,
            replay_data: ReplayData::new(),
            beatmap_hash,
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

    // --- TEMPS MUSICAL (AUDIO MASTER) ---
    pub fn get_audio_time(&self) -> f64 {
        let now = Instant::now();

        if let Some(start_time) = self.audio_start_instant {
            // Temps Écoulé Réel * Rate = Temps Musical
            let raw_elapsed_ms = now.duration_since(start_time).as_secs_f64() * 1000.0;
            let musical_time = raw_elapsed_ms * self.rate;
            musical_time
        } else {
            // Lead-in (intro avant musique)
            let elapsed = now.duration_since(self.engine_start_time).as_secs_f64() * 1000.0;
            // On fait avancer le temps négatif à la vitesse du rate
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
        
        for note in &mut self.chart { note.hit = false; }
        self.played_samples.store(0, Ordering::Relaxed);
        self.audio_started = false;
        
        if let Ok(sink) = self.audio_sink.lock() {
            sink.stop();
            sink.clear();
            sink.set_speed(self.rate as f32);
        }

        if let Some(ref audio_path) = self.audio_path {
            match File::open(audio_path) {
                Ok(file) => match Decoder::new(BufReader::new(file)) {
                    Ok(source) => {
                        self.sample_rate = source.sample_rate();
                        self.channels = source.channels();
                        let monitor = AudioMonitor { inner: source, played_samples: self.played_samples.clone() };
                        if let Ok(sink) = self.audio_sink.lock() {
                            sink.append(monitor);
                            sink.pause();
                        }
                    }
                    Err(e) => eprintln!("Error decoding: {}", e),
                },
                Err(e) => eprintln!("Error loading: {}", e),
            }
        }
        self.replay_data = ReplayData::new();
    }

    pub async fn save_replay(&self, db: &crate::database::connection::Database) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref hash) = self.beatmap_hash {
            let mut replay_data_with_stats = self.replay_data.clone();
            replay_data_with_stats.hit_stats = Some(self.hit_stats.clone());
            let json_data = replay_data_with_stats.to_json()?;
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
            db.insert_replay(hash, timestamp, self.notes_passed as i32, self.hit_stats.calculate_accuracy(), self.max_combo as i32, self.rate, &json_data).await?;
            Ok(())
        } else { Err("No hash".into()) }
    }
    
    pub fn set_volume(&self, volume: f32) {
        if let Ok(sink) = self.audio_sink.lock() { sink.set_volume(volume); }
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

    // --- INPUT & JUGEMENT ---

    pub fn process_input(&mut self, column: usize) -> Option<Judgement> {
        let audio_time = self.get_audio_time();
        let mut best_note: Option<(usize, f64)> = None;

        for (idx, note) in self.chart.iter().enumerate().skip(self.head_index) {
            if note.column == column && !note.hit {
                // Différence brute en MS musicales
                let musical_diff = note.timestamp_ms - audio_time;
                
                // Pour la HitWindow, comme on ne peut pas modifier la struct externe,
                // on normalise l'input.
                // Mathématiquement : (Diff / Rate < Window) EST ÉQUIVALENT À (Diff < Window * Rate)
                // C'est exactement la logique "J'ai 500ms -> je dois avoir 750ms" appliquée aux fenêtres.
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
            
            // On sauvegarde le timing réel (normalisé) pour la cohérence stats
            self.last_hit_timing = Some(normalized_diff);
            self.last_hit_judgement = Some(judgement);
            
            if judgement != Judgement::GhostTap { self.replay_data.add_hit(note_idx, normalized_diff); }

            match judgement {
                Judgement::Marv => { self.hit_stats.marv += 1; self.combo += 1; }
                Judgement::Perfect => { self.hit_stats.perfect += 1; self.combo += 1; }
                Judgement::Great => { self.hit_stats.great += 1; self.combo += 1; }
                Judgement::Good => { self.hit_stats.good += 1; self.combo += 1; }
                Judgement::Bad => { self.hit_stats.bad += 1; self.combo += 1; }
                Judgement::Miss => { self.hit_stats.miss += 1; self.combo = 0; }
                Judgement::GhostTap => { self.hit_stats.ghost_tap += 1; }
            }
            if self.combo > self.max_combo { self.max_combo = self.combo; }
            if judgement != Judgement::GhostTap { self.notes_passed += 1; }
            return Some(judgement);
        }

        let has_notes_in_column = self.chart.iter().skip(self.head_index)
            .any(|note| note.column == column && !note.hit);

        if has_notes_in_column {
            self.hit_stats.ghost_tap += 1;
            self.last_hit_timing = None;
            self.last_hit_judgement = Some(Judgement::GhostTap);
            self.replay_data.add_key_press(audio_time, column);
            return Some(Judgement::GhostTap);
        }

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
                
                // LOGIQUE DE MULTIPLICATION ICI
                // Seuil de base (-150ms) * Rate (1.5) = -225ms musicales.
                // Si la différence musicale est pire que -225ms, c'est un Miss.
                let miss_threshold_musical = -150.0 * self.rate;
                
                if musical_diff < miss_threshold_musical {
                    note.hit = true;
                    self.hit_stats.miss += 1;
                    self.combo = 0;
                    self.notes_passed += 1;
                    // Pour le replay/stats, on normalise quand même pour rester cohérent
                    self.replay_data.add_hit(idx, musical_diff / self.rate);
                }
            }
        }
    }

    pub fn get_remaining_notes(&self) -> usize {
        self.chart.iter().skip(self.head_index).filter(|note| !note.hit).count()
    }
    pub fn is_game_finished(&self) -> bool {
        if self.head_index >= self.chart.len() { return true; }
        self.chart.iter().skip(self.head_index).all(|note| note.hit)
    }

    // --- VISUAL (SCROLL) ---
    
    pub fn get_visible_notes(
        &mut self,
        pixel_system: &PixelSystem,
        playfield_config: &PlayfieldConfig,
        playfield_x: f32,
        _playfield_width: f32,
    ) -> Vec<InstanceRaw> {
        let audio_time = self.get_audio_time();

        // LOGIQUE DE MULTIPLICATION (Ta demande 500ms -> 750ms)
        // La distance visible en temps musical DOIT ÊTRE plus grande.
        // ScrollSpeed (500ms) * Rate (1.5) = 750ms de notes visibles.
        let visible_duration_musical = self.scroll_speed_ms * self.rate;

        let max_future_time = audio_time + visible_duration_musical;
        let min_past_time = audio_time - (200.0 * self.rate); 

        while self.head_index < self.chart.len() {
            if self.chart[self.head_index].timestamp_ms < min_past_time {
                self.head_index += 1;
                self.notes_passed += 1;
            } else { break; }
        }

        let column_width_norm = pixel_system.pixels_to_normalized(playfield_config.column_width_pixels);
        let note_width_norm = pixel_system.pixels_to_normalized(playfield_config.note_width_pixels);
        let note_height_norm = pixel_system.pixels_to_normalized(playfield_config.note_height_pixels);
        let mut instances = Vec::with_capacity(500);

        for note in self.chart.iter().skip(self.head_index) {
            if note.timestamp_ms > max_future_time { break; }

            let musical_time_to_hit = note.timestamp_ms - audio_time;
            
            // Calcul de la progression (0.0 à 1.0)
            // Note à 750ms (Musical) / Fenêtre Visuelle de 750ms (Musical) = 1.0 (Haut écran)
            let progress = musical_time_to_hit / visible_duration_musical;

            let y_pos = HIT_LINE_Y + (VISIBLE_DISTANCE * progress as f32);
            let center_x = playfield_x + (note.column as f32 * column_width_norm) + (column_width_norm / 2.0);

            instances.push(InstanceRaw {
                offset: [center_x, y_pos],
                scale: [note_width_norm, note_height_norm],
            });
        }
        instances
    }
}