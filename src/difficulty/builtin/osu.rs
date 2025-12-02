//! osu! difficulty calculator using rosu-pp.

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

        let weight = |value: f64| -> f64 {
            if etterna_ssr.overall > 0.0 {
                (value / etterna_ssr.overall) * sr
            } else {
                sr
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

    /// Calculate difficulty directly from a beatmap (without etterna reference).
    pub fn calculate_from_beatmap_standalone(
        map: &rosu_map::Beatmap,
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

        // Without etterna reference, use simple distribution
        Ok(BeatmapSsr {
            overall: sr,
            stream: sr * 0.85,
            jumpstream: sr * 0.90,
            handstream: sr * 0.75,
            stamina: sr * 0.80,
            jackspeed: sr * 0.60,
            chordjack: sr * 0.65,
            technical: sr * 0.50,
        })
    }
}

impl DifficultyCalculator for OsuCalculator {
    fn id(&self) -> &str {
        "osu"
    }

    fn display_name(&self) -> &str {
        "osu! (rosu-pp)"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn calculate(&self, ctx: &CalculationContext) -> Result<BeatmapSsr, CalcError> {
        // Simplified calculation when only context is available.
        // In practice, calculate_from_beatmap should be used directly.
        let base = ctx.nps * ctx.rate * 0.5; // Approximate star rating
        
        // Check for etterna result to weight the skills
        if let Some(etterna_ssr) = ctx.other_results.get("etterna") {
            let weight = |value: f64| -> f64 {
                if etterna_ssr.overall > 0.0 {
                    (value / etterna_ssr.overall) * base
                } else {
                    base
                }
            };

            Ok(BeatmapSsr {
                overall: base,
                stream: weight(etterna_ssr.stream),
                jumpstream: weight(etterna_ssr.jumpstream),
                handstream: weight(etterna_ssr.handstream),
                stamina: weight(etterna_ssr.stamina),
                jackspeed: weight(etterna_ssr.jackspeed),
                chordjack: weight(etterna_ssr.chordjack),
                technical: weight(etterna_ssr.technical),
            })
        } else {
            Ok(BeatmapSsr {
                overall: base,
                stream: base * 0.85,
                jumpstream: base * 0.90,
                handstream: base * 0.75,
                stamina: base * 0.80,
                jackspeed: base * 0.60,
                chordjack: base * 0.65,
                technical: base * 0.50,
            })
        }
    }

    fn supports_arbitrary_rates(&self) -> bool {
        true // rosu-pp supports any rate
    }

    fn available_rates(&self) -> Option<Vec<f64>> {
        None // Arbitrary rates supported
    }
}

