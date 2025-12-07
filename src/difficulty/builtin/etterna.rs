//! Etterna difficulty calculator using MinaCalc.

use crate::difficulty::{BeatmapSsr, CalcError};
use minacalc_rs::{AllRates, Calc, HashMapCalcExt, OsuCalcExt};
use std::sync::{Arc, Mutex, OnceLock};

struct CalcHolder(Calc);

unsafe impl Send for CalcHolder {}
unsafe impl Sync for CalcHolder {}

static GLOBAL_CALC: OnceLock<Arc<Mutex<CalcHolder>>> = OnceLock::new();

fn init_global_calc() -> Result<(), CalcError> {
    if GLOBAL_CALC.get().is_none() {
        let calc = Calc::new()
            .map_err(|e| CalcError::CalculationFailed(format!("MinaCalc init: {}", e)))?;
        let holder = Arc::new(Mutex::new(CalcHolder(calc)));
        let _ = GLOBAL_CALC.set(holder);
    }
    Ok(())
}

fn with_global_calc<F, R>(f: F) -> Result<R, CalcError>
where
    F: FnOnce(&Calc) -> Result<R, CalcError>,
{
    init_global_calc()?;
    let calc_arc = GLOBAL_CALC
        .get()
        .ok_or_else(|| CalcError::Other("Global MinaCalc not initialized".to_string()))?;
    let calc_guard = calc_arc
        .lock()
        .map_err(|_| CalcError::Other("Calc lock poisoned".to_string()))?;
    f(&calc_guard.0)
}

/// Etterna difficulty calculator using MinaCalc.
#[derive(Debug, Clone, Default)]
pub struct EtternaCalculator;

impl EtternaCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate difficulty for a beatmap at a specific rate.
    pub fn calculate_from_beatmap(
        map: &rosu_map::Beatmap,
        rate: f64,
    ) -> Result<BeatmapSsr, CalcError> {
        with_global_calc(|calc| {
            let map_string = map
                .clone()
                .encode_to_string()
                .map_err(|e| CalcError::InvalidBeatmap(e.to_string()))?;

            let msd_results: AllRates = calc
                .calculate_msd_from_string(map_string)
                .map_err(|e| CalcError::CalculationFailed(e.to_string()))?;

            let hashmap = msd_results
                .as_hashmap()
                .map_err(|e| CalcError::CalculationFailed(e.to_string()))?;

            let rate_key_precision_two = format!("{:.2}", rate);
            let rate_key_precision_one = format!("{:.1}", rate);

            let ssr_entry = hashmap
                .get(&rate_key_precision_two)
                .or_else(|| hashmap.get(&rate_key_precision_one))
                .or_else(|| hashmap.get("1.0"))
                .ok_or_else(|| CalcError::UnsupportedRate(rate))?;

            Ok(BeatmapSsr {
                overall: ssr_entry.overall as f64,
                stream: ssr_entry.stream as f64,
                jumpstream: ssr_entry.jumpstream as f64,
                handstream: ssr_entry.handstream as f64,
                stamina: ssr_entry.stamina as f64,
                jackspeed: ssr_entry.jackspeed as f64,
                chordjack: ssr_entry.chordjack as f64,
                technical: ssr_entry.technical as f64,
            })
        })
    }

    /// Calculate difficulty for all available rates.
    pub fn calculate_all_rates(
        map: &rosu_map::Beatmap,
    ) -> Result<Vec<(f64, BeatmapSsr)>, CalcError> {
        with_global_calc(|calc| {
            let map_string = map
                .clone()
                .encode_to_string()
                .map_err(|e| CalcError::InvalidBeatmap(e.to_string()))?;

            let msd_results: AllRates = calc
                .calculate_msd_from_string(map_string)
                .map_err(|e| CalcError::CalculationFailed(e.to_string()))?;

            let hashmap = msd_results
                .as_hashmap()
                .map_err(|e| CalcError::CalculationFailed(e.to_string()))?;

            let mut results = Vec::new();

            for (rate_key, ssr_entry) in hashmap.iter() {
                let Ok(rate_value) = rate_key.parse::<f64>() else {
                    continue;
                };

                let ssr = BeatmapSsr {
                    overall: ssr_entry.overall as f64,
                    stream: ssr_entry.stream as f64,
                    jumpstream: ssr_entry.jumpstream as f64,
                    handstream: ssr_entry.handstream as f64,
                    stamina: ssr_entry.stamina as f64,
                    jackspeed: ssr_entry.jackspeed as f64,
                    chordjack: ssr_entry.chordjack as f64,
                    technical: ssr_entry.technical as f64,
                };

                results.push((rate_value, ssr));
            }

            results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        })
    }
}
