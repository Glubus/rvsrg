//! Converts ROX charts to rosu_map::Beatmap for difficulty calculation.
//!
//! This module enables PP/SSR calculation for all supported chart formats by:
//! 1. Decoding the source file with ROX (any format: .osu, .qua, .sm, .json)
//! 2. Encoding to .osu format in memory using OsuEncoder
//! 3. Parsing the .osu bytes with rosu_map for difficulty calculation

use rhythm_open_exchange::codec::formats::osu::OsuEncoder;
use rhythm_open_exchange::codec::{Encoder, auto_decode};
use rosu_map::Beatmap;
use std::path::Path;

/// Load any supported chart format and convert to rosu_map::Beatmap.
///
/// This allows difficulty calculators (MinaCalc, rosu-pp) that require
/// rosu_map::Beatmap to work with any chart format supported by ROX.
///
/// # Arguments
/// * `path` - Path to any supported chart file (.osu, .qua, .sm, .ssc, .json)
///
/// # Returns
/// A rosu_map::Beatmap that can be used with difficulty calculators.
///
/// # Errors
/// Returns an error if:
/// - The file cannot be decoded by ROX
/// - The chart cannot be encoded to .osu format
/// - The .osu content cannot be parsed by rosu_map
pub fn load_as_rosu_beatmap(path: &Path) -> Result<Beatmap, String> {
    // Step 1: Decode any format with ROX
    let chart =
        auto_decode(path).map_err(|e| format!("ROX decode failed for {:?}: {}", path, e))?;

    rox_chart_to_rosu(&chart)
}

/// Convert an already-decoded RoxChart to rosu_map::Beatmap.
///
/// This is useful when you already have a decoded chart (e.g., during scanning)
/// and want to calculate difficulty without re-reading the file.
pub fn rox_chart_to_rosu(chart: &rhythm_open_exchange::RoxChart) -> Result<Beatmap, String> {
    // Encode to .osu format string
    let osu_content =
        OsuEncoder::encode_to_string(chart).map_err(|e| format!("OsuEncoder failed: {}", e))?;

    // Parse .osu content with rosu_map
    let beatmap = Beatmap::from_bytes(osu_content.as_bytes())
        .map_err(|e| format!("rosu_map parse failed: {}", e))?;

    Ok(beatmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_osu_file() {
        // This would test with a real .osu file if available
        // For now, we just ensure the module compiles correctly
    }
}
