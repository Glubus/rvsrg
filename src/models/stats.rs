//! Hit statistics and judgement types.
//!
//! This module defines the judgement system used for scoring,
//! including accuracy calculation and hit statistics tracking.

/// RGBA colors for each judgement type.
#[derive(Clone)]
pub struct JudgementColors {
    pub marv: [f32; 4],
    pub perfect: [f32; 4],
    pub great: [f32; 4],
    pub good: [f32; 4],
    pub bad: [f32; 4],
    pub miss: [f32; 4],
    pub ghost_tap: [f32; 4],
}

impl JudgementColors {
    /// Creates default judgement colors.
    pub fn new() -> Self {
        Self {
            marv: [0.0, 1.0, 1.0, 1.0],      // Cyan
            perfect: [1.0, 1.0, 0.0, 1.0],   // Yellow
            great: [0.0, 1.0, 0.0, 1.0],     // Green
            good: [0.0, 0.0, 0.5, 1.0],      // Dark blue
            bad: [1.0, 0.41, 0.71, 1.0],     // Pink
            miss: [1.0, 0.0, 0.0, 1.0],      // Red
            ghost_tap: [0.5, 0.5, 0.5, 1.0], // Gray
        }
    }
}

impl Default for JudgementColors {
    fn default() -> Self {
        Self::new()
    }
}

/// Hit judgement types from best to worst.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Judgement {
    /// Perfect timing (best).
    Marv,
    /// Excellent timing.
    Perfect,
    /// Good timing.
    Great,
    /// Acceptable timing.
    Good,
    /// Poor timing.
    Bad,
    /// Missed note.
    Miss,
    /// Key press without a note (not counted as miss).
    GhostTap,
}

/// Accumulated hit statistics for a play session.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HitStats {
    pub marv: u32,
    pub perfect: u32,
    pub great: u32,
    pub good: u32,
    pub bad: u32,
    pub miss: u32,
    pub ghost_tap: u32,
}

impl HitStats {
    /// Creates empty hit statistics.
    pub fn new() -> Self {
        Self {
            marv: 0,
            perfect: 0,
            great: 0,
            good: 0,
            bad: 0,
            miss: 0,
            ghost_tap: 0,
        }
    }

    /// Calculates accuracy percentage (0-100).
    ///
    /// Uses a weighted formula:
    /// - Marv/Perfect: 100% weight (6 points)
    /// - Great: 66.7% weight (4 points)
    /// - Good: 33.3% weight (2 points)
    /// - Bad: 16.7% weight (1 point)
    /// - Miss: 0% weight (0 points)
    pub fn calculate_accuracy(&self) -> f64 {
        let total =
            (self.marv + self.perfect + self.great + self.good + self.bad + self.miss) as f64;

        if total == 0.0 {
            return 0.0;
        }

        let score = (self.marv + self.perfect) as f64 * 6.0
            + self.great as f64 * 4.0
            + self.good as f64 * 2.0
            + self.bad as f64;

        (score / (total * 6.0)) * 100.0
    }
}

impl Default for HitStats {
    fn default() -> Self {
        Self::new()
    }
}

