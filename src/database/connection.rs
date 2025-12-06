//! Database connection helpers built on top of sqlx/SQLite.

#![allow(dead_code)]

use crate::database::models::{
    Beatmap, BeatmapLight, BeatmapRating, BeatmapWithRatings, Beatmapset, BeatmapsetLight,
};
use crate::database::query;
use crate::models::search::MenuSearchFilters;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::path::{Path, PathBuf};

const MIGRATION_CREATE_BEATMAPSET: &str = include_str!("migrations/001_create_beatmapset.sql");
const MIGRATION_CREATE_BEATMAP: &str = include_str!("migrations/002_create_beatmap.sql");
const MIGRATION_CREATE_REPLAY: &str = include_str!("migrations/003_create_replay.sql");
const MIGRATION_CREATE_BEATMAP_RATING: &str =
    include_str!("migrations/005_create_beatmap_rating.sql");

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Opens (or creates) the SQLite database file.
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        // Ensure the parent directory exists.
        if let Some(parent) = db_path.parent()
            && !parent.exists()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            return Err(sqlx::Error::Io(std::io::Error::other(format!(
                "Unable to create parent directory: {}",
                e
            ))));
        }

        // sqlx prefers absolute paths for SQLite.
        let absolute_path = if db_path.is_absolute() {
            db_path.to_path_buf()
        } else {
            // Derive an absolute path from the current working directory.
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(db_path)
        };

        // Use SqliteConnectOptions directly on the resolved file path and auto-create.
        let options = SqliteConnectOptions::new()
            .filename(&absolute_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;
        let db = Database { pool };
        db.init_schema().await?;
        Ok(db)
    }

    /// Creates missing tables by replaying the embedded migrations.
    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        for migration in [
            MIGRATION_CREATE_BEATMAPSET,
            MIGRATION_CREATE_BEATMAP,
            MIGRATION_CREATE_REPLAY,
            MIGRATION_CREATE_BEATMAP_RATING,
        ] {
            sqlx::query(migration).execute(&self.pool).await?;
        }

        Ok(())
    }

    /// Returns the underlying sqlx connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Clears all tables (used during rescans).
    pub async fn clear_all(&self) -> Result<(), sqlx::Error> {
        query::clear_all(&self.pool).await
    }

    /// Inserts or updates a beatmapset row.
    pub async fn insert_beatmapset(
        &self,
        path: &str,
        image_path: Option<&str>,
        artist: Option<&str>,
        title: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        query::insert_beatmapset(&self.pool, path, image_path, artist, title).await
    }

    /// Inserts or updates a beatmap row.
    pub async fn insert_beatmap(
        &self,
        beatmapset_id: i64,
        hash: &str,
        path: &str,
        difficulty_name: Option<&str>,
        note_count: i32,
        duration_ms: i32,
        nps: f64,
    ) -> Result<String, sqlx::Error> {
        query::insert_beatmap(
            &self.pool,
            beatmapset_id,
            hash,
            path,
            difficulty_name,
            note_count,
            duration_ms,
            nps,
        )
        .await
    }

    /// Inserts or updates SR/ratings for a beatmap.
    pub async fn upsert_beatmap_rating(
        &self,
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
        query::upsert_beatmap_rating(
            &self.pool,
            beatmap_hash,
            name,
            overall,
            stream,
            jumpstream,
            handstream,
            stamina,
            jackspeed,
            chordjack,
            technical,
        )
        .await
    }

    /// Fetches all ratings for a beatmap.
    pub async fn get_ratings_for_beatmap(
        &self,
        beatmap_hash: &str,
    ) -> Result<Vec<BeatmapRating>, sqlx::Error> {
        query::get_ratings_for_beatmap(&self.pool, beatmap_hash).await
    }

    /// Fetches all ratings across every beatmap.
    pub async fn get_all_beatmap_ratings(&self) -> Result<Vec<BeatmapRating>, sqlx::Error> {
        query::get_all_beatmap_ratings(&self.pool).await
    }

    /// Fetches every beatmapset together with their beatmaps.
    pub async fn get_all_beatmapsets(
        &self,
    ) -> Result<Vec<(Beatmapset, Vec<BeatmapWithRatings>)>, sqlx::Error> {
        query::get_all_beatmapsets(&self.pool).await
    }

    /// Searches beatmapsets using the provided filters.
    pub async fn search_beatmapsets(
        &self,
        filters: &MenuSearchFilters,
    ) -> Result<Vec<(Beatmapset, Vec<BeatmapWithRatings>)>, sqlx::Error> {
        query::search_beatmapsets(&self.pool, filters).await
    }

    /// Counts beatmapsets in the database.
    pub async fn count_beatmapsets(&self) -> Result<i32, sqlx::Error> {
        query::count_beatmapsets(&self.pool).await
    }

    // ========================================================================
    // PAGINATION METHODS (new)
    // ========================================================================

    /// Counts beatmapsets matching the given filters.
    pub async fn count_beatmapsets_filtered(
        &self,
        filters: &MenuSearchFilters,
    ) -> Result<usize, sqlx::Error> {
        query::count_beatmapsets_filtered(&self.pool, filters).await
    }

    /// Retrieves a page of beatmapsets (lightweight, no ratings).
    pub async fn get_beatmapsets_page(
        &self,
        offset: usize,
        limit: usize,
        filters: &MenuSearchFilters,
    ) -> Result<Vec<BeatmapsetLight>, sqlx::Error> {
        query::get_beatmapsets_page(&self.pool, offset, limit, filters).await
    }

    /// Retrieves a single beatmap by hash.
    pub async fn get_beatmap_by_hash(&self, hash: &str) -> Result<Option<Beatmap>, sqlx::Error> {
        query::get_beatmap_by_hash(&self.pool, hash).await
    }

    // ========================================================================
    // REPLAY METHODS
    // ========================================================================

    /// Persists a replay row.
    pub async fn insert_replay(
        &self,
        beatmap_hash: &str,
        timestamp: i64,
        score: i32,
        accuracy: f64,
        max_combo: i32,
        rate: f64,
        data: &str,
    ) -> Result<String, sqlx::Error> {
        query::insert_replay(
            &self.pool,
            beatmap_hash,
            timestamp,
            score,
            accuracy,
            max_combo,
            rate,
            data,
        )
        .await
    }

    /// Retrieves all replays for a given beatmap hash.
    pub async fn get_replays_for_beatmap(
        &self,
        beatmap_hash: &str,
    ) -> Result<Vec<crate::database::models::Replay>, sqlx::Error> {
        query::get_replays_for_beatmap(&self.pool, beatmap_hash).await
    }

    /// Retrieves the top scores ordered by accuracy.
    pub async fn get_top_scores(
        &self,
        limit: i32,
    ) -> Result<Vec<crate::database::models::Replay>, sqlx::Error> {
        query::get_top_scores(&self.pool, limit).await
    }
}
