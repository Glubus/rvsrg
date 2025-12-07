//! Error type for difficulty calculations.

/// Error type for difficulty calculation failures.
#[derive(Debug, Clone)]
pub enum CalcError {
    /// The beatmap data is invalid or missing.
    InvalidBeatmap(String),
    /// The calculator failed to compute the difficulty.
    CalculationFailed(String),
    /// The requested rate is not supported.
    UnsupportedRate(f64),
    /// Generic error with message.
    Other(String),
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::InvalidBeatmap(msg) => write!(f, "Invalid beatmap: {}", msg),
            CalcError::CalculationFailed(msg) => write!(f, "Calculation failed: {}", msg),
            CalcError::UnsupportedRate(rate) => write!(f, "Unsupported rate: {}", rate),
            CalcError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CalcError {}
