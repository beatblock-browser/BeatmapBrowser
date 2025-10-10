use uuid::Uuid;
use worker::{Env, Error, Result};
use chrono::{DateTime, Utc};
use worker::d1::{D1Database, D1Result};
use worker::wasm_bindgen::JsValue;
use serde_json::Value as D1Value;
use std::collections::HashMap;

use crate::api::{BeatMap, LevelVariant};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Relevance,
    Upvotes,
    Newest,
}

impl DatabaseBackend {
    pub async fn set_user_moderator(&self, user_id: &str, is_mod: bool) -> Result<()> {
        let stmt = self.db.prepare("UPDATE users SET is_moderator = ?2 WHERE id = ?1");
        stmt.bind(&[JsValue::from_str(user_id), JsValue::from_f64(if is_mod { 1.0 } else { 0.0 })])?
            .run()
            .await
            .map_err(|e| Error::RustError(format!("d1 set moderator: {e}")))?;
        Ok(())
    }

    pub async fn is_user_moderator(&self, user_id: &str) -> Result<bool> {
        let stmt = self.db.prepare("SELECT is_moderator FROM users WHERE id = ?1");
        let res = stmt.bind(&[JsValue::from_str(user_id)])?.all().await?;
        let rows = Self::rows(res);
        let val = rows.first().and_then(|r| r.get("is_moderator")).and_then(Self::parse_bool).unwrap_or(false);
        Ok(val)
    }

    pub async fn set_map_deleted(&self, map_id: &str, deleted: bool, who: Option<&str>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let stmt = if deleted {
            self.db.prepare(&format!("UPDATE {MAPS_TABLE} SET deleted = 1, deleted_at = ?2, deleted_by = ?3 WHERE id = ?1"))
        } else {
            self.db.prepare(&format!("UPDATE {MAPS_TABLE} SET deleted = 0, deleted_at = NULL, deleted_by = NULL, update_date = ?2 WHERE id = ?1"))
        };
        let binds: Vec<JsValue> = if deleted {
            vec![JsValue::from_str(map_id), JsValue::from_str(&now), JsValue::from_str(who.unwrap_or(""))]
        } else {
            vec![JsValue::from_str(map_id), JsValue::from_str(&now)]
        };
        stmt.bind(&binds[..])?.run().await.map_err(|e| Error::RustError(format!("d1 set map deleted: {e}")))?;
        Ok(())
    }

    pub async fn is_map_deleted(&self, map_id: &str) -> Result<bool> {
        let stmt = self.db.prepare(&format!("SELECT deleted FROM {MAPS_TABLE} WHERE id = ?1"));
        let res = stmt.bind(&[JsValue::from_str(map_id)])?.all().await?;
        let rows = Self::rows(res);
        let val = rows.first().and_then(|r| r.get("deleted")).and_then(Self::parse_bool).unwrap_or(false);
        Ok(val)
    }
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

    async fn find_map_by_identity(
        &self,
        song: &str,
        artist: &str,
        charter: &str,
        charter_uid: Uuid,
    ) -> Result<Option<Uuid>>;

    async fn update_map_metadata(&self, updated: &UpdateMap<'_>) -> Result<()>;
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

#[derive(Debug)]
pub struct UpdateMap<'a> {
    pub id: Uuid,
    pub song: &'a str,
    pub artist: &'a str,
    pub charter: &'a str,
    pub description: &'a str,
    pub artist_list: &'a str,
    pub image: Option<bool>,
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
        result.results::<std::collections::BTreeMap<String, D1Value>>().unwrap_or_default()
    }

    fn group_maps(
        rows: Vec<std::collections::BTreeMap<String, D1Value>>,
    ) -> Vec<BeatMap> {
        // Preserve SQL order by maintaining a Vec with insertion order and an index map
        let mut maps: Vec<BeatMap> = Vec::new();
        let mut index_by_id: HashMap<Uuid, usize> = HashMap::new();
        for row in rows {
            let id = row.get("id").and_then(Self::parse_uuid).unwrap_or_else(Uuid::nil);
            let idx = match index_by_id.get(&id) {
                Some(i) => *i,
                None => {
                    let i = maps.len();
                    maps.push(BeatMap {
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
                    index_by_id.insert(id, i);
                    i
                }
            };
            if let (Some(display), Some(difficulty)) = (
                match row.get("level_display") { Some(D1Value::String(s)) => Some(s.clone()), _ => None },
                row.get("level_difficulty").and_then(Self::parse_f64),
            ) {
                maps[idx].difficulties.push(LevelVariant { display, difficulty });
            }
        }
        maps
    }
}

impl Database for DatabaseBackend {
    async fn search_songs(&self, params: &SearchParams) -> Result<(Vec<BeatMap>, bool, u64)> {
        let trimmed = params.query.trim().to_string();
        let mut idx: usize = 1;
        let mut where_parts: Vec<String> = Vec::new();
        // Separate bind vectors: WHERE binds are shared by data and count queries; relevance binds are only for data query
        let mut where_binds: Vec<JsValue> = Vec::new();
        let mut relevance_binds: Vec<JsValue> = Vec::new();

        // Exclude soft-deleted maps by default
        where_parts.push("m.deleted = 0".to_string());

        // Text filter: multi-term AND; for each term, require a match in any target field
        if !trimmed.is_empty() {
            let terms: Vec<&str> = trimmed.split_whitespace().filter(|t| !t.is_empty()).collect();
            for t in terms {
                let p1 = format!("?{}", idx); idx += 1;
                let p2 = format!("?{}", idx); idx += 1;
                let p3 = format!("?{}", idx); idx += 1;
                let p4 = format!("?{}", idx); idx += 1;
                where_binds.push(JsValue::from_str(&format!("%{}%", t)));
                where_binds.push(JsValue::from_str(&format!("%{}%", t)));
                where_binds.push(JsValue::from_str(&format!("%{}%", t)));
                where_binds.push(JsValue::from_str(&format!("%{}%", t)));
                where_parts.push(format!(
                    "(m.song LIKE {p1} OR m.artist LIKE {p2} OR m.charter LIKE {p3} OR m.artist_list LIKE {p4})"
                ));
            }
        }

        // Min upvotes
        if let Some(minu) = params.min_upvotes {
            let p = format!("?{}", idx); idx += 1;
            where_parts.push(format!("m.upvotes >= {p}"));
            where_binds.push(JsValue::from_f64(minu as f64));
        }

        // Difficulty buckets by numeric ranges: [1,2), [2,3), [3,4), [4,5); Special is outside [1,5)
        if !params.difficulties.is_empty() {
            // Build OR conditions for selected buckets
            let mut conds: Vec<String> = Vec::new();
            let mut has_easy = false;
            let mut has_hard = false;
            let mut has_challenge = false;
            let mut has_apoc = false;
            let mut has_special = false;
            // Define bucket ranges
            let easy_rng = "(dd.difficulty >= 1 AND dd.difficulty < 2)";
            let hard_rng = "(dd.difficulty >= 2 AND dd.difficulty < 3)";
            let challenge_rng = "(dd.difficulty >= 3 AND dd.difficulty < 4)";
            let apoc_rng = "(dd.difficulty >= 4 AND dd.difficulty < 5)";
            for d in &params.difficulties {
                let dl = d.trim().to_lowercase();
                match dl.as_str() {
                    "easy" => { if !has_easy { conds.push(easy_rng.to_string()); has_easy = true; } },
                    "hard" => { if !has_hard { conds.push(hard_rng.to_string()); has_hard = true; } },
                    "challenge" => { if !has_challenge { conds.push(challenge_rng.to_string()); has_challenge = true; } },
                    "apocraphyia" => { if !has_apoc { conds.push(apoc_rng.to_string()); has_apoc = true; } },
                    "special" => { if !has_special {
                        // Special: outside [1,5)
                        conds.push(format!("(dd.difficulty < 1 OR dd.difficulty >= 5)"));
                        has_special = true;
                    } },
                    _ => {}
                }
            }
            if !conds.is_empty() {
                where_parts.push(format!(
                    "EXISTS (SELECT 1 FROM {DIFFICULTIES_TABLE} dd WHERE dd.map_id = m.id AND ({}))",
                    conds.join(" OR ")
                ));
            }
        }

        let where_sql = if where_parts.is_empty() { String::new() } else { format!("WHERE {}", where_parts.join(" AND ")) };

        // Sorting
        let mut select_extras: String = String::new();
        let order_sql: String = match params.sort {
            SortBy::Upvotes => "ORDER BY m.upvotes DESC, m.upload_date DESC".to_string(),
            SortBy::Newest => "ORDER BY m.upload_date DESC".to_string(),
            SortBy::Relevance => {
                if trimmed.is_empty() {
                    "ORDER BY m.upload_date DESC".to_string()
                } else {
                    // Contains patterns for bucket scoring
                    let sc = format!("?{}", idx); idx += 1; // song contains
                    let ac = format!("?{}", idx); idx += 1; // artist contains
                    let cc = format!("?{}", idx); idx += 1; // charter contains
                    let alc = format!("?{}", idx); idx += 1; // artist_list contains (low weight)
                    relevance_binds.push(JsValue::from_str(&format!("%{}%", trimmed)));
                    relevance_binds.push(JsValue::from_str(&format!("%{}%", trimmed)));
                    relevance_binds.push(JsValue::from_str(&format!("%{}%", trimmed)));
                    relevance_binds.push(JsValue::from_str(&format!("%{}%", trimmed)));

                    // Prefix patterns for additional boost
                    let sp = format!("?{}", idx); idx += 1; // song prefix
                    let ap = format!("?{}", idx); idx += 1; // artist prefix
                    let cp = format!("?{}", idx); idx += 1; // charter prefix
                    let alp = format!("?{}", idx); idx += 1; // artist_list prefix
                    relevance_binds.push(JsValue::from_str(&format!("{}%", trimmed)));
                    relevance_binds.push(JsValue::from_str(&format!("{}%", trimmed)));
                    relevance_binds.push(JsValue::from_str(&format!("{}%", trimmed)));
                    relevance_binds.push(JsValue::from_str(&format!("{}%", trimmed)));

                    // Exact artist equality boost (case-insensitive)
                    let ae = format!("?{}", idx);
                    relevance_binds.push(JsValue::from_str(&trimmed));

                    // Compute relevance flags in SELECT and later order by a weighted composite score using these flags
                    select_extras = format!(
                        ", \
                           (CASE WHEN m.song LIKE {sc} THEN 1 ELSE 0 END) AS _sl,\
                           (CASE WHEN m.artist LIKE {ac} THEN 1 ELSE 0 END) AS _al,\
                           (CASE WHEN m.charter LIKE {cc} THEN 1 ELSE 0 END) AS _cl,\
                           (CASE WHEN m.artist_list LIKE {alc} THEN 1 ELSE 0 END) AS _all,\
                           (CASE WHEN m.song LIKE {sp} THEN 1 ELSE 0 END) AS _sp,\
                           (CASE WHEN m.artist LIKE {ap} THEN 1 ELSE 0 END) AS _ap,\
                           (CASE WHEN m.charter LIKE {cp} THEN 1 ELSE 0 END) AS _cp,\
                           (CASE WHEN m.artist_list LIKE {alp} THEN 1 ELSE 0 END) AS _alp,\
                           (CASE WHEN m.artist = {ae} COLLATE NOCASE THEN 1 ELSE 0 END) AS _ae"
                    );
                    // Weighted composite score with diminishing returns on upvotes and no double counting:
                    // - Field bucket (no double count): max(song*2, artist*3, charter_or_list*1) * 100
                    //   where charter_or_list = 1 if either charter or artist_list matches
                    // - Prefix boosts (no double count): max(song*18, artist*30, charter*8, artist_list*4)
                    // - Exact artist equality: +70
                    // - Upvotes: piecewise linear to approximate sqrt/log (diminishing returns)
                    //   upvote_score = CASE
                    //       up<=40:  up*2.2
                    //       up<=100: 40*2.2 + (up-40)*0.7
                    //       else:    40*2.2 + 60*0.7 + (up-100)*0.25
                    // - Recency reduced boost
                    let score_expr = "((MAX((_sl*2), (_al*3), (CASE WHEN (_cl + _all) > 0 THEN 1 ELSE 0 END))*100) \
                                        + (MAX((_sp*18), (_ap*30), (_cp*8), (_alp*4))) \
                                        + (_ae*70) \
                                        + (CASE \
                                            WHEN m.upvotes <= 40 THEN (m.upvotes * 2.2) \
                                            WHEN m.upvotes <= 100 THEN ((40 * 2.2) + ((m.upvotes - 40) * 0.7)) \
                                            ELSE ((40 * 2.2) + (60 * 0.7) + ((m.upvotes - 100) * 0.25)) \
                                          END) \
                                        + ((-1) * (julianday('now') - julianday(m.upload_date)) * 0.1))";
                    format!("ORDER BY {score} DESC, m.upload_date DESC", score = score_expr)
                }
            }
        };

        let limit = (params.page_size + 1) as i64; // one extra to detect has_more
        let offset = (params.page as i64) * (params.page_size as i64);

        let sql = format!(
            "SELECT m.id, m.song, m.artist, m.charter, m.charter_uid, m.description, m.artist_list, m.image, m.upvotes, m.upload_date, m.update_date, d.display AS level_display, d.difficulty AS level_difficulty{select_extras} \
             FROM {MAPS_TABLE} m \
             LEFT JOIN {DIFFICULTIES_TABLE} d ON d.map_id = m.id \
             {where_sql} \
             {order_sql} \
             LIMIT {limit} OFFSET {offset}",
            select_extras = select_extras
        );

        let mut stmt = self.db.prepare(&sql);
        // Bind WHERE binds followed by relevance binds in a single call to preserve parameter ordering
        let mut all_binds: Vec<JsValue> = Vec::new();
        if !where_binds.is_empty() { all_binds.extend(where_binds.clone().into_iter()); }
        if !relevance_binds.is_empty() { all_binds.extend(relevance_binds.into_iter()); }
        if !all_binds.is_empty() {
            stmt = stmt.bind(&all_binds[..])?;
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
        if !where_binds.is_empty() {
            count_stmt = count_stmt.bind(&where_binds[..])?;
        }
        let count_res = count_stmt.all().await?;
        let total_count: u64 = match count_res.results::<std::collections::BTreeMap<String, D1Value>>() {
            Ok(rows) => rows
                .first()
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
                .prepare(format!(
                    "INSERT OR IGNORE INTO {USER_UPVOTES_TABLE}(user_id, map_id) VALUES (?1, ?2)"
                ))
                .bind(&[JsValue::from_str(user_id), JsValue::from_str(map_id)])?,
        );
        statements.push(
            self.db
                .prepare(format!(
                    "UPDATE {MAPS_TABLE} SET upvotes = upvotes + 1 WHERE id = ?1"
                ))
                .bind(&[JsValue::from_str(map_id)])?,
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
                .prepare(format!(
                    "DELETE FROM {USER_UPVOTES_TABLE} WHERE user_id = ?1 AND map_id = ?2"
                ))
                .bind(&[JsValue::from_str(user_id), JsValue::from_str(map_id)])?,
        );
        statements.push(
            self.db
                .prepare(format!(
                    "UPDATE {MAPS_TABLE} SET upvotes = CASE WHEN upvotes > 0 THEN upvotes - 1 ELSE 0 END WHERE id = ?1"
                ))
                .bind(&[JsValue::from_str(map_id)])?,
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
            "SELECT m.id, m.song, m.artist, m.charter, m.charter_uid, m.description, m.artist_list, m.image, m.upvotes, m.upload_date, m.update_date, d.display AS level_display, d.difficulty AS level_difficulty FROM {MAPS_TABLE} m LEFT JOIN {DIFFICULTIES_TABLE} d ON d.map_id = m.id WHERE m.charter_uid = ?1 AND m.deleted = 0 ORDER BY m.upload_date DESC LIMIT 100"
        );
        let stmt = self.db.prepare(&sql);
        let result = stmt.bind(&[JsValue::from_str(user_id)])?.all().await?;
        Ok(Self::group_maps(Self::rows(result)))
    }

    async fn delete_map(&self, map_id: &str) -> Result<()> {
        let mut statements = Vec::new();
        statements.push(
            self.db
                .prepare(format!("DELETE FROM {DIFFICULTIES_TABLE} WHERE map_id = ?1"))
                .bind(&[JsValue::from_str(map_id)])?,
        );
        statements.push(
            self.db
                .prepare(format!("DELETE FROM {USER_UPVOTES_TABLE} WHERE map_id = ?1"))
                .bind(&[JsValue::from_str(map_id)])?,
        );
        statements.push(
            self.db
                .prepare(format!("DELETE FROM {MAPS_TABLE} WHERE id = ?1"))
                .bind(&[JsValue::from_str(map_id)])?,
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
        let stmt = self.db.prepare(format!(
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

    async fn find_map_by_identity(
        &self,
        song: &str,
        artist: &str,
        charter: &str,
        charter_uid: Uuid,
    ) -> Result<Option<Uuid>> {
        let sql = format!(
            "SELECT id FROM {MAPS_TABLE}
             WHERE charter_uid = ?4
               AND song = ?1
               AND artist = ?2
               AND charter = ?3
             LIMIT 1"
        );
        let stmt = self.db.prepare(&sql);
        let res = stmt
            .bind(&[
                JsValue::from_str(song),
                JsValue::from_str(artist),
                JsValue::from_str(charter),
                JsValue::from_str(&charter_uid.to_string()),
            ])?
            .all()
            .await?;
        let rows = Self::rows(res);
        let id = rows
            .first()
            .and_then(|r| r.get("id"))
            .and_then(Self::parse_uuid);
        Ok(id)
    }

    async fn update_map_metadata(&self, updated: &UpdateMap<'_>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let stmt = self.db.prepare(format!(
            "UPDATE {MAPS_TABLE} SET song = ?1, artist = ?2, charter = ?3, description = ?4, artist_list = ?5, image = COALESCE(?6, image), update_date = ?7 WHERE id = ?8"
        ));
        let image_val = match updated.image {
            Some(true) => JsValue::from_f64(1.0),
            Some(false) => JsValue::from_f64(0.0),
            None => JsValue::NULL,
        };
        stmt
            .bind(&[
                JsValue::from_str(updated.song),
                JsValue::from_str(updated.artist),
                JsValue::from_str(updated.charter),
                JsValue::from_str(updated.description),
                JsValue::from_str(updated.artist_list),
                image_val,
                JsValue::from_str(&now),
                JsValue::from_str(&updated.id.to_string()),
            ])?
            .run()
            .await
            .map_err(|e| Error::RustError(format!("d1 update map: {e}")))?;
        Ok(())
    }
}
