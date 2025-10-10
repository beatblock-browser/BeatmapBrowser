use serde::Deserialize;
use uuid::Uuid;
use worker::{Env, Request, Response, Result, Error};

use crate::json_response;
use crate::util::auth::require_auth;
use crate::util::db::DatabaseBackend;
use crate::util::roles::is_admin;

#[derive(Debug, Deserialize)]
pub struct SetModeratorRequest {
    pub user_id: String,
    pub make_moderator: bool,
}

#[derive(Debug, Deserialize)]
pub struct MapStatusRequest {
    pub map_id: String,
}

pub async fn map_status(mut req: Request, env: Env) -> Result<Response> {
    // Auth required to reveal deletion status; mods/admins may see deleted
    let claims = require_auth(&req, &env)?;
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let is_mod = db.is_user_moderator(&claims.sub).await.unwrap_or(false);
    let is_admin_user = Uuid::parse_str(&claims.sub).ok().map(|u| is_admin(&u)).unwrap_or(false);
    let payload: MapStatusRequest = req.json().await?;
    let is_deleted = db.is_map_deleted(&payload.map_id).await.unwrap_or(false);
    json_response(200, &serde_json::json!({
        "is_deleted": is_deleted,
        "is_moderator": is_mod || is_admin_user,
        "is_admin": is_admin_user
    }))
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub map_id: String,
}

pub async fn set_moderator(mut req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let claims = require_auth(&req, &env)?;
    let uid = Uuid::parse_str(&claims.sub).map_err(|e| Error::RustError(format!("invalid user id: {e}")))?;
    if !is_admin(&uid) {
        return json_response(403, &serde_json::json!({ "error": "Admin required" }));
    }
    let payload: SetModeratorRequest = req.json().await?;
    let target = Uuid::parse_str(&payload.user_id).map_err(|e| Error::RustError(format!("invalid target user id: {e}")))?;
    db.set_user_moderator(&target.to_string(), payload.make_moderator).await?;
    json_response(200, &serde_json::json!({ "status": "ok" }))
}

pub async fn restore_map(mut req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let claims = require_auth(&req, &env)?;
    let uid = Uuid::parse_str(&claims.sub).map_err(|e| Error::RustError(format!("invalid user id: {e}")))?;
    if !is_admin(&uid) {
        return json_response(403, &serde_json::json!({ "error": "Admin required" }));
    }
    let payload: RestoreRequest = req.json().await?;
    let _ = Uuid::parse_str(&payload.map_id).map_err(|e| Error::RustError(format!("invalid map id: {e}")))?;
    db.set_map_deleted(&payload.map_id, false, None).await?;
    json_response(200, &serde_json::json!({ "status": "ok" }))
}
