//! General skin metadata.

use serde::{Deserialize, Serialize};

/// General skin information (name, author, version, font)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinGeneral {
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub font: Option<String>,
}

impl Default for SkinGeneral {
    fn default() -> Self {
        Self {
            name: "Default Skin".to_string(),
            version: "1.0".to_string(),
            author: "System".to_string(),
            font: Some("font.ttf".to_string()),
        }
    }
}

