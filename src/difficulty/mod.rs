//! Difficulty calculation module.
//!
//! This module provides difficulty calculation using Etterna (MinaCalc) and osu! (rosu-pp).
//!
//! ## Usage
//!
//! Difficulty is calculated on-demand when a beatmap is selected,
//! rather than during the initial scan. This dramatically improves scan speed.

#![allow(dead_code)]

pub mod builtin;
pub mod calculator;

// Re-export commonly used types
pub use builtin::{EtternaCalculator, OsuCalculator};
pub use calculator::{CalcError, CalculationContext, DifficultyCalculator};

use minacalc_rs::Calc;
use rosu_map::Beatmap;
use rosu_map::section::hit_objects::{HitObject, HitObjectKind};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex, OnceLock};

struct CalcHolder(Calc);

unsafe impl Send for CalcHolder {}
unsafe impl Sync for CalcHolder {}

#[derive(Debug, Clone, Default)]
pub struct BeatmapSsr {
    pub overall: f64,
    pub stream: f64,
    pub jumpstream: f64,
    pub handstream: f64,
    pub stamina: f64,
    pub jackspeed: f64,
    pub chordjack: f64,
    pub technical: f64,
}

#[derive(Debug, Clone)]
pub struct BeatmapRatingValue {
    pub name: String,
    pub ssr: BeatmapSsr,
}

impl BeatmapRatingValue {
    pub fn new(name: impl Into<String>, ssr: BeatmapSsr) -> Self {
        Self {
            name: name.into(),
            ssr,
        }
    }
}

/// Basic info about a beatmap (without ratings).
/// Used during scan phase - ratings are calculated on-demand later.
#[derive(Debug, Clone)]
pub struct BeatmapBasicInfo {
    pub duration_ms: i32,
    pub nps: f64,
    pub note_count: i32,
}

#[derive(Debug, Clone)]
pub struct DifficultyInfo {
    pub duration_ms: i32,
    pub nps: f64,
    pub ratings: Vec<BeatmapRatingValue>,
}

impl DifficultyInfo {
    pub fn new(duration_ms: i32, nps: f64, ratings: Vec<BeatmapRatingValue>) -> Self {
        Self {
            duration_ms,
            nps,
            ratings,
        }
    }
}

static GLOBAL_CALC: OnceLock<Arc<Mutex<CalcHolder>>> = OnceLock::new();

pub fn init_global_calc() -> Result<(), Box<dyn std::error::Error>> {
    if GLOBAL_CALC.get().is_none() {
        let calc = Calc::new()?;
        let holder = Arc::new(Mutex::new(CalcHolder(calc)));
        let _ = GLOBAL_CALC.set(holder);
    }
    Ok(())
}

fn with_global_calc<F, R>(f: F) -> Result<R, Box<dyn std::error::Error>>
where
    F: FnOnce(&Calc) -> Result<R, Box<dyn std::error::Error>>,
{
    init_global_calc()?;
    let calc_arc = GLOBAL_CALC
        .get()
        .ok_or_else(|| std::io::Error::other("Global MinaCalc not initialized"))?;
    let calc_guard = calc_arc
        .lock()
        .map_err(|_| std::io::Error::other("Calc lock poisoned"))?;
    f(&calc_guard.0)
}

/// Extracts basic metadata from a beatmap without calculating difficulty.
/// This is used during the scan phase for fast importing.
pub fn extract_basic_info(map: &Beatmap) -> Result<BeatmapBasicInfo, Box<dyn std::error::Error>> {
    if map.hit_objects.is_empty() {
        return Err(Box::new(std::io::Error::other("No hit objects found")));
    }

    let first = map.hit_objects.first().map(|h| h.start_time).unwrap_or(0.0);
    let last = map
        .hit_objects
        .last()
        .map(|h| h.start_time.max(resolve_end_time(h)))
        .unwrap_or(first);

    let duration = (last - first).max(0.0);
    let duration_secs = duration / 1000.0;
    let nps = if duration_secs > 0.0 {
        map.hit_objects.len() as f64 / duration_secs
    } else {
        0.0
    };

    let note_count = map
        .hit_objects
        .iter()
        .filter(|ho| matches!(ho.kind, HitObjectKind::Circle(_)))
        .count() as i32;

    Ok(BeatmapBasicInfo {
        duration_ms: duration as i32,
        nps,
        note_count,
    })
}

/// Analyse basique d'une beatmap (placeholder pour calculs futurs)
/// NOTE: This still calculates ratings for backward compatibility.
/// Use `extract_basic_info` for the new scan-without-calc flow.
pub fn analyze(map: &Beatmap) -> Result<DifficultyInfo, Box<dyn std::error::Error>> {
    init_global_calc()?;
    with_global_calc(|calc| analyze_with_calc(map, calc))
}

pub fn analyze_for_rate(
    map: &Beatmap,
    rate: f64,
) -> Result<Vec<BeatmapRatingValue>, Box<dyn std::error::Error>> {
    init_global_calc()?;
    with_global_calc(|calc| {
        let etterna_ssr = EtternaCalculator::calculate_from_beatmap(map, rate)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let osu_ssr = OsuCalculator::calculate_from_beatmap(map, &etterna_ssr, rate)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(vec![
            BeatmapRatingValue::new("etterna", etterna_ssr),
            BeatmapRatingValue::new("osu", osu_ssr),
        ])
    })
}

#[derive(Debug, Clone)]
pub struct RateDifficultyCache {
    pub available_rates: Vec<f64>,
    pub ratings_by_rate: Vec<(f64, Vec<BeatmapRatingValue>)>,
}

pub fn analyze_all_rates(map: &Beatmap) -> Result<RateDifficultyCache, Box<dyn std::error::Error>> {
    init_global_calc()?;
    with_global_calc(|calc| analyze_all_rates_with_calc(map, calc))
}

fn analyze_all_rates_with_calc(
    map: &Beatmap,
    _calc: &Calc,
) -> Result<RateDifficultyCache, Box<dyn std::error::Error>> {
    // Use the new builtin calculators
    let etterna_rates = EtternaCalculator::calculate_all_rates(map)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let mut per_rate: Vec<(f64, Vec<BeatmapRatingValue>)> = Vec::new();

    for (rate_value, etterna_ssr) in etterna_rates {
        let osu_ssr = OsuCalculator::calculate_from_beatmap(map, &etterna_ssr, rate_value)
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        per_rate.push((
            rate_value,
            vec![
                BeatmapRatingValue::new("etterna", etterna_ssr),
                BeatmapRatingValue::new("osu", osu_ssr),
            ],
        ));
    }

    per_rate.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    let available_rates = per_rate.iter().map(|(rate, _)| *rate).collect();

    Ok(RateDifficultyCache {
        available_rates,
        ratings_by_rate: per_rate,
    })
}

fn analyze_with_calc(
    map: &Beatmap,
    _calc: &Calc,
) -> Result<DifficultyInfo, Box<dyn std::error::Error>> {
    if map.hit_objects.is_empty() {
        return Err(Box::new(std::io::Error::other("No hit objects found")));
    }

    let first = map.hit_objects.first().map(|h| h.start_time).unwrap_or(0.0);
    let last = map
        .hit_objects
        .last()
        .map(|h| h.start_time.max(resolve_end_time(h)))
        .unwrap_or(first);

    let duration = (last - first).max(0.0);
    let duration_secs = duration / 1000.0;
    let nps = if duration_secs > 0.0 {
        map.hit_objects.len() as f64 / duration_secs
    } else {
        0.0
    };

    let etterna_ssr = EtternaCalculator::calculate_from_beatmap(map, 1.0)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let osu_ssr = OsuCalculator::calculate_from_beatmap(map, &etterna_ssr, 1.0)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let ratings = vec![
        BeatmapRatingValue::new("etterna", etterna_ssr),
        BeatmapRatingValue::new("osu", osu_ssr),
    ];

    Ok(DifficultyInfo::new(duration as i32, nps, ratings))
}

fn resolve_end_time(obj: &HitObject) -> f64 {
    match &obj.kind {
        HitObjectKind::Hold(hold) => obj.start_time + hold.duration,
        _ => obj.start_time,
    }
}

/// Calculate difficulty for a specific beatmap at a given rate.
/// This is the new on-demand calculation API.
pub fn calculate_on_demand(
    map: &Beatmap,
    calculator_id: &str,
    rate: f64,
) -> Result<BeatmapSsr, CalcError> {
    match calculator_id {
        "etterna" => EtternaCalculator::calculate_from_beatmap(map, rate),
        "osu" => {
            // osu! needs etterna results for weighted skills
            let etterna_ssr = EtternaCalculator::calculate_from_beatmap(map, rate)?;
            OsuCalculator::calculate_from_beatmap(map, &etterna_ssr, rate)
        }
        _ => Err(CalcError::Other(format!(
            "Unknown calculator: {}",
            calculator_id
        ))),
    }
}

/// Calculate difficulty for all calculators at a given rate.
pub fn calculate_all_calculators(
    map: &Beatmap,
    rate: f64,
) -> Result<Vec<BeatmapRatingValue>, CalcError> {
    let etterna_ssr = EtternaCalculator::calculate_from_beatmap(map, rate)?;
    let osu_ssr = OsuCalculator::calculate_from_beatmap(map, &etterna_ssr, rate)?;

    Ok(vec![
        BeatmapRatingValue::new("etterna", etterna_ssr),
        BeatmapRatingValue::new("osu", osu_ssr),
    ])
}
