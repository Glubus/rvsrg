//! Hit window timing configuration.



use crate::models::stats::Judgement;

/// Hit window timing thresholds in milliseconds.
#[derive(Debug, Clone, Copy)]
pub struct HitWindow {
    pub marv_ms: f64,
    pub perfect_ms: f64,
    pub great_ms: f64,
    pub good_ms: f64,
    pub bad_ms: f64,
    pub miss_ms: f64,
}

impl HitWindow {
    /// Default hit window values.
    pub fn new() -> Self {
        Self {
            marv_ms: 16.0,
            perfect_ms: 50.0,
            great_ms: 65.0,
            good_ms: 100.0,
            bad_ms: 150.0,
            miss_ms: 200.0,
        }
    }

    /// Crée une HitWindow basée sur l'OD (Overall Difficulty) d'OSU.
    /// Retourne `Self` au lieu d'un tuple.
    pub fn from_osu_od(od: f64) -> Self {
        Self {
            marv_ms: 16.0,                 // Fixe (selon ton code précédent)
            perfect_ms: 64.0 - (3.0 * od), // 300
            great_ms: 97.0 - (3.0 * od),   // 100
            good_ms: 127.0 - (3.0 * od),   // 50
            bad_ms: 151.0 - (3.0 * od),    // (Approximation Bad)
            miss_ms: 188.0 - (3.0 * od),   // (Seuil Miss)
        }
    }

    /// Crée une HitWindow basée sur le Judge Level d'Etterna (J4 = Standard)
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

        // Règle spéciale Etterna : Bad ne descend jamais sous 180ms
        let bad_calculated = (base_bad * scale).max(180.0);

        Self {
            marv_ms: base_marv * scale,
            perfect_ms: base_perf * scale,
            great_ms: base_great * scale,
            good_ms: base_good * scale,
            bad_ms: bad_calculated,
            miss_ms: 500.0, // Standard Etterna Miss window
        }
    }

    /// Constructeur utilitaire pour des valeurs custom complètes
    pub fn from_custom(marv: f64, perf: f64, great: f64, good: f64, bad: f64, miss: f64) -> Self {
        Self {
            marv_ms: marv,
            perfect_ms: perf,
            great_ms: great,
            good_ms: good,
            bad_ms: bad,
            miss_ms: miss,
        }
    }

    pub fn judge(&self, timing_diff_ms: f64) -> (Judgement, bool) {
        let abs_diff = timing_diff_ms.abs();

        // Si le timing dépasse la fenêtre de miss définie, c'est un Ghost Tap
        if abs_diff > self.miss_ms {
            return (Judgement::GhostTap, false);
        }

        if abs_diff <= self.marv_ms {
            (Judgement::Marv, true)
        } else if abs_diff <= self.perfect_ms {
            (Judgement::Perfect, true)
        } else if abs_diff <= self.great_ms {
            (Judgement::Great, true)
        } else if abs_diff <= self.good_ms {
            (Judgement::Good, true)
        } else if abs_diff <= self.bad_ms {
            (Judgement::Bad, true)
        } else {
            // Dans la zone entre Bad et Miss
            (Judgement::Miss, true)
        }
    }
}

