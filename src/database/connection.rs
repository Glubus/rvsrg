use crate::database::models::{Beatmap, Beatmapset};
use crate::database::query;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::path::{Path, PathBuf};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Ouvre ou crée la base de données
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        // S'assurer que le répertoire parent existe
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Err(sqlx::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Unable to create parent directory: {}", e),
                    )));
                }
            }
        }

        // Pour sqlx avec SQLite, convertir le chemin en chemin absolu
        let absolute_path = if db_path.is_absolute() {
            db_path.to_path_buf()
        } else {
            // Convertir en chemin absolu depuis le répertoire de travail courant
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(db_path)
        };

        // Utiliser SqliteConnectOptions directement avec le chemin du fichier
        // create_if_missing(true) crée automatiquement le fichier s'il n'existe pas
        let options = SqliteConnectOptions::new()
            .filename(&absolute_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;
        let db = Database { pool };
        db.init_schema().await?;
        Ok(db)
    }

    /// Initialise les tables si elles n'existent pas
    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // Table beatmapset (on garde l'id pour la compatibilité avec les relations)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS beatmapset (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                image_path TEXT,
                artist TEXT,
                title TEXT
            )",
        )
        .execute(&self.pool)
        .await?;

        // Table beatmap - hash MD5 comme clé primaire
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS beatmap (
                hash TEXT PRIMARY KEY,
                beatmapset_id INTEGER NOT NULL,
                path TEXT NOT NULL UNIQUE,
                difficulty_name TEXT,
                note_count INTEGER NOT NULL,
                FOREIGN KEY (beatmapset_id) REFERENCES beatmapset(id) ON DELETE CASCADE
            )",
        )
        .execute(&self.pool)
        .await?;

        // Table replay
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS replay (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                beatmap_hash TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                score INTEGER NOT NULL,
                accuracy REAL NOT NULL,
                max_combo INTEGER NOT NULL,
                rate REAL NOT NULL DEFAULT 1.0,
                data TEXT NOT NULL,
                FOREIGN KEY (beatmap_hash) REFERENCES beatmap(hash) ON DELETE CASCADE
            )",
        )
        .execute(&self.pool)
        .await?;

        // Migration: Ajouter la colonne rate si elle n'existe pas (pour les bases de données existantes)
        let _ = sqlx::query(
            "ALTER TABLE replay ADD COLUMN rate REAL NOT NULL DEFAULT 1.0"
        )
        .execute(&self.pool)
        .await; // On ignore l'erreur si la colonne existe déjà

        Ok(())
    }

    /// Retourne une référence au pool pour les requêtes
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Vide toutes les tables (pour rescan)
    pub async fn clear_all(&self) -> Result<(), sqlx::Error> {
        query::clear_all(&self.pool).await
    }

    /// Insère ou met à jour un beatmapset
    pub async fn insert_beatmapset(
        &self,
        path: &str,
        image_path: Option<&str>,
        artist: Option<&str>,
        title: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        query::insert_beatmapset(&self.pool, path, image_path, artist, title).await
    }

    /// Insère ou met à jour une beatmap
    pub async fn insert_beatmap(
        &self,
        beatmapset_id: i64,
        hash: &str,
        path: &str,
        difficulty_name: Option<&str>,
        note_count: i32,
    ) -> Result<String, sqlx::Error> {
        query::insert_beatmap(&self.pool, beatmapset_id, hash, path, difficulty_name, note_count).await
    }

    /// Récupère tous les beatmapsets avec leurs beatmaps
    pub async fn get_all_beatmapsets(
        &self,
    ) -> Result<Vec<(Beatmapset, Vec<Beatmap>)>, sqlx::Error> {
        query::get_all_beatmapsets(&self.pool).await
    }

    /// Compte le nombre total de beatmapsets
    pub async fn count_beatmapsets(&self) -> Result<i32, sqlx::Error> {
        query::count_beatmapsets(&self.pool).await
    }

    /// Insère un replay
    pub async fn insert_replay(
        &self,
        beatmap_hash: &str,
        timestamp: i64,
        score: i32,
        accuracy: f64,
        max_combo: i32,
        rate: f64,
        data: &str,
    ) -> Result<i64, sqlx::Error> {
        query::insert_replay(&self.pool, beatmap_hash, timestamp, score, accuracy, max_combo, rate, data).await
    }

    /// Récupère tous les replays pour une beatmap
    pub async fn get_replays_for_beatmap(
        &self,
        beatmap_hash: &str,
    ) -> Result<Vec<crate::database::models::Replay>, sqlx::Error> {
        query::get_replays_for_beatmap(&self.pool, beatmap_hash).await
    }

    /// Récupère les meilleurs scores par accuracy
    pub async fn get_top_scores(
        &self,
        limit: i32,
    ) -> Result<Vec<crate::database::models::Replay>, sqlx::Error> {
        query::get_top_scores(&self.pool, limit).await
    }
}
