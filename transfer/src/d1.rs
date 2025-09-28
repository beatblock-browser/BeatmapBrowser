use anyhow::{Context, Error};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Transaction, TransactionBehavior};
use uuid::Uuid;

use crate::types::{BeatMap, LevelVariant, User};

pub struct D1 {
    conn: Connection,
}

impl D1 {
    pub fn connect(sqlite_path: &str) -> Result<Self, Error> {
        let conn = Connection::open(sqlite_path)
            .with_context(|| format!("Failed to open D1 SQLite at {}", sqlite_path))?;
        // Enforce foreign keys to match Cloudflare behavior
        conn.pragma_update(None, "foreign_keys", "ON")
            .context("Failed to enable foreign_keys pragma")?;
        Ok(Self { conn })
    }

    pub fn upsert_map(&mut self, map: &BeatMap) -> Result<(), Error> {
        let tx = self.begin_tx()?;
        D1::insert_map(&tx, map)?;
        D1::replace_difficulties(&tx, map.id, &map.difficulties)?;
        tx.commit().context("Commit map upsert transaction failed")?;
        Ok(())
    }

    pub fn upsert_user(&mut self, user: &User) -> Result<(), Error> {
        let tx = self.begin_tx()?;
        // Minimal users table: id, name, created_at
        tx.execute(
            "INSERT OR REPLACE INTO users (id, name, created_at) VALUES (?1, COALESCE((SELECT name FROM users WHERE id = ?1), ''), COALESCE((SELECT created_at FROM users WHERE id = ?1), datetime('now')))",
            params![user.id.to_string()],
        )
        .context("Upsert user failed")?;

        // Replace user_upvotes from source's upvoted list
        tx.execute(
            "DELETE FROM user_upvotes WHERE user_id = ?1",
            params![user.id.to_string()],
        )
        .context("Clearing existing user_upvotes failed")?;

        for map_id in &user.upvoted {
            // Only insert upvote if the map was present in the original dataset (now in D1)
            let mut exists_stmt = tx
                .prepare("SELECT 1 FROM maps WHERE id = ?1")
                .context("Prepare exists check for maps failed")?;
            let exists = exists_stmt
                .exists(params![map_id.to_string()])
                .context("Execute exists check for maps failed")?;
            if !exists {
                println!(
                    "Skipping upvote: user {} references missing map {} (not present in source)",
                    user.id, map_id
                );
                continue;
            }

            tx.execute(
                "INSERT OR IGNORE INTO user_upvotes (user_id, map_id) VALUES (?1, ?2)",
                params![user.id.to_string(), map_id.to_string()],
            )
            .with_context(|| format!("Insert user_upvotes for user {} failed", user.id))?;
        }

        tx.commit().context("Commit user upsert transaction failed")?;
        Ok(())
    }

    fn begin_tx(&mut self) -> Result<Transaction<'_>, Error> {
        self.conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("Begin transaction failed")
    }

    fn insert_map(tx: &Transaction<'_>, map: &BeatMap) -> Result<(), Error> {
        let now_upload: DateTime<Utc> = map.upload_date;
        let now_update: DateTime<Utc> = map.update_date;
        tx.execute(
            "INSERT OR REPLACE INTO maps (id, song, artist, charter, charter_uid, description, artist_list, image, upvotes, upload_date, update_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                map.id.to_string(),
                map.song,
                map.artist,
                map.charter,
                map.charter_uid.to_string(),
                map.description,
                map.artist_list,
                if map.image { 1 } else { 0 },
                map.upvotes as i64,
                now_upload.to_rfc3339(),
                now_update.to_rfc3339()
            ],
        )
        .with_context(|| format!("Insert map {} failed", map.id))?;
        Ok(())
    }

    fn replace_difficulties(
        tx: &Transaction<'_>,
        map_id: Uuid,
        difficulties: &[LevelVariant],
    ) -> Result<(), Error> {
        tx.execute(
            "DELETE FROM map_difficulties WHERE map_id = ?1",
            params![map_id.to_string()],
        )
        .with_context(|| format!("Clearing difficulties for {} failed", map_id))?;

        for diff in difficulties {
            tx.execute(
                "INSERT INTO map_difficulties (map_id, display, difficulty) VALUES (?1, ?2, ?3)",
                params![map_id.to_string(), diff.display, diff.difficulty],
            )
            .with_context(|| format!("Insert difficulty '{}' for {} failed", diff.display, map_id))?;
        }
        Ok(())
    }

    
}
