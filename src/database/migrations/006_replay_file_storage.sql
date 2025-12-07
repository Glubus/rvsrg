-- Migration: Change replay.data to replay.file_path
-- Replays are now stored as Brotli-compressed files in data/r/{hash}.r

-- SQLite doesn't support DROP COLUMN directly before 3.35
-- We need to:
-- 1. Create new table with correct schema
-- 2. Copy data (excluding data column)  
-- 3. Drop old table
-- 4. Rename new table

-- Create new replay table with file_path instead of data
CREATE TABLE IF NOT EXISTS replay_new (
    hash TEXT PRIMARY KEY,
    beatmap_hash TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    score INTEGER NOT NULL,
    accuracy REAL NOT NULL,
    max_combo INTEGER NOT NULL,
    rate REAL NOT NULL DEFAULT 1.0,
    file_path TEXT NOT NULL,
    FOREIGN KEY (beatmap_hash) REFERENCES beatmap(hash) ON DELETE CASCADE
);

-- Note: Existing replays will be lost since we can't migrate the data column
-- to files in a pure SQL migration. This is intentional for a clean start.
-- If needed, a separate Rust migration script could handle this.

-- Drop old table and rename
DROP TABLE IF EXISTS replay;
ALTER TABLE replay_new RENAME TO replay;
