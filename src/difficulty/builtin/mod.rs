//! Built-in difficulty calculators (Etterna MinaCalc, osu! rosu-pp).

mod etterna;
mod osu;

pub use etterna::EtternaCalculator;
pub use osu::OsuCalculator;

