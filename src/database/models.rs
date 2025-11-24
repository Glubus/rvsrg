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
    pub id: i64,
    pub beatmapset_id: i64,
    pub path: String,
    pub difficulty_name: Option<String>,
    pub note_count: i32,
}
