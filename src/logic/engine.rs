use crate::input::events::GameAction;
use crate::logic::audio::AudioManager;
use crate::models::engine::{
    HIT_LINE_Y, HitWindow, NUM_COLUMNS, NoteData, VISIBLE_DISTANCE, load_map,
};
use crate::models::replay::ReplayData;
use crate::models::stats::{HitStats, Judgement};
use crate::shared::snapshot::GameplaySnapshot;
use std::path::PathBuf;

pub struct GameEngine {
    pub chart: Vec<NoteData>,
    pub head_index: usize,

    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub hit_stats: HitStats,
    pub notes_passed: u32,

    pub keys_held: Vec<bool>,
    pub last_hit_timing: Option<f64>,
    pub last_hit_judgement: Option<Judgement>,

    pub audio_manager: AudioManager,
    pub audio_clock: f64,

    pub rate: f64,
    pub scroll_speed_ms: f64,
    pub hit_window: HitWindow,

    pub replay_data: ReplayData,
    pub beatmap_hash: Option<String>,
    started_audio: bool,
}

impl GameEngine {
    const PRE_ROLL_MS: f64 = 3000.0;

    pub fn new(map_path: PathBuf, rate: f64, beatmap_hash: Option<String>) -> Self {
        let (audio_path, chart) = load_map(map_path);

        let mut audio_manager = AudioManager::new();
        audio_manager.load_music(&audio_path);
        audio_manager.set_speed(rate as f32);

        Self {
            chart,
            head_index: 0,
            score: 0,
            combo: 0,
            max_combo: 0,
            hit_stats: HitStats::new(),
            notes_passed: 0,
            keys_held: vec![false; NUM_COLUMNS],
            last_hit_timing: None,
            last_hit_judgement: None,
            audio_manager,
            audio_clock: -Self::PRE_ROLL_MS,
            replay_data: ReplayData::new(),
            beatmap_hash,
            started_audio: false,
            rate,
            scroll_speed_ms: 500.0,
            hit_window: HitWindow::new(),
        }
    }

    pub fn update(&mut self, dt_seconds: f64) {
        // 1. Avancer l'horloge lisse
        self.audio_clock += dt_seconds * 1000.0 * self.rate;

        if !self.started_audio {
            if self.audio_clock >= 0.0 {
                self.audio_manager.play();
                self.started_audio = true;
            } else {
                return;
            }
        }

        // 2. Synchronisation
        let raw_audio_time = self.audio_manager.get_position_seconds() * 1000.0;
        let drift = raw_audio_time - self.audio_clock;

        if drift.abs() > 80.0 {
            self.audio_clock = raw_audio_time;
        } else if drift.abs() > 5.0 {
            self.audio_clock += drift * 0.35;
        }

        let current_time = self.audio_clock;

        // --- Gestion des Miss ---
        let miss_threshold = self.hit_window.miss_ms;
        let mut new_head = self.head_index;

        while new_head < self.chart.len() {
            // CORRECTION : On ne prend pas de référence mutable 'note' ici qui bloquerait 'self'
            // On vérifie juste si la note est déjà touchée
            if self.chart[new_head].hit {
                new_head += 1;
                continue;
            }

            // On copie le timestamp (c'est un f64, c'est léger)
            let note_timestamp = self.chart[new_head].timestamp_ms;

            if current_time > (note_timestamp + miss_threshold) {
                // La note est manquée. On modifie 'self' séquentiellement sans conflit.

                // 1. Marquer comme hit
                self.chart[new_head].hit = true;

                // 2. Appliquer le jugement (emprunte self)
                self.apply_judgement(Judgement::Miss);

                // 3. Ajouter au replay (emprunte self)
                self.replay_data
                    .add_hit(new_head, (note_timestamp - current_time) / self.rate);

                new_head += 1;
            } else {
                break;
            }
        }
        self.head_index = new_head;
    }

    pub fn handle_input(&mut self, action: GameAction) {
        match action {
            GameAction::Hit { column } => {
                if column < self.keys_held.len() {
                    self.keys_held[column] = true;
                }
                self.process_hit(column);
            }
            GameAction::Release { column } => {
                if column < self.keys_held.len() {
                    self.keys_held[column] = false;
                }
            }
            GameAction::TogglePause => { /* TODO */ }
            _ => {}
        }
    }

    fn process_hit(&mut self, column: usize) {
        let current_time = self.audio_clock;
        let mut best_note_idx = None;
        let mut min_diff = f64::MAX;
        let search_limit = current_time + self.hit_window.miss_ms;

        // Ici on itère avec une référence immuable, c'est ok car on ne modifie rien dans la boucle
        for (i, note) in self.chart.iter().enumerate().skip(self.head_index) {
            if note.timestamp_ms > search_limit {
                break;
            }
            if note.column == column && !note.hit {
                let diff = (note.timestamp_ms - current_time).abs();
                if diff <= self.hit_window.miss_ms && diff < min_diff {
                    min_diff = diff;
                    best_note_idx = Some(i);
                }
            }
        }

        // Une fois la boucle finie, l'emprunt immuable est terminé. On peut modifier self.
        if let Some(idx) = best_note_idx {
            let diff = self.chart[idx].timestamp_ms - current_time;
            let (judgement, _) = self.hit_window.judge(diff);

            self.chart[idx].hit = true;
            self.last_hit_timing = Some(diff);
            self.last_hit_judgement = Some(judgement);
            self.apply_judgement(judgement);
            self.replay_data.add_hit(idx, diff);
        } else {
            self.last_hit_timing = None;
            self.last_hit_judgement = Some(Judgement::GhostTap);
            self.apply_judgement(Judgement::GhostTap);
            self.replay_data.add_key_press(current_time, column);
        }
    }

    fn apply_judgement(&mut self, j: Judgement) {
        match j {
            Judgement::Miss => {
                self.hit_stats.miss += 1;
                self.combo = 0;
                self.notes_passed += 1;
            }
            Judgement::GhostTap => {
                self.hit_stats.ghost_tap += 1;
            }
            _ => {
                match j {
                    Judgement::Marv => self.hit_stats.marv += 1,
                    Judgement::Perfect => self.hit_stats.perfect += 1,
                    Judgement::Great => self.hit_stats.great += 1,
                    Judgement::Good => self.hit_stats.good += 1,
                    Judgement::Bad => self.hit_stats.bad += 1,
                    _ => {}
                }
                self.combo += 1;
                if self.combo > self.max_combo {
                    self.max_combo = self.combo;
                }
                self.notes_passed += 1;
                self.score += match j {
                    Judgement::Marv => 300,
                    Judgement::Perfect => 300,
                    Judgement::Great => 200,
                    Judgement::Good => 100,
                    Judgement::Bad => 50,
                    _ => 0,
                };
            }
        }
    }

    pub fn get_time(&self) -> f64 {
        self.audio_clock
    }

    pub fn is_finished(&self) -> bool {
        if let Some(last_note) = self.chart.last() {
            return self.audio_clock > (last_note.timestamp_ms + 2000.0);
        }
        true
    }

    pub fn get_snapshot(&self) -> GameplaySnapshot {
        let effective_speed = self.scroll_speed_ms * self.rate;
        let max_visible_time = self.audio_clock + effective_speed;

        let visible_notes: Vec<NoteData> = self
            .chart
            .iter()
            .skip(self.head_index)
            .take_while(|n| n.timestamp_ms <= max_visible_time + 2000.0)
            .filter(|n| !n.hit)
            .cloned()
            .collect();

        GameplaySnapshot {
            audio_time: self.audio_clock,
            timestamp: std::time::Instant::now(),
            rate: self.rate,
            scroll_speed: self.scroll_speed_ms,
            visible_notes,
            keys_held: self.keys_held.clone(),
            score: self.score,
            accuracy: self.hit_stats.calculate_accuracy(),
            combo: self.combo,
            hit_stats: self.hit_stats.clone(),
            remaining_notes: self.chart.len().saturating_sub(self.notes_passed as usize),
            last_hit_judgement: self.last_hit_judgement,
            last_hit_timing: self.last_hit_timing,
        }
    }

    pub fn update_hit_window(&mut self, mode: crate::models::settings::HitWindowMode, value: f64) {
        self.hit_window = match mode {
            crate::models::settings::HitWindowMode::OsuOD => HitWindow::from_osu_od(value),
            crate::models::settings::HitWindowMode::EtternaJudge => {
                HitWindow::from_etterna_judge(value as u8)
            }
        };
    }
}
