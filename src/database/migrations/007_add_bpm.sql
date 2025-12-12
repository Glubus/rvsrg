-- Add BPM column to beatmap table
-- This column stores the dominant BPM (the BPM that lasts the longest in the chart)
ALTER TABLE beatmap
ADD COLUMN bpm REAL DEFAULT 0.0;