use uuid::Uuid;
use worker::{Env, Error, Result};
use chrono::{DateTime, Utc};
use worker::d1::{D1Database, D1Result};
use worker::wasm_bindgen::JsValue;
use serde_json::Value as D1Value;

use crate::api::{BeatMap, LevelVariant};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Relevance,
    Upvotes,
    Newest,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SearchParams {
    pub query: String,
    pub min_upvotes: Option<u64>,
    pub difficulties: Vec<String>,
    pub sort: SortBy,
    pub page: u32,
    pub page_size: u32,
}

pub trait Database {
    async fn search_songs(&self, params: &SearchParams) -> Result<(Vec<BeatMap>, bool, u64)>;

    async fn get_map_by_id(&self, id: Uuid) -> Result<Option<BeatMap>>;

    async fn upvote_map(&self, user_id: &str, map_id: &str) -> Result<()>;

    async fn unvote_map(&self, user_id: &str, map_id: &str) -> Result<()>;

    async fn usersongs(&self, user_id: &str) -> Result<Vec<BeatMap>>;

    async fn delete_map(&self, map_id: &str) -> Result<()>;

    async fn create_map(&self, new_map: &NewMap<'_>) -> Result<()>;
}

pub struct DatabaseBackend {
    db: D1Database,
}

pub const MAPS_TABLE: &str = "maps";
pub const DIFFICULTIES_TABLE: &str = "map_difficulties";
pub const USER_UPVOTES_TABLE: &str = "user_upvotes";

#[derive(Debug)]
pub struct NewMap<'a> {
    pub id: Uuid,
    pub song: &'a str,
    pub artist: &'a str,
    pub charter: &'a str,
    pub charter_uid: Uuid,
    pub description: &'a str,
    pub artist_list: &'a str,
    pub image: bool,
}

impl DatabaseBackend {
    pub fn from_env(env: &Env) -> Result<Self> {
        let db = env.d1("beatblockbrowser")?;
        Ok(Self { db })
    }

    fn parse_uuid(value: &D1Value) -> Option<Uuid> {
        match value {
            D1Value::String(s) => Uuid::parse_str(s).ok(),
            _ => None,
        }
    }

    fn parse_bool(value: &D1Value) -> Option<bool> {
        match value {
            D1Value::Bool(b) => Some(*b),
            D1Value::Number(n) => n.as_f64().map(|f| f != 0.0),
            D1Value::String(s) => Some(s == "1" || s.eq_ignore_ascii_case("true")),
            _ => None,
        }
    }

    fn parse_u64(value: &D1Value) -> Option<u64> {
        match value {
            D1Value::Number(n) => n.as_u64(),
            D1Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    fn parse_f64(value: &D1Value) -> Option<f64> {
        match value {
            D1Value::Number(n) => n.as_f64(),
            D1Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    fn parse_dt(value: &D1Value) -> Option<DateTime<Utc>> {
        if let D1Value::String(s) = value {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Some(dt.with_timezone(&Utc));
            }
        }
        None
    }

    fn rows(result: D1Result) -> Vec<std::collections::BTreeMap<String, D1Value>> {
        match result.results::<std::collections::BTreeMap<String, D1Value>>() {
            Ok(rows) => rows,
            Err(_) => Vec::new(),
        }
    }

    fn group_maps(
        rows: Vec<std::collections::BTreeMap<String, D1Value>>,
    ) -> Vec<BeatMap> {
        use std::collections::BTreeMap;
        let mut grouped: BTreeMap<Uuid, BeatMap> = BTreeMap::new();
        for row in rows {
            let id = row.get("id").and_then(Self::parse_uuid).unwrap_or_else(Uuid::nil);
            let entry = grouped.entry(id).or_insert_with(|| BeatMap {
                id,
                song: match row.get("song") { Some(D1Value::String(s)) => s.clone(), _ => String::new() },
                artist: match row.get("artist") { Some(D1Value::String(s)) => s.clone(), _ => String::new() },
                charter: match row.get("charter") { Some(D1Value::String(s)) => s.clone(), _ => String::new() },
                charter_uid: row.get("charter_uid").and_then(Self::parse_uuid).unwrap_or_else(Uuid::nil),
                difficulties: Vec::new(),
                description: match row.get("description") { Some(D1Value::String(s)) => s.clone(), _ => String::new() },
                artist_list: match row.get("artist_list") { Some(D1Value::String(s)) => s.clone(), _ => String::new() },
                image: row.get("image").and_then(Self::parse_bool).unwrap_or(false),
                upvotes: row.get("upvotes").and_then(Self::parse_u64).unwrap_or(0),
                upload_date: row.get("upload_date").and_then(Self::parse_dt).unwrap_or_else(Utc::now),
                update_date: row.get("update_date").and_then(Self::parse_dt).unwrap_or_else(Utc::now),
            });
            if let (Some(display), Some(difficulty)) = (
                match row.get("level_display") { Some(D1Value::String(s)) => Some(s.clone()), _ => None },
                row.get("level_difficulty").and_then(Self::parse_f64),
            ) {
                entry.difficulties.push(LevelVariant { display, difficulty });
            }
        }
        grouped.into_values().collect()
    }
}

impl Database for DatabaseBackend {
    async fn search_songs(&self, params: &SearchParams) -> Result<(Vec<BeatMap>, bool, u64)> {
        let trimmed = params.query.trim().to_string();
        let mut idx: usize = 1;
        let mut where_parts: Vec<String> = Vec::new();
        let mut bind_values: Vec<JsValue> = Vec::new();

        // Text filter
        let text_like = if !trimmed.is_empty() {
            let p1 = format!("?{}", idx); idx += 1;
            let p2 = format!("?{}", idx); idx += 1;
            let p3 = format!("?{}", idx); idx += 1;
            bind_values.push(JsValue::from_str(&format!("%{}%", trimmed)));
            bind_values.push(JsValue::from_str(&format!("%{}%", trimmed)));
            bind_values.push(JsValue::from_str(&format!("%{}%", trimmed)));
            Some(format!("(m.song LIKE {p1} OR m.artist LIKE {p2} OR m.charter LIKE {p3})"))
        } else { None };
        if let Some(part) = text_like { where_parts.push(part); }

        // Min upvotes
        if let Some(minu) = params.min_upvotes {
            let p = format!("?{}", idx); idx += 1;
            where_parts.push(format!("m.upvotes >= {p}"));
            bind_values.push(JsValue::from_f64(minu as f64));
        }

        // Difficulties IN list
        if !params.difficulties.is_empty() {
            let mut placeholders: Vec<String> = Vec::new();
            for d in &params.difficulties {
                let p = format!("?{}", idx); idx += 1;
                placeholders.push(p);
                bind_values.push(JsValue::from_str(d));
            }
            where_parts.push(format!(
                "EXISTS (SELECT 1 FROM {DIFFICULTIES_TABLE} dd WHERE dd.map_id = m.id AND dd.display IN ({}))",
                placeholders.join(", ")
            ));
        }

        let where_sql = if where_parts.is_empty() { String::new() } else { format!("WHERE {}", where_parts.join(" AND ")) };

        // Sorting
        let (order_sql, relevance_binds): (String, Vec<JsValue>) = match params.sort {
            SortBy::Upvotes => ("ORDER BY m.upvotes DESC, m.upload_date DESC".to_string(), vec![]),
            SortBy::Newest => ("ORDER BY m.upload_date DESC".to_string(), vec![]),
            SortBy::Relevance => {
                if trimmed.is_empty() {
                    ("ORDER BY m.upload_date DESC".to_string(), vec![])
                } else {
                    let pr1 = format!("?{}", idx); idx += 1;
                    let pr2 = format!("?{}", idx); idx += 1;
                    let pr3 = format!("?{}", idx); idx += 1;
                    (
                        format!("ORDER BY (CASE WHEN m.song LIKE {pr1} THEN 3 WHEN m.artist LIKE {pr2} THEN 2 WHEN m.charter LIKE {pr3} THEN 1 ELSE 0 END) DESC, m.upload_date DESC"),
                        vec![
                            JsValue::from_str(&format!("%{}%", trimmed)),
                            JsValue::from_str(&format!("%{}%", trimmed)),
                            JsValue::from_str(&format!("%{}%", trimmed)),
                        ],
                    )
                }
            }
        };
        bind_values.extend(relevance_binds);

        let limit = (params.page_size + 1) as i64; // one extra to detect has_more
        let offset = (params.page as i64) * (params.page_size as i64);

        let sql = format!(
            "SELECT m.id, m.song, m.artist, m.charter, m.charter_uid, m.description, m.artist_list, m.image, m.upvotes, m.upload_date, m.update_date, d.display AS level_display, d.difficulty AS level_difficulty \
             FROM {MAPS_TABLE} m \
             LEFT JOIN {DIFFICULTIES_TABLE} d ON d.map_id = m.id \
             {where_sql} \
             {order_sql} \
             LIMIT {limit} OFFSET {offset}"
        );

        let mut stmt = self.db.prepare(&sql);
        if !bind_values.is_empty() {
            stmt = stmt.bind(&bind_values[..])?;
        }
        let result = stmt.all().await?;
        let mut maps = Self::group_maps(Self::rows(result));
        let has_more = maps.len() as u32 > params.page_size;
        if has_more {
            maps.truncate(params.page_size as usize);
        }
        // Total count query
        let count_sql = format!(
            "SELECT COUNT(1) as cnt FROM {MAPS_TABLE} m {where_sql}"
        );
        let mut count_stmt = self.db.prepare(&count_sql);
        if !bind_values.is_empty() {
            // Rebuild binds but exclude relevance-only binds at the end (3 values)
            let mut count_binds: Vec<JsValue> = bind_values.clone();
            if matches!(params.sort, SortBy::Relevance) && !trimmed.is_empty() {
                // remove last three
                for _ in 0..3 { count_binds.pop(); }
            }
            count_stmt = count_stmt.bind(&count_binds[..])?;
        }
        let count_res = count_stmt.all().await?;
        let total_count: u64 = match count_res.results::<std::collections::BTreeMap<String, D1Value>>() {
            Ok(rows) => rows
                .get(0)
                .and_then(|r| r.get("cnt"))
                .and_then(Self::parse_u64)
                .unwrap_or(0),
            Err(_) => 0,
        };
        Ok((maps, has_more, total_count))
    }

    async fn get_map_by_id(&self, id: Uuid) -> Result<Option<BeatMap>> {
        let sql = format!(
            "SELECT m.id, m.song, m.artist, m.charter, m.charter_uid, m.description, m.artist_list, m.image, m.upvotes, m.upload_date, m.update_date, d.display AS level_display, d.difficulty AS level_difficulty FROM {MAPS_TABLE} m LEFT JOIN {DIFFICULTIES_TABLE} d ON d.map_id = m.id WHERE m.id = ?1"
        );
        let stmt = self.db.prepare(&sql);
        let result = stmt.bind(&[JsValue::from_str(&id.to_string())])?.all().await?;
        let mut maps = Self::group_maps(Self::rows(result));
        Ok(maps.pop())
    }

    async fn upvote_map(&self, user_id: &str, map_id: &str) -> Result<()> {
        let mut statements = Vec::new();
        statements.push(
            self.db
                .prepare(&format!(
                    "INSERT OR IGNORE INTO {USER_UPVOTES_TABLE}(user_id, map_id) VALUES (?1, ?2)"
                ))
                .bind(&[JsValue::from_str(&user_id), JsValue::from_str(&map_id)])?,
        );
        statements.push(
            self.db
                .prepare(&format!(
                    "UPDATE {MAPS_TABLE} SET upvotes = upvotes + 1 WHERE id = ?1"
                ))
                .bind(&[JsValue::from_str(&map_id)])?,
        );
        self
            .db
            .batch(statements)
            .await
            .map_err(|e| Error::RustError(format!("d1 upvote tx: {e}")))?;
        Ok(())
    }

    async fn unvote_map(&self, user_id: &str, map_id: &str) -> Result<()> {
        let mut statements = Vec::new();
        statements.push(
            self.db
                .prepare(&format!(
                    "DELETE FROM {USER_UPVOTES_TABLE} WHERE user_id = ?1 AND map_id = ?2"
                ))
                .bind(&[JsValue::from_str(&user_id), JsValue::from_str(&map_id)])?,
        );
        statements.push(
            self.db
                .prepare(&format!(
                    "UPDATE {MAPS_TABLE} SET upvotes = CASE WHEN upvotes > 0 THEN upvotes - 1 ELSE 0 END WHERE id = ?1"
                ))
                .bind(&[JsValue::from_str(&map_id)])?,
        );
        self
            .db
            .batch(statements)
            .await
            .map_err(|e| Error::RustError(format!("d1 unvote tx: {e}")))?;
        Ok(())
    }

    async fn usersongs(&self, user_id: &str) -> Result<Vec<BeatMap>> {
        let sql = format!(
            "SELECT m.id, m.song, m.artist, m.charter, m.charter_uid, m.description, m.artist_list, m.image, m.upvotes, m.upload_date, m.update_date, d.display AS level_display, d.difficulty AS level_difficulty FROM {MAPS_TABLE} m LEFT JOIN {DIFFICULTIES_TABLE} d ON d.map_id = m.id WHERE m.charter_uid = ?1 ORDER BY m.upload_date DESC LIMIT 100"
        );
        let stmt = self.db.prepare(&sql);
        let result = stmt.bind(&[JsValue::from_str(&user_id.to_string())])?.all().await?;
        Ok(Self::group_maps(Self::rows(result)))
    }

    async fn delete_map(&self, map_id: &str) -> Result<()> {
        let mut statements = Vec::new();
        statements.push(
            self.db
                .prepare(&format!("DELETE FROM {DIFFICULTIES_TABLE} WHERE map_id = ?1"))
                .bind(&[JsValue::from_str(&map_id)])?,
        );
        statements.push(
            self.db
                .prepare(&format!("DELETE FROM {USER_UPVOTES_TABLE} WHERE map_id = ?1"))
                .bind(&[JsValue::from_str(&map_id)])?,
        );
        statements.push(
            self.db
                .prepare(&format!("DELETE FROM {MAPS_TABLE} WHERE id = ?1"))
                .bind(&[JsValue::from_str(&map_id)])?,
        );
        self
            .db
            .batch(statements)
            .await
            .map_err(|e| Error::RustError(format!("d1 delete tx: {e}")))?;
        Ok(())
    }

    async fn create_map(&self, new_map: &NewMap<'_>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let stmt = self.db.prepare(&format!(
            "INSERT INTO {MAPS_TABLE}(id, song, artist, charter, charter_uid, description, artist_list, image, upvotes, upload_date, update_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?9)"
        ));
        stmt.bind(&[
            JsValue::from_str(&new_map.id.to_string()),
            JsValue::from_str(new_map.song),
            JsValue::from_str(new_map.artist),
            JsValue::from_str(new_map.charter),
            JsValue::from_str(&new_map.charter_uid.to_string()),
            JsValue::from_str(new_map.description),
            JsValue::from_str(new_map.artist_list),
            JsValue::from_f64(if new_map.image { 1.0 } else { 0.0 }),
            JsValue::from_str(&now),
        ])?.run().await.map_err(|e| Error::RustError(format!("d1 insert map: {e}")))?;
        Ok(())
    }
}
