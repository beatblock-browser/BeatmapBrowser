use serde::Deserialize;
use worker::{Env, Request, Response, Result};

use crate::json_response;
use crate::util::db::{Database, DatabaseBackend};
use crate::util::auth::require_auth;

#[derive(Debug, Deserialize)]
pub struct UpvoteRequest {
    pub map_id: String,
}

pub async fn upvote(mut req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let payload: UpvoteRequest = req.json().await?;
    let claims = require_auth(&req, &env)?;
    db.upvote_map(&claims.sub, &payload.map_id).await?;
    json_response(200, &serde_json::json!({ "status": "ok" }))
}

pub async fn unvote(mut req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let payload: UpvoteRequest = req.json().await?;
    let claims = require_auth(&req, &env)?;
    db.unvote_map(&claims.sub, &payload.map_id).await?;
    json_response(200, &serde_json::json!({ "status": "ok" }))
}
