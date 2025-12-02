//! osu! difficulty calculator using rosu-pp.
//!
//! Uses rosu-pp for the overall star rating, then weights the
//! skill breakdown proportionally to Etterna's SSR values.

use crate::difficulty::{BeatmapSsr, CalcError, CalculationContext, DifficultyCalculator};
use std::str::FromStr;

/// osu! difficulty calculator using rosu-pp.
#[derive(Debug, Clone)]
pub struct OsuCalculator {
    version: String,
}

impl Default for OsuCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl OsuCalculator {
    pub fn new() -> Self {
        Self {
            version: "v1.0".to_string(),
        }
    }

    /// Calculate difficulty for a beatmap at a specific rate.
    /// Uses etterna SSR for weighted skill breakdown.
    pub fn calculate_from_beatmap(
        map: &rosu_map::Beatmap,
        etterna_ssr: &BeatmapSsr,
        rate: f64,
    ) -> Result<BeatmapSsr, CalcError> {
        let map_str = map
            .clone()
            .encode_to_string()
            .map_err(|e| CalcError::InvalidBeatmap(e.to_string()))?;

        let rosu_map = rosu_pp::Beatmap::from_str(&map_str)
            .map_err(|e| CalcError::InvalidBeatmap(e.to_string()))?;

        let diff_attrs = rosu_pp::Difficulty::new()
            .clock_rate(rate)
            .calculate(&rosu_map);
        let sr = diff_attrs.stars();

        // Weight each skill proportionally: (skill/overall) * osu_sr
        let weight = |value: f64| -> f64 {
            if etterna_ssr.overall > 0.0 {
                (value / etterna_ssr.overall) * sr
            } else {
                0.0
            }
        };

        Ok(BeatmapSsr {
            overall: sr,
            stream: weight(etterna_ssr.stream),
            jumpstream: weight(etterna_ssr.jumpstream),
            handstream: weight(etterna_ssr.handstream),
            stamina: weight(etterna_ssr.stamina),
            jackspeed: weight(etterna_ssr.jackspeed),
            chordjack: weight(etterna_ssr.chordjack),
            technical: weight(etterna_ssr.technical),
        })
    }
}

impl DifficultyCalculator for OsuCalculator {
    fn id(&self) -> &str {
        "osu"
    }

    fn display_name(&self) -> &str {
        "osu!"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn calculate(&self, ctx: &CalculationContext) -> Result<BeatmapSsr, CalcError> {
        // Requires etterna SSR to weight the skills properly
        let etterna_ssr = ctx
            .other_results
            .get("etterna")
            .ok_or_else(|| CalcError::Other("osu! calculator requires etterna SSR".to_string()))?;

        // Approximate star rating from NPS (used when we don't have the actual beatmap)
        let sr = ctx.nps * ctx.rate * 0.5;

        let weight = |value: f64| -> f64 {
            if etterna_ssr.overall > 0.0 {
                (value / etterna_ssr.overall) * sr
            } else {
                0.0
            }
        };

        Ok(BeatmapSsr {
            overall: sr,
            stream: weight(etterna_ssr.stream),
            jumpstream: weight(etterna_ssr.jumpstream),
            handstream: weight(etterna_ssr.handstream),
            stamina: weight(etterna_ssr.stamina),
            jackspeed: weight(etterna_ssr.jackspeed),
            chordjack: weight(etterna_ssr.chordjack),
            technical: weight(etterna_ssr.technical),
        })
    }

    fn supports_arbitrary_rates(&self) -> bool {
        true
    }
}


