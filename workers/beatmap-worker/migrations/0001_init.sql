-- D1 initial schema for BeatmapBrowser
PRAGMA foreign_keys = ON;

-- Core maps table
CREATE TABLE IF NOT EXISTS maps (
  id TEXT PRIMARY KEY,
  song TEXT NOT NULL,
  artist TEXT NOT NULL,
  charter TEXT NOT NULL,
  charter_uid TEXT NOT NULL,
  description TEXT DEFAULT '',
  artist_list TEXT DEFAULT '',
  image INTEGER NOT NULL DEFAULT 0,
  upvotes INTEGER NOT NULL DEFAULT 0,
  upload_date TEXT NOT NULL,
  update_date TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_maps_song ON maps(song);
CREATE INDEX IF NOT EXISTS idx_maps_artist ON maps(artist);
CREATE INDEX IF NOT EXISTS idx_maps_charter ON maps(charter);
CREATE INDEX IF NOT EXISTS idx_maps_upload_date ON maps(upload_date DESC);

-- Level variants per map
CREATE TABLE IF NOT EXISTS map_difficulties (
  map_id TEXT NOT NULL,
  display TEXT NOT NULL,
  difficulty REAL NOT NULL,
  PRIMARY KEY(map_id, display),
  FOREIGN KEY(map_id) REFERENCES maps(id) ON DELETE CASCADE
);

-- Users table (minimal, extend as needed)
CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  name TEXT,
  created_at TEXT DEFAULT (datetime('now'))
);

-- Track which maps a user has upvoted
CREATE TABLE IF NOT EXISTS user_upvotes (
  user_id TEXT NOT NULL,
  map_id TEXT NOT NULL,
  PRIMARY KEY(user_id, map_id),
  FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY(map_id) REFERENCES maps(id) ON DELETE CASCADE
);
