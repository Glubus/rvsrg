//! Hit window timing configuration.
//!
//! All thresholds are stored in **microseconds (i64)** for consistency
//! with the rest of the timing system.

use crate::models::stats::Judgement;

/// Microseconds per millisecond.
pub const US_PER_MS: i64 = 1000;

/// Hit window timing thresholds in microseconds.
#[derive(Debug, Clone, Copy)]
pub struct HitWindow {
    pub marv_us: i64,
    pub perfect_us: i64,
    pub great_us: i64,
    pub good_us: i64,
    pub bad_us: i64,
    pub miss_us: i64,
}

impl HitWindow {
    /// Default hit window values.
    pub fn new() -> Self {
        Self {
            marv_us: 16 * US_PER_MS,
            perfect_us: 50 * US_PER_MS,
            great_us: 65 * US_PER_MS,
            good_us: 100 * US_PER_MS,
            bad_us: 150 * US_PER_MS,
            miss_us: 200 * US_PER_MS,
        }
    }

    /// Creates a HitWindow based on osu! OD (Overall Difficulty).
    pub fn from_osu_od(od: f64) -> Self {
        Self {
            marv_us: (16.0 * US_PER_MS as f64) as i64,
            perfect_us: ((64.0 - 3.0 * od) * US_PER_MS as f64) as i64,
            great_us: ((97.0 - 3.0 * od) * US_PER_MS as f64) as i64,
            good_us: ((127.0 - 3.0 * od) * US_PER_MS as f64) as i64,
            bad_us: ((151.0 - 3.0 * od) * US_PER_MS as f64) as i64,
            miss_us: ((188.0 - 3.0 * od) * US_PER_MS as f64) as i64,
        }
    }

    /// Creates a HitWindow based on Etterna Judge Level (J4 = Standard).
    pub fn from_etterna_judge(judge_level: u8) -> Self {
        let scale = if judge_level == 9 {
            0.2
        } else {
            1.0 - ((judge_level as f64 - 4.0) / 6.0)
        };

        let base_marv = 22.5;
        let base_perf = 45.0;
        let base_great = 90.0;
        let base_good = 135.0;
        let base_bad = 180.0;

        // Etterna special rule: Bad never goes below 180ms
        let bad_calculated = (base_bad * scale).max(180.0);

        Self {
            marv_us: (base_marv * scale * US_PER_MS as f64) as i64,
            perfect_us: (base_perf * scale * US_PER_MS as f64) as i64,
            great_us: (base_great * scale * US_PER_MS as f64) as i64,
            good_us: (base_good * scale * US_PER_MS as f64) as i64,
            bad_us: (bad_calculated * US_PER_MS as f64) as i64,
            miss_us: 500 * US_PER_MS, // Standard Etterna Miss window
        }
    }

    /// Custom constructor with all values (in µs).
    pub fn from_custom_us(
        marv: i64,
        perf: i64,
        great: i64,
        good: i64,
        bad: i64,
        miss: i64,
    ) -> Self {
        Self {
            marv_us: marv,
            perfect_us: perf,
            great_us: great,
            good_us: good,
            bad_us: bad,
            miss_us: miss,
        }
    }

    /// Returns the miss threshold (already in µs).
    #[inline]
    pub fn miss_threshold(&self) -> i64 {
        self.miss_us
    }

    /// Judges a timing difference (in microseconds).
    /// Returns the judgement and whether the note was hit (true) or missed (false).
    pub fn judge(&self, timing_diff_us: i64) -> (Judgement, bool) {
        let abs_diff = timing_diff_us.abs();

        // If timing exceeds the miss window, it's a Ghost Tap
        if abs_diff > self.miss_us {
            return (Judgement::GhostTap, false);
        }

        if abs_diff <= self.marv_us {
            (Judgement::Marv, true)
        } else if abs_diff <= self.perfect_us {
            (Judgement::Perfect, true)
        } else if abs_diff <= self.great_us {
            (Judgement::Great, true)
        } else if abs_diff <= self.good_us {
            (Judgement::Good, true)
        } else if abs_diff <= self.bad_us {
            (Judgement::Bad, true)
        } else {
            // In the zone between Bad and Miss
            (Judgement::Miss, true)
        }
    }

    /// Judges a timing difference in milliseconds (for compatibility).
    pub fn judge_ms(&self, timing_diff_ms: f64) -> (Judgement, bool) {
        self.judge((timing_diff_ms * US_PER_MS as f64) as i64)
    }
}
