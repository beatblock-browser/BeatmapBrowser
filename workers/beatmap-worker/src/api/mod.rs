use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;
use worker::{Env, Request, Response, Result, Error};
use chrono::{DateTime, Utc};
use log::{info, error};

use crate::json_response;
use crate::util::db::{Database, DatabaseBackend};

pub mod upvote;
pub mod usersongs;
pub mod delete;
pub mod signin;
pub mod upload;

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
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
            let ok = rows.results::<JsonValue>().unwrap_or_default().len() > 0;
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
    info!("/api/search: request received");
    let payload: SearchRequest = req.json().await.map_err(|e| {
        error!("/api/search: failed to parse JSON body: {}", e);
        e
    })?;

    info!("/api/search: query='{}'", payload.query);
    let db: DatabaseBackend = DatabaseBackend::from_env(&env).map_err(|e| {
        error!("/api/search: failed to init database binding: {}", e);
        e
    })?;
    let results = db.search_songs(&payload.query).await.map_err(|e| {
        error!("/api/search: database error for query '{}': {}", payload.query, e);
        e
    })?;
    info!("/api/search: success, {} results", results.len());

    let body = SearchResult {
        query: payload.query,
        results,
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
        Some(map) => json_response(200, &map),
        None => json_response(404, &serde_json::json!({
            "error": "Beatmap not found"
        })),
    }
}
