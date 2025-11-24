use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Beatmapset {
    pub id: i64,
    pub path: String,
    pub image_path: Option<String>,
    pub artist: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Beatmap {
    pub hash: String,  // MD5 hash comme clé primaire
    pub beatmapset_id: i64,
    pub path: String,
    pub difficulty_name: Option<String>,
    pub note_count: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct Replay {
    pub id: i64,
    pub beatmap_hash: String,  // Référence vers beatmap.hash
    pub timestamp: i64,  // Timestamp Unix de la partie
    pub score: i32,
    pub accuracy: f64,
    pub max_combo: i32,
    pub rate: f64,  // Rate de la partie (1.0 = normal, 1.5 = 1.5x, etc.)
    pub data: String,  // JSON ou autre format pour les données de replay
}
