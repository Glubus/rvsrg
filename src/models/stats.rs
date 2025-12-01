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
    pub fn new() -> Self {
        Self {
            marv: [0.0, 1.0, 1.0, 1.0],
            perfect: [1.0, 1.0, 0.0, 1.0],
            great: [0.0, 1.0, 0.0, 1.0],
            good: [0.0, 0.0, 0.5, 1.0],
            bad: [1.0, 0.41, 0.71, 1.0],
            miss: [1.0, 0.0, 0.0, 1.0],
            ghost_tap: [0.5, 0.5, 0.5, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Judgement {
    Marv,
    Perfect,
    Great,
    Good,
    Bad,
    Miss,
    GhostTap,
}

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
