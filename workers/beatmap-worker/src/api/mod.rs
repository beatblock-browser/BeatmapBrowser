use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use worker::{Env, Request, Response, Result, Error};
use chrono::{DateTime, Utc};

use crate::json_response;
use crate::util::auth::{require_auth, optional_auth};
use crate::util::db::{Database, DatabaseBackend, SearchParams, SortBy};
use crate::util::roles::is_admin;
pub mod upvote;
pub mod usersongs;
pub mod delete;
pub mod signin;
pub mod upload;
pub mod moderation;

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub min_upvotes: Option<u64>,
    pub difficulties: Option<Vec<String>>, // matches LevelVariant.display
    pub sort: Option<SortBy>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

 

#[derive(Debug, Deserialize)]
pub struct CheckDuplicateRequest {
    pub song: String,
    pub artist: String,
    pub charter: String,
}

#[derive(Debug, Serialize)]
pub struct CheckDuplicateResponse {
    pub exists: bool,
    pub id: Option<Uuid>,
}

pub async fn handle_check_duplicate(mut req: Request, env: Env) -> Result<Response> {
    let claims = require_auth(&req, &env)?;
    let charter_uid = Uuid::parse_str(&claims.sub)
        .map_err(|_| Error::RustError("Invalid user id in token".into()))?;
    let payload: CheckDuplicateRequest = req.json().await?;
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let existing = db
        .find_map_by_identity(
            &payload.song,
            &payload.artist,
            &payload.charter,
            charter_uid,
        )
        .await?;
    let body = CheckDuplicateResponse { exists: existing.is_some(), id: existing };
    json_response(200, &body)
}

pub async fn handle_ping(_req: Request, env: Env) -> Result<Response> {
    // Simple D1 roundtrip to check connectivity and measure time
    let t0 = chrono::Utc::now();
    let db = env.d1("beatblockbrowser")?;
    let stmt = db.prepare("SELECT 1 as ok");
    let res = stmt.all().await;
    match res {
        Ok(rows) => {
            let t1 = chrono::Utc::now();
            let ok = !rows.results::<JsonValue>().unwrap_or_default().is_empty();
            json_response(200, &serde_json::json!({
                "ok": ok,
                "timing_ms": (t1 - t0).num_milliseconds()
            }))
        }
        Err(e) => json_response(500, &serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub query: String,
    pub results: Vec<BeatMap>,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
    pub total_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelVariant {
    pub display: String,
    pub difficulty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatMap {
    pub song: String,
    pub artist: String,
    pub charter: String,
    pub charter_uid: Uuid,
    pub difficulties: Vec<LevelVariant>,
    pub description: String,
    pub artist_list: String,
    pub image: bool,
    pub upvotes: u64,
    pub upload_date: DateTime<Utc>,
    pub update_date: DateTime<Utc>,
    pub id: Uuid,
}

pub async fn handle_search(mut req: Request, env: Env) -> Result<Response> {
    let payload: SearchRequest = req.json().await?;
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let page = payload.page.unwrap_or(0);
    let page_size = payload.page_size.unwrap_or(20).clamp(1, 100);
    let params = SearchParams {
        query: payload.query.clone(),
        min_upvotes: payload.min_upvotes,
        difficulties: payload.difficulties.unwrap_or_default(),
        sort: payload.sort.unwrap_or(SortBy::Newest),
        page,
        page_size,
    };

    let (results, has_more, total_count) = db.search_songs(&params).await?;

    let body = SearchResult {
        query: payload.query,
        results,
        page,
        page_size,
        has_more,
        total_count,
    };
    json_response(200, &body)
}

pub async fn handle_get_map(
    _req: Request,
    env: Env,
    map_id: String,
) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let id = Uuid::parse_str(&map_id).map_err(|_| Error::RustError("Invalid map id".into()))?;
    match db.get_map_by_id(id).await? {
        Some(map) => {
            // Check soft-deleted visibility
            let deleted = db.is_map_deleted(&map.id.to_string()).await.unwrap_or(false);
            if !deleted {
                return json_response(200, &map);
            }
            // Only moderators or admins can view deleted maps
            // Try to read optional auth from a dummy request we don't have here; caller passes _req
            // Note: we received _req above; use it now
            let mut can_view = false;
            // We don't have claims here; require optional auth via headers not available (since _req is available)
            // Re-fetching is fine: the original request context is present
            if let Some(claims) = optional_auth(&_req, &env) {
                if let Ok(uid) = Uuid::parse_str(&claims.sub) {
                    if is_admin(&uid) {
                        can_view = true;
                    } else if db.is_user_moderator(&claims.sub).await.unwrap_or(false) {
                        can_view = true;
                    }
                }
            }
            if can_view {
                json_response(200, &map)
            } else {
                json_response(404, &serde_json::json!({
                    "error": "Beatmap not found"
                }))
            }
        },
        None => json_response(404, &serde_json::json!({
            "error": "Beatmap not found"
        })),
    }
}
