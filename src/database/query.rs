use crate::database::models::{Beatmap, Beatmapset};
use sqlx::SqlitePool;

/// Vide toutes les tables (pour rescan)
pub async fn clear_all(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM beatmap").execute(pool).await?;
    sqlx::query("DELETE FROM beatmapset").execute(pool).await?;
    Ok(())
}

/// Insère ou met à jour un beatmapset
pub async fn insert_beatmapset(
    pool: &SqlitePool,
    path: &str,
    image_path: Option<&str>,
    artist: Option<&str>,
    title: Option<&str>,
) -> Result<i64, sqlx::Error> {
    // Vérifier si le beatmapset existe déjà
    let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM beatmapset WHERE path = ?1")
        .bind(path)
        .fetch_optional(pool)
        .await?;

    match existing {
        Some(id) => {
            // Mettre à jour
            sqlx::query(
                "UPDATE beatmapset SET image_path = ?1, artist = ?2, title = ?3 WHERE id = ?4",
            )
            .bind(image_path)
            .bind(artist)
            .bind(title)
            .bind(id)
            .execute(pool)
            .await?;
            Ok(id)
        }
        None => {
            // Insérer
            let result = sqlx::query(
                "INSERT INTO beatmapset (path, image_path, artist, title) VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(path)
            .bind(image_path)
            .bind(artist)
            .bind(title)
            .execute(pool)
            .await?;
            Ok(result.last_insert_rowid())
        }
    }
}

/// Insère ou met à jour une beatmap
pub async fn insert_beatmap(
    pool: &SqlitePool,
    beatmapset_id: i64,
    path: &str,
    difficulty_name: Option<&str>,
    note_count: i32,
) -> Result<i64, sqlx::Error> {
    // Vérifier si la beatmap existe déjà
    let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM beatmap WHERE path = ?1")
        .bind(path)
        .fetch_optional(pool)
        .await?;

    match existing {
        Some(id) => {
            // Mettre à jour
            sqlx::query(
                "UPDATE beatmap SET beatmapset_id = ?1, difficulty_name = ?2, note_count = ?3 WHERE id = ?4"
            )
            .bind(beatmapset_id)
            .bind(difficulty_name)
            .bind(note_count)
            .bind(id)
            .execute(pool)
            .await?;
            Ok(id)
        }
        None => {
            // Insérer
            let result = sqlx::query(
                "INSERT INTO beatmap (beatmapset_id, path, difficulty_name, note_count) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(beatmapset_id)
            .bind(path)
            .bind(difficulty_name)
            .bind(note_count)
            .execute(pool)
            .await?;
            Ok(result.last_insert_rowid())
        }
    }
}

/// Récupère tous les beatmapsets avec leurs beatmaps
pub async fn get_all_beatmapsets(
    pool: &SqlitePool,
) -> Result<Vec<(Beatmapset, Vec<Beatmap>)>, sqlx::Error> {
    let beatmapsets: Vec<Beatmapset> = sqlx::query_as(
        "SELECT id, path, image_path, artist, title FROM beatmapset ORDER BY artist, title",
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for beatmapset in beatmapsets {
        let beatmaps: Vec<Beatmap> = sqlx::query_as(
            "SELECT id, beatmapset_id, path, difficulty_name, note_count FROM beatmap WHERE beatmapset_id = ?1 ORDER BY difficulty_name"
        )
        .bind(beatmapset.id)
        .fetch_all(pool)
        .await?;

        result.push((beatmapset, beatmaps));
    }

    Ok(result)
}

/// Compte le nombre total de beatmapsets
pub async fn count_beatmapsets(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    let count: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM beatmapset")
        .fetch_optional(pool)
        .await?;
    Ok(count.unwrap_or(0) as i32)
}
