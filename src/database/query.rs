//! Raw sqlx query helpers for the persistent database layer.

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

use crate::database::models::{
    Beatmap, BeatmapLight, BeatmapRating, BeatmapWithRatings, Beatmapset, BeatmapsetLight, Replay,
};
use crate::models::search::MenuSearchFilters;
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Clears beatmap tables (used during rescans).
///
/// Note: Replays are NOT deleted as they are user data.
pub async fn clear_all(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM beatmap_rating")
        .execute(pool)
        .await?;
    // Replays are preserved - they are valuable user data!
    sqlx::query("DELETE FROM beatmap").execute(pool).await?;
    sqlx::query("DELETE FROM beatmapset").execute(pool).await?;
    Ok(())
}

/// Inserts or updates a beatmapset record.
pub async fn insert_beatmapset(
    pool: &SqlitePool,
    path: &str,
    image_path: Option<&str>,
    artist: Option<&str>,
    title: Option<&str>,
) -> Result<i64, sqlx::Error> {
    // Check whether the beatmapset already exists.
    let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM beatmapset WHERE path = ?1")
        .bind(path)
        .fetch_optional(pool)
        .await?;

    match existing {
        Some(id) => {
            // Update existing row.
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
            // Insert a new row.
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

/// Inserts or updates a beatmap record.
pub async fn insert_beatmap(
    pool: &SqlitePool,
    beatmapset_id: i64,
    hash: &str,
    path: &str,
    difficulty_name: Option<&str>,
    note_count: i32,
    duration_ms: i32,
    nps: f64,
) -> Result<String, sqlx::Error> {
    // Check whether a beatmap already exists for the given hash.
    let existing: Option<String> = sqlx::query_scalar("SELECT hash FROM beatmap WHERE hash = ?1")
        .bind(hash)
        .fetch_optional(pool)
        .await?;

    match existing {
        Some(existing_hash) => {
            // Update the existing row.
            sqlx::query(
                "UPDATE beatmap SET beatmapset_id = ?1, path = ?2, difficulty_name = ?3, note_count = ?4, duration_ms = ?5, nps = ?6 WHERE hash = ?7"
            )
            .bind(beatmapset_id)
            .bind(path)
            .bind(difficulty_name)
            .bind(note_count)
            .bind(duration_ms)
            .bind(nps)
            .bind(&existing_hash)
            .execute(pool)
            .await?;
            Ok(existing_hash)
        }
        None => {
            // Insert a new row.
            sqlx::query(
                "INSERT INTO beatmap (hash, beatmapset_id, path, difficulty_name, note_count, duration_ms, nps) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            )
            .bind(hash)
            .bind(beatmapset_id)
            .bind(path)
            .bind(difficulty_name)
            .bind(note_count)
            .bind(duration_ms)
            .bind(nps)
            .execute(pool)
            .await?;
            Ok(hash.to_string())
        }
    }
}

/// Inserts or updates a rating entry for a beatmap.
pub async fn upsert_beatmap_rating(
    pool: &SqlitePool,
    beatmap_hash: &str,
    name: &str,
    overall: f64,
    stream: f64,
    jumpstream: f64,
    handstream: f64,
    stamina: f64,
    jackspeed: f64,
    chordjack: f64,
    technical: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO beatmap_rating (
            beatmap_hash, name, overall, stream, jumpstream, handstream, stamina, jackspeed, chordjack, technical
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(beatmap_hash, name) DO UPDATE SET
            overall = excluded.overall,
            stream = excluded.stream,
            jumpstream = excluded.jumpstream,
            handstream = excluded.handstream,
            stamina = excluded.stamina,
            jackspeed = excluded.jackspeed,
            chordjack = excluded.chordjack,
            technical = excluded.technical",
    )
    .bind(beatmap_hash)
    .bind(name)
    .bind(overall)
    .bind(stream)
    .bind(jumpstream)
    .bind(handstream)
    .bind(stamina)
    .bind(jackspeed)
    .bind(chordjack)
    .bind(technical)
    .execute(pool)
    .await?;
    Ok(())
}

/// Retrieves every rating for a specific beatmap.
pub async fn get_ratings_for_beatmap(
    pool: &SqlitePool,
    beatmap_hash: &str,
) -> Result<Vec<BeatmapRating>, sqlx::Error> {
    let ratings: Vec<BeatmapRating> = sqlx::query_as(
        "SELECT id, beatmap_hash, name, overall, stream, jumpstream, handstream, stamina, jackspeed, chordjack, technical
         FROM beatmap_rating WHERE beatmap_hash = ?1 ORDER BY name",
    )
    .bind(beatmap_hash)
    .fetch_all(pool)
    .await?;
    Ok(ratings)
}

/// Retrieves all ratings across the database.
pub async fn get_all_beatmap_ratings(pool: &SqlitePool) -> Result<Vec<BeatmapRating>, sqlx::Error> {
    let ratings: Vec<BeatmapRating> = sqlx::query_as(
        "SELECT id, beatmap_hash, name, overall, stream, jumpstream, handstream, stamina, jackspeed, chordjack, technical FROM beatmap_rating",
    )
    .fetch_all(pool)
    .await?;
    Ok(ratings)
}

/// Retrieves every beatmapset together with its beatmaps/ratings.
pub async fn get_all_beatmapsets(
    pool: &SqlitePool,
) -> Result<Vec<(Beatmapset, Vec<BeatmapWithRatings>)>, sqlx::Error> {
    let beatmapsets: Vec<Beatmapset> = sqlx::query_as(
        "SELECT id, path, image_path, artist, title FROM beatmapset ORDER BY artist, title",
    )
    .fetch_all(pool)
    .await?;

    let ratings = get_all_beatmap_ratings(pool).await?;
    let mut ratings_map: HashMap<String, Vec<BeatmapRating>> = HashMap::new();
    for rating in ratings {
        ratings_map
            .entry(rating.beatmap_hash.clone())
            .or_default()
            .push(rating);
    }

    let mut result = Vec::new();
    for beatmapset in beatmapsets {
        let beatmaps: Vec<Beatmap> = sqlx::query_as(
            "SELECT hash, beatmapset_id, path, difficulty_name, note_count, duration_ms, nps FROM beatmap WHERE beatmapset_id = ?1 ORDER BY difficulty_name"
        )
        .bind(beatmapset.id)
        .fetch_all(pool)
        .await?;

        let with_ratings = beatmaps
            .into_iter()
            .map(|beatmap| {
                let ratings = ratings_map.remove(&beatmap.hash).unwrap_or_default();
                BeatmapWithRatings::new(beatmap, ratings)
            })
            .collect();

        result.push((beatmapset, with_ratings));
    }

    Ok(result)
}

// ============================================================================
// PAGINATION QUERIES (new)
// ============================================================================

/// Counts the total number of beatmapsets.
pub async fn count_beatmapsets(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    let count: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM beatmapset")
        .fetch_optional(pool)
        .await?;
    Ok(count.unwrap_or(0) as i32)
}

/// Counts beatmapsets matching the given filters.
pub async fn count_beatmapsets_filtered(
    pool: &SqlitePool,
    filters: &MenuSearchFilters,
) -> Result<usize, sqlx::Error> {
    let query_text = filters.query.to_lowercase();
    let query_like = format!("%{}%", query_text);

    let min_duration_active = filters.min_duration_seconds.is_some() as i32;
    let min_duration_ms = filters
        .min_duration_seconds
        .map(|s| (s * 1000.0) as i32)
        .unwrap_or(0);

    let max_duration_active = filters.max_duration_seconds.is_some() as i32;
    let max_duration_ms = filters
        .max_duration_seconds
        .map(|s| (s * 1000.0) as i32)
        .unwrap_or(0);

    let count: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT bs.id)
        FROM beatmapset bs
        JOIN beatmap b ON b.beatmapset_id = bs.id
        WHERE
            (?1 = '' OR LOWER(bs.title) LIKE ?2 OR LOWER(bs.artist) LIKE ?2 OR LOWER(IFNULL(b.difficulty_name, '')) LIKE ?2)
            AND (?3 = 0 OR b.duration_ms >= ?4)
            AND (?5 = 0 OR b.duration_ms <= ?6)
        "#,
    )
    .bind(query_text.trim())
    .bind(query_like)
    .bind(min_duration_active)
    .bind(min_duration_ms)
    .bind(max_duration_active)
    .bind(max_duration_ms)
    .fetch_optional(pool)
    .await?;

    Ok(count.unwrap_or(0) as usize)
}

/// Retrieves a page of beatmapsets (lightweight, no ratings).
pub async fn get_beatmapsets_page(
    pool: &SqlitePool,
    offset: usize,
    limit: usize,
    filters: &MenuSearchFilters,
) -> Result<Vec<BeatmapsetLight>, sqlx::Error> {
    let query_text = filters.query.to_lowercase();
    let query_like = format!("%{}%", query_text);

    let min_duration_active = filters.min_duration_seconds.is_some() as i32;
    let min_duration_ms = filters
        .min_duration_seconds
        .map(|s| (s * 1000.0) as i32)
        .unwrap_or(0);

    let max_duration_active = filters.max_duration_seconds.is_some() as i32;
    let max_duration_ms = filters
        .max_duration_seconds
        .map(|s| (s * 1000.0) as i32)
        .unwrap_or(0);

    let beatmapsets: Vec<Beatmapset> = sqlx::query_as(
        r#"
        SELECT DISTINCT bs.id, bs.path, bs.image_path, bs.artist, bs.title
        FROM beatmapset bs
        JOIN beatmap b ON b.beatmapset_id = bs.id
        WHERE
            (?1 = '' OR LOWER(bs.title) LIKE ?2 OR LOWER(bs.artist) LIKE ?2 OR LOWER(IFNULL(b.difficulty_name, '')) LIKE ?2)
            AND (?3 = 0 OR b.duration_ms >= ?4)
            AND (?5 = 0 OR b.duration_ms <= ?6)
        ORDER BY bs.artist, bs.title
        LIMIT ?7 OFFSET ?8
        "#,
    )
    .bind(query_text.trim())
    .bind(query_like)
    .bind(min_duration_active)
    .bind(min_duration_ms)
    .bind(max_duration_active)
    .bind(max_duration_ms)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();

    for beatmapset in beatmapsets {
        let beatmaps: Vec<Beatmap> = sqlx::query_as(
            "SELECT hash, beatmapset_id, path, difficulty_name, note_count, duration_ms, nps FROM beatmap WHERE beatmapset_id = ?1 ORDER BY difficulty_name",
        )
        .bind(beatmapset.id)
        .fetch_all(pool)
        .await?;

        let light_beatmaps: Vec<BeatmapLight> =
            beatmaps.into_iter().map(BeatmapLight::from).collect();
        result.push(BeatmapsetLight::new(beatmapset, light_beatmaps));
    }

    Ok(result)
}

/// Retrieves a single beatmap by hash.
pub async fn get_beatmap_by_hash(
    pool: &SqlitePool,
    hash: &str,
) -> Result<Option<Beatmap>, sqlx::Error> {
    let beatmap: Option<Beatmap> = sqlx::query_as(
        "SELECT hash, beatmapset_id, path, difficulty_name, note_count, duration_ms, nps FROM beatmap WHERE hash = ?1",
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?;
    Ok(beatmap)
}

// ============================================================================
// SEARCH QUERIES (updated - no rating filter since ratings are calculated on-demand)
// ============================================================================

pub async fn search_beatmapsets(
    pool: &SqlitePool,
    filters: &MenuSearchFilters,
) -> Result<Vec<(Beatmapset, Vec<BeatmapWithRatings>)>, sqlx::Error> {
    let query_text = filters.query.to_lowercase();
    let query_like = format!("%{}%", query_text);
    let rating_column = filters.rating_metric.column_name();
    let rating_source = filters.rating_source.as_str();

    let min_rating_active = filters.min_rating.is_some() as i32;
    let min_rating_value = filters.min_rating.unwrap_or(0.0);
    let max_rating_active = filters.max_rating.is_some() as i32;
    let max_rating_value = filters.max_rating.unwrap_or(0.0);

    let min_duration_active = filters.min_duration_seconds.is_some() as i32;
    let min_duration_ms = filters
        .min_duration_seconds
        .map(|s| (s * 1000.0) as i32)
        .unwrap_or(0);

    let max_duration_active = filters.max_duration_seconds.is_some() as i32;
    let max_duration_ms = filters
        .max_duration_seconds
        .map(|s| (s * 1000.0) as i32)
        .unwrap_or(0);

    let sql = format!(
        r#"
        SELECT DISTINCT bs.id, bs.path, bs.image_path, bs.artist, bs.title
        FROM beatmapset bs
        JOIN beatmap b ON b.beatmapset_id = bs.id
        LEFT JOIN beatmap_rating br ON br.beatmap_hash = b.hash AND LOWER(br.name) = LOWER(?3)
        WHERE
            (?1 = '' OR LOWER(bs.title) LIKE ?2 OR LOWER(bs.artist) LIKE ?2 OR LOWER(IFNULL(b.difficulty_name, '')) LIKE ?2)
            AND (?4 = 0 OR IFNULL(br.{col}, 0) >= ?5)
            AND (?6 = 0 OR IFNULL(br.{col}, 0) <= ?7)
            AND (?8 = 0 OR b.duration_ms >= ?9)
            AND (?10 = 0 OR b.duration_ms <= ?11)
        ORDER BY bs.artist, bs.title
        LIMIT 500
        "#,
        col = rating_column
    );

    let beatmapsets: Vec<Beatmapset> = sqlx::query_as(&sql)
        .bind(query_text.trim())
        .bind(query_like)
        .bind(rating_source)
        .bind(min_rating_active)
        .bind(min_rating_value)
        .bind(max_rating_active)
        .bind(max_rating_value)
        .bind(min_duration_active)
        .bind(min_duration_ms)
        .bind(max_duration_active)
        .bind(max_duration_ms)
        .fetch_all(pool)
        .await?;

    let mut result = Vec::new();

    for beatmapset in beatmapsets {
        let beatmaps: Vec<Beatmap> = sqlx::query_as(
            "SELECT hash, beatmapset_id, path, difficulty_name, note_count, duration_ms, nps FROM beatmap WHERE beatmapset_id = ?1 ORDER BY difficulty_name",
        )
        .bind(beatmapset.id)
        .fetch_all(pool)
        .await?;

        let mut with_ratings = Vec::new();
        for beatmap in beatmaps {
            let ratings = get_ratings_for_beatmap(pool, &beatmap.hash).await?;
            with_ratings.push(BeatmapWithRatings::new(beatmap, ratings));
        }

        result.push((beatmapset, with_ratings));
    }

    Ok(result)
}

// ============================================================================
// REPLAY QUERIES
// ============================================================================

/// Inserts a replay and derives a deterministic hash.
pub async fn insert_replay(
    pool: &SqlitePool,
    beatmap_hash: &str,
    timestamp: i64,
    score: i32,
    accuracy: f64,
    max_combo: i32,
    rate: f64,
    data: &str,
) -> Result<String, sqlx::Error> {
    let hash_input = format!(
        "{}:{}:{}:{}:{}:{}:{}",
        beatmap_hash, timestamp, score, accuracy, max_combo, rate, data
    );
    let hash = format!("{:x}", md5::compute(hash_input));

    sqlx::query(
        "INSERT INTO replay (hash, beatmap_hash, timestamp, score, accuracy, max_combo, rate, data) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )
    .bind(&hash)
    .bind(beatmap_hash)
    .bind(timestamp)
    .bind(score)
    .bind(accuracy)
    .bind(max_combo)
    .bind(rate)
    .bind(data)
    .execute(pool)
    .await?;
    Ok(hash)
}

/// Retrieves all replays for a beatmap, sorted by rate then accuracy (best first).
pub async fn get_replays_for_beatmap(
    pool: &SqlitePool,
    beatmap_hash: &str,
) -> Result<Vec<Replay>, sqlx::Error> {
    let replays: Vec<Replay> = sqlx::query_as(
        "SELECT hash, beatmap_hash, timestamp, score, accuracy, max_combo, rate, data FROM replay WHERE beatmap_hash = ?1 ORDER BY rate DESC, accuracy DESC, timestamp DESC LIMIT 10"
    )
    .bind(beatmap_hash)
    .fetch_all(pool)
    .await?;
    Ok(replays)
}

/// Retrieves the top scores sorted by rate then accuracy (cross-beatmap).
pub async fn get_top_scores(pool: &SqlitePool, limit: i32) -> Result<Vec<Replay>, sqlx::Error> {
    let replays: Vec<Replay> = sqlx::query_as(
        "SELECT hash, beatmap_hash, timestamp, score, accuracy, max_combo, rate, data FROM replay ORDER BY rate DESC, accuracy DESC, timestamp DESC LIMIT ?1"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(replays)
}
